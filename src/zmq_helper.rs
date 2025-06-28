use core::{
    future::{Future, poll_fn},
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use anyhow::{Error, Result, anyhow};
use derive_more::AsRef;
use futures::FutureExt;
use tap::TryConv;
use tokio::{
    select,
    sync::{
        Mutex,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
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
struct DealerSocket(Socket);

unsafe impl Send for DealerSocket {}
unsafe impl Sync for DealerSocket {}

impl TryFrom<&Context> for DealerSocket {
    type Error = Error;

    fn try_from(context: &Context) -> Result<Self> {
        let socket = context.socket(SocketType::DEALER)?;

        Ok(Self(socket))
    }
}

impl Future for DealerSocket {
    type Output = Result<Message>;

    fn poll(self: Pin<&mut Self>, _ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(
            (*self)
                .as_ref()
                .recv_msg(zmq::DONTWAIT)
                .map_err(Error::from),
        )
    }
}

impl DealerSocket {
    fn configure(&self, password: &str, identity: &str) -> Result<()> {
        self.as_ref().set_plain_username(Some("rcon"))?;
        if !password.is_empty() {
            self.as_ref().set_plain_password(Some(password))?;
        } else {
            self.as_ref().set_plain_password(None)?;
        }

        let identity_str = if identity.is_empty() {
            let identity = Uuid::new_v4();
            identity.to_string().replace("-", "")
        } else {
            identity.to_string()
        };

        self.as_ref().set_identity(identity_str.as_bytes())?;
        self.as_ref().set_rcvtimeo(100)?;
        self.as_ref().set_sndtimeo(100)?;
        self.as_ref().set_heartbeat_ivl(600_000)?;
        self.as_ref().set_heartbeat_timeout(600_000)?;

        self.as_ref().set_zap_domain("rcon")?;

        Ok(())
    }

    fn disconnect(&self) -> Result<()> {
        self.as_ref()
            .get_last_endpoint()?
            .map_err(|_err| zmq::Error::EFAULT)
            .and_then(|last_endpoint| self.as_ref().disconnect(&last_endpoint))?;

        Ok(())
    }

    fn get_endpoint(&self) -> Result<String> {
        let fd = self.as_ref().get_fd()?;

        Ok(format!("inproc://monitor.s-{fd}"))
    }

    fn monitor(&self, context: &Context, events: SocketEvent) -> Result<MonitoringSocket> {
        let monitor_endpoint = self.get_endpoint()?;
        self.as_ref().monitor(&monitor_endpoint, events as i32)?;

        let monitor = context.socket(zmq::PAIR)?;

        Ok(MonitoringSocket(monitor))
    }
}

#[derive(AsRef)]
#[as_ref(Socket)]
struct MonitoringSocket(Socket);

unsafe impl Send for MonitoringSocket {}
unsafe impl Sync for MonitoringSocket {}

impl TryFrom<(&Context, &DealerSocket)> for MonitoringSocket {
    type Error = Error;

    fn try_from((ctx, socket): (&Context, &DealerSocket)) -> Result<Self> {
        socket.monitor(ctx, SocketEvent::ALL)
    }
}

impl Future for MonitoringSocket {
    type Output = Result<MultipartMessage>;

    fn poll(self: Pin<&mut Self>, _ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(
            (*self)
                .as_ref()
                .recv_multipart(zmq::DONTWAIT)
                .map_err(Error::from)
                .and_then(MultipartMessage::try_from),
        )
    }
}

impl MonitoringSocket {
    fn disconnect(&self) -> Result<()> {
        self.as_ref()
            .get_last_endpoint()?
            .map_err(|_err| zmq::Error::EFAULT)
            .and_then(|last_endpoint| self.as_ref().disconnect(&last_endpoint))?;

        Ok(())
    }
}

struct ZmqSocketPair {
    dealer: Mutex<DealerSocket>,
    monitor: Mutex<MonitoringSocket>,
}

unsafe impl Send for ZmqSocketPair {}
unsafe impl Sync for ZmqSocketPair {}

impl ZmqSocketPair {
    fn new() -> Result<Self> {
        let context = Context::new();
        let socket = (&context).try_conv::<DealerSocket>()?;
        let monitor = (&context, &socket).try_conv::<MonitoringSocket>()?.into();

        Ok(Self {
            dealer: socket.into(),
            monitor,
        })
    }

    async fn configure(&self, password: &str, identity: &str) -> Result<()> {
        self.dealer.lock().await.configure(password, identity)
    }

    async fn connect(&self, address: &str) -> Result<()> {
        let monitor_endpoint = self.dealer.lock().await.get_endpoint()?;
        self.monitor
            .lock()
            .await
            .as_ref()
            .connect(&monitor_endpoint)?;
        self.dealer.lock().await.as_ref().connect(address)?;

        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        self.monitor.lock().await.disconnect()?;
        self.dealer.lock().await.disconnect()?;

        Ok(())
    }

    async fn send(&self, msg: &str, flags: i32) -> Result<()> {
        self.dealer.lock().await.as_ref().send(msg, flags)?;

        Ok(())
    }

    async fn recv_msg(&mut self) -> Result<Message> {
        let mut socket = self.dealer.lock().await;
        poll_fn(|ctx| socket.poll_unpin(ctx)).await
    }

    async fn monitor(&self) -> Result<MultipartMessage> {
        let mut monitor = self.monitor.lock().await;
        poll_fn(|ctx| (*monitor).poll_unpin(ctx)).await
    }

    async fn zmq_poll(&self) -> Result<i32> {
        self.dealer
            .lock()
            .await
            .as_ref()
            .poll(zmq::POLLIN, 100)
            .map_err(Error::from)
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

    let mut zmq_socket_pair = ZmqSocketPair::new()?;
    zmq_socket_pair
        .configure(&args.password, &args.identity)
        .await?;

    zmq_socket_pair.connect(&args.host).await?;

    let first = AtomicBool::new(true);

    'outer: while let Ok(event) = zmq_socket_pair.zmq_poll().await {
        if !CONTINUE_RUNNING.load(Ordering::Acquire) {
            break;
        }

        match zmq_socket_pair.monitor().await {
            Ok(MultipartMessage {
                event_id: SocketEvent::CONNECTED,
                ..
            }) => {
                if first.load(Ordering::Acquire) {
                    first.store(false, Ordering::Release);
                    display_sender.send("ZMQ registering with the server.".to_string())?;
                }
                if let Err(e) = zmq_socket_pair.send("register", zmq::DONTWAIT).await {
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
                if let Err(e) = zmq_socket_pair
                    .connect(msg.as_str().unwrap_or(&args.host))
                    .await
                {
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
            Err(err)
                if err
                    .downcast_ref::<zmq::Error>()
                    .is_some_and(|&zmq_error| zmq_error == zmq::Error::EAGAIN) => {}
            Err(err) => {
                display_sender.send(format!("ZMQ error: {err}"))?;
            }
        }

        loop {
            select!(
                Some(line) = poll_receiver(&mut zmq_receiver) => {
                    zmq_socket_pair.send(&line, zmq::DONTWAIT).await?;
                },
                Ok(zmq_msg) = zmq_socket_pair.recv_msg(), if event != 0 => {
                    if let Some(zmq_str) = zmq_msg.as_str() {
                        display_sender.send(trim_ql_msg(zmq_str))?;
                    }
                }
                else => {
                    continue 'outer;
                }
            );
        }
    }

    drop(zmq_receiver);

    if CONTINUE_RUNNING.load(Ordering::SeqCst) {
        display_sender.send("Exiting ZMQ...".to_string())?;
    }

    zmq_socket_pair.disconnect().await?;

    drop(display_sender);

    CONTINUE_RUNNING.store(false, Ordering::SeqCst);

    Ok(())
}
