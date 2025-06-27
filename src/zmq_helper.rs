use core::{
    future::poll_fn,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use anyhow::{Error, Result, anyhow};
use derive_more::AsRef;
use tap::TryConv;
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use uuid::Uuid;
use zmq::{Context, Message, Socket, SocketEvent, SocketType};

use crate::{CONTINUE_RUNNING, cmd_line::CommandLineOptions};

#[derive(Debug, PartialEq)]
pub struct MultipartMessage {
    event_id: SocketEvent,
    #[allow(dead_code)]
    event_value: u32,
    #[allow(dead_code)]
    msg: Message,
}

impl TryFrom<Vec<Vec<u8>>> for MultipartMessage {
    type Error = Error;

    fn try_from(message: Vec<Vec<u8>>) -> Result<Self, Self::Error> {
        if message.len() != 2 {
            return Err(anyhow!("invalid msg received"));
        }

        let Some(first_msg) = message.first() else {
            return Err(anyhow!("invalid msg received"));
        };

        if first_msg.len() != 6 {
            return Err(anyhow!("invalid msg received"));
        }

        let Some(event_id) = first_msg
            .first_chunk::<2>()
            .map(|raw_event_id| u16::from_le_bytes(*raw_event_id))
            .map(SocketEvent::from_raw)
        else {
            return Err(anyhow!("invalid first two bytes"));
        };

        let Some(event_value) = first_msg
            .last_chunk::<4>()
            .map(|raw_event_value| u32::from_le_bytes(*raw_event_value))
        else {
            return Err(anyhow!("invalid last four bytes"));
        };

        let Some(msg) = message.get(1) else {
            return Err(anyhow!("invalid msg received"));
        };

        Ok(Self {
            event_id,
            event_value,
            msg: msg.into(),
        })
    }
}

#[derive(AsRef)]
#[as_ref(Socket)]
pub struct MonitoringSocket(Socket);

impl TryFrom<(&Context, &Socket)> for MonitoringSocket {
    type Error = Error;

    fn try_from((ctx, socket): (&Context, &Socket)) -> Result<Self> {
        let fd = socket.get_fd()?;

        let monitor_endpoint = format!("inproc://monitor.s-{fd}");
        socket.monitor(&monitor_endpoint, SocketEvent::ALL as i32)?;

        let monitor = ctx.socket(zmq::PAIR)?;

        Ok(Self(monitor))
    }
}

struct ZmqSocketPair {
    socket: Socket,
    monitor: MonitoringSocket,
}

unsafe impl Send for ZmqSocketPair {}
unsafe impl Sync for ZmqSocketPair {}

impl ZmqSocketPair {
    fn new() -> Result<Self> {
        let context = Context::new();
        let socket = context.socket(SocketType::DEALER)?;
        let monitor = (&context, &socket).try_conv::<MonitoringSocket>()?;

        Ok(Self { socket, monitor })
    }

    fn configure(&self, password: &str, identity: &str) -> Result<()> {
        self.socket.set_plain_username(Some("rcon"))?;
        if !password.is_empty() {
            self.socket.set_plain_password(Some(password))?;
        } else {
            self.socket.set_plain_password(None)?;
        }

        let identity_str = if identity.is_empty() {
            let identity = Uuid::new_v4();
            identity.to_string().replace("-", "")
        } else {
            identity.to_string()
        };

        self.socket.set_identity(identity_str.as_bytes())?;
        self.socket.set_rcvtimeo(100)?;
        self.socket.set_sndtimeo(100)?;
        self.socket.set_heartbeat_ivl(600_000)?;
        self.socket.set_heartbeat_timeout(600_000)?;

        self.socket.set_zap_domain("rcon")?;

        Ok(())
    }

    fn connect(&self, address: &str) -> Result<()> {
        let fd = self.socket.get_fd()?;

        let monitor_endpoint = format!("inproc://monitor.s-{fd}");
        self.monitor.as_ref().connect(&monitor_endpoint)?;
        self.socket.connect(address)?;

        Ok(())
    }

    fn disconnect(&self) -> Result<()> {
        self.monitor
            .as_ref()
            .get_last_endpoint()?
            .map_err(|_err| zmq::Error::EFAULT)
            .and_then(|last_endpoint| self.monitor.as_ref().disconnect(&last_endpoint))?;

        self.socket
            .get_last_endpoint()?
            .map_err(|_err| zmq::Error::EFAULT)
            .and_then(|last_endpoint| self.socket.disconnect(&last_endpoint))
            .map_err(Error::from)
    }

    fn send(&self, msg: &str, flags: i32) -> Result<()> {
        self.socket.send(msg, flags)?;

        Ok(())
    }

    fn recv_msg(&self) -> Poll<Result<Message>> {
        Poll::Ready(self.socket.recv_msg(zmq::DONTWAIT).map_err(Error::from))
    }

    fn recv_multipart(&self) -> Poll<Result<MultipartMessage>> {
        Poll::Ready(
            self.monitor
                .as_ref()
                .recv_multipart(zmq::DONTWAIT)
                .map_err(Error::from)
                .and_then(MultipartMessage::try_from),
        )
    }

    fn poll(&self) -> Poll<Result<i32>> {
        Poll::Ready(self.socket.poll(zmq::POLLIN, 100).map_err(Error::from))
    }
}

async fn poll_receiver<T>(receiver: &mut UnboundedReceiver<T>) -> Option<T> {
    if receiver.is_closed() || receiver.is_empty() {
        return None;
    }
    receiver.recv().await
}

fn trim_ql_msg(msg: &str) -> String {
    msg.replace("\n", "")
        .replace("\\n", "")
        .replace('\u{0019}', "")
}

pub(crate) async fn run_zmq(
    args: CommandLineOptions,
    mut zmq_receiver: UnboundedReceiver<String>,
    display_sender: UnboundedSender<String>,
) -> Result<()> {
    display_sender.send(format!("ZMQ connecting to {}...", &args.host))?;

    let zmq_socket_pair = ZmqSocketPair::new()?;
    zmq_socket_pair.configure(&args.password, &args.identity)?;

    zmq_socket_pair.connect(&args.host)?;

    let first = AtomicBool::new(true);

    while let Ok(event) = poll_fn(|_| zmq_socket_pair.poll()).await {
        if !CONTINUE_RUNNING.load(Ordering::Acquire) {
            break;
        }

        match poll_fn(|_| zmq_socket_pair.recv_multipart()).await {
            Ok(MultipartMessage {
                event_id: SocketEvent::CONNECTED,
                ..
            }) => {
                if first.load(Ordering::Acquire) {
                    first.store(false, Ordering::Release);
                    display_sender.send("ZMQ registering with the server.".to_string())?;
                }
                if let Err(e) = zmq_socket_pair.send("register", zmq::DONTWAIT) {
                    display_sender.send(format!("error registering with ZMQ: {e:?}."))?;
                }
            }
            Ok(MultipartMessage {
                event_id: SocketEvent::CONNECT_DELAYED | SocketEvent::CONNECT_RETRIED,
                ..
            }) => {
                continue;
            }
            Ok(MultipartMessage {
                event_id: SocketEvent::HANDSHAKE_SUCCEEDED,
                ..
            }) => {
                first.store(true, Ordering::Release);
                display_sender.send(format!("ZMQ connected to {}.", &args.host))?;
            }
            Ok(MultipartMessage {
                event_id:
                    event_id @ (SocketEvent::HANDSHAKE_FAILED_AUTH
                    | SocketEvent::HANDSHAKE_FAILED_PROTOCOL
                    | SocketEvent::HANDSHAKE_FAILED_NO_DETAIL
                    | SocketEvent::MONITOR_STOPPED),
                event_value,
                ..
            }) => {
                display_sender.send(format!(
                    "ZMQ socket error: {:#06x?} {event_id:?}({event_value:?}).",
                    event_id.to_raw()
                ))?;
                break;
            }
            Ok(MultipartMessage {
                event_id: SocketEvent::DISCONNECTED | SocketEvent::CLOSED,
                msg,
                ..
            }) => {
                if first.load(Ordering::Acquire) {
                    first.store(false, Ordering::Release);
                    display_sender.send("Reconnecting ZMQ...".to_string())?;
                }
                if let Err(e) = zmq_socket_pair.connect(msg.as_str().unwrap_or(&args.host)) {
                    display_sender.send(format!("error reconnecting: {e:?}."))?;
                }
            }
            Ok(MultipartMessage {
                event_id,
                event_value,
                ..
            }) => {
                display_sender.send(format!(
                    "ZMQ socket error: {:#06x?} {event_id:?}({event_value:?}).",
                    event_id.to_raw()
                ))?;
            }
            Err(err) => match err.downcast_ref::<zmq::Error>() {
                Some(zmq::Error::EAGAIN) => (),
                _ => {
                    display_sender.send(format!("ZMQ error: {err}"))?;
                }
            },
        }

        'inner: loop {
            select!(
                Some(line) = poll_receiver(&mut zmq_receiver) => {
                    zmq_socket_pair.send(&line, zmq::DONTWAIT)?;
                },
                Ok(zmq_msg) = poll_fn(|_| zmq_socket_pair.recv_msg()), if event != 0 => {
                    if let Some(zmq_str) = zmq_msg.as_str() {
                        display_sender.send(trim_ql_msg(zmq_str))?;
                    }
                }
                else => {
                    break 'inner;
                }
            );
        }
    }

    drop(zmq_receiver);

    if CONTINUE_RUNNING.load(Ordering::SeqCst) {
        display_sender.send("Exiting ZMQ...".to_string())?;
    }

    zmq_socket_pair.disconnect()?;

    drop(display_sender);

    CONTINUE_RUNNING.store(false, Ordering::SeqCst);

    Ok(())
}
