use core::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Error, Result, anyhow};
use derive_more::AsRef;
use tap::TryConv;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
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

        #[cfg(target_endian = "little")]
        let Some(event_id) = first_msg
            .first_chunk::<2>()
            .map(|raw_event_id| u16::from_le_bytes(*raw_event_id))
            .map(SocketEvent::from_raw)
        else {
            return Err(anyhow!("invalid first two bytes"));
        };
        #[cfg(not(target_endian = "little"))]
        let Some(event_id) = first_msg
            .first_chunk::<2>()
            .map(|raw_event_id| u16::from_be_bytes(*raw_event_id))
            .map(SocketEvent::from_raw)
        else {
            return Err(anyhow!("invalid first two bytes"));
        };

        #[cfg(target_endian = "little")]
        let Some(event_value) = first_msg
            .last_chunk::<4>()
            .map(|raw_event_value| u32::from_le_bytes(*raw_event_value))
        else {
            return Err(anyhow!("invalid last four bytes"));
        };
        #[cfg(not(target_endian = "little"))]
        let Some(event_value) = first_msg
            .last_chunk::<4>()
            .map(|raw_event_value| u32::from_be_bytes(*raw_event_value))
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

        monitor.connect(&monitor_endpoint)?;

        Ok(Self(monitor))
    }
}

impl MonitoringSocket {
    pub fn disconnect(&self) -> Result<()> {
        self.as_ref()
            .get_last_endpoint()?
            .map_err(|_err| zmq::Error::EFAULT)
            .and_then(|last_endpoint| self.as_ref().disconnect(&last_endpoint))
            .map_err(Error::from)
    }
}

pub(crate) fn create_socket(
    context: &Context,
    password: &str,
    identity: &str,
) -> zmq::Result<Socket> {
    let socket = context.socket(SocketType::DEALER)?;

    socket.set_plain_username(Some("rcon"))?;
    if !password.is_empty() {
        socket.set_plain_password(Some(password))?;
    } else {
        socket.set_plain_password(None)?;
    }

    let identity_str = if identity.is_empty() {
        let identity = Uuid::new_v4();
        identity.to_string().replace("-", "")
    } else {
        identity.to_string()
    };

    socket.set_identity(identity_str.as_bytes())?;

    socket.set_zap_domain("rcon")?;

    Ok(socket)
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

    let zmq_context = Context::new();
    let socket = create_socket(&zmq_context, &args.password, &args.identity)?;

    let monitor_socket = (&zmq_context, &socket).try_conv::<MonitoringSocket>()?;

    socket.connect(&args.host)?;

    let first = AtomicBool::new(true);

    while let Ok(event) = socket.poll(zmq::POLLIN, 100) {
        if !CONTINUE_RUNNING.load(Ordering::Acquire) {
            break;
        }

        match monitor_socket
            .as_ref()
            .recv_multipart(zmq::DONTWAIT)
            .map_err(Error::from)
            .and_then(MultipartMessage::try_from)
        {
            Ok(MultipartMessage {
                event_id: SocketEvent::CONNECTED,
                ..
            }) => {
                if first.load(Ordering::Acquire) {
                    first.store(false, Ordering::Release);
                    display_sender.send("ZMQ registering with the server.".to_string())?;
                }
                if let Err(e) = socket.send("register", zmq::DONTWAIT) {
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
                    SocketEvent::HANDSHAKE_FAILED_AUTH
                    | SocketEvent::CLOSED
                    | SocketEvent::HANDSHAKE_FAILED_PROTOCOL
                    | SocketEvent::HANDSHAKE_FAILED_NO_DETAIL
                    | SocketEvent::MONITOR_STOPPED,
                ..
            }) => {
                break;
            }
            Ok(MultipartMessage {
                event_id: SocketEvent::DISCONNECTED,
                ..
            }) => {
                if first.load(Ordering::Acquire) {
                    first.store(false, Ordering::Release);
                    display_sender.send("Reconnecting ZMQ...".to_string())?;
                }
                if let Err(e) = socket.connect(&args.host) {
                    display_sender.send(format!("error reconnecting: {e:?}."))?;
                }
            }
            Ok(MultipartMessage {
                event_id: socket_event,
                ..
            }) => {
                display_sender.send(format!(
                    "{:#06x?} {:?}",
                    socket_event.to_raw(),
                    socket_event
                ))?;
            }
            Err(err) => match err.downcast_ref::<zmq::Error>() {
                Some(zmq::Error::EAGAIN) => (),
                _ => {
                    display_sender.send(format!("zmq error: {err}"))?;
                }
            },
        }

        if zmq_receiver.is_closed() {
            display_sender.send("receiver disconnected".to_string())?;
            break;
        }

        while !zmq_receiver.is_empty() {
            if let Some(line) = zmq_receiver.recv().await {
                socket.send(&line, zmq::DONTWAIT)?;
            }
        }

        if event == 0 {
            continue;
        }

        while let Ok(zmq_msg) = socket.recv_msg(zmq::DONTWAIT) {
            if let Some(zmq_str) = zmq_msg.as_str() {
                display_sender.send(trim_ql_msg(zmq_str))?;
            }
        }
    }

    drop(zmq_receiver);

    if CONTINUE_RUNNING.load(Ordering::SeqCst) {
        display_sender.send("Exiting ZMQ...".to_string())?;
    }

    monitor_socket.disconnect()?;
    socket.disconnect(&args.host)?;

    drop(display_sender);

    CONTINUE_RUNNING.store(false, Ordering::SeqCst);

    Ok(())
}
