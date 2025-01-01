use crate::{CONTINUE_RUNNING, cmd_line::CommandLineOptions};

use std::{
    ops::Deref,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::Result;

use crossbeam_channel::{Receiver, Sender};

use uuid::Uuid;

use zmq::{Context, DONTWAIT, Message, POLLIN, Socket, SocketEvent, SocketType};

pub struct MultipartMessage {
    event_id: u16,
    #[allow(dead_code)]
    event_value: u32,
    #[allow(dead_code)]
    msg: Message,
}

impl TryFrom<Vec<Vec<u8>>> for MultipartMessage {
    type Error = &'static str;

    fn try_from(msg: Vec<Vec<u8>>) -> Result<Self, Self::Error> {
        if msg.len() != 2 {
            return Err("invalid msg received");
        }

        if msg[0].len() != 6 {
            return Err("invalid msg received");
        }

        let Some(raw_event_id) = msg[0].first_chunk::<2>() else {
            return Err("invalid first two bytes");
        };
        let event_id = if cfg!(target_endian = "little") {
            u16::from_le_bytes(*raw_event_id)
        } else {
            u16::from_be_bytes(*raw_event_id)
        };

        let Some(raw_event_value) = msg[0].last_chunk::<4>() else {
            return Err("invalid last four bytes");
        };
        let event_value = if cfg!(target_endian = "little") {
            u32::from_le_bytes(*raw_event_value)
        } else {
            u32::from_be_bytes(*raw_event_value)
        };

        let message = Message::from(msg[1].clone());

        Ok(Self {
            event_id,
            event_value,
            msg: message,
        })
    }
}

impl MultipartMessage {
    pub fn get_event_id(&self) -> SocketEvent {
        SocketEvent::from_raw(self.event_id)
    }

    #[allow(dead_code)]
    pub fn get_event_value(&self) -> u32 {
        self.event_value
    }

    #[allow(dead_code)]
    pub fn get_msg(&self) -> &Message {
        &self.msg
    }
}

pub struct MonitoringSocket(Socket);

impl TryFrom<(&Context, &Socket)> for MonitoringSocket {
    type Error = zmq::Error;

    fn try_from((ctx, socket): (&Context, &Socket)) -> zmq::Result<Self> {
        let fd = socket.get_fd()?;

        let monitor_endpoint = format!("inproc://monitor.s-{fd}");
        socket.monitor(&monitor_endpoint, SocketEvent::ALL as i32)?;

        let monitor = ctx.socket(zmq::PAIR)?;

        monitor.connect(&monitor_endpoint)?;

        Ok(Self(monitor))
    }
}

impl AsRef<Socket> for MonitoringSocket {
    fn as_ref(&self) -> &Socket {
        &self.0
    }
}

impl Deref for MonitoringSocket {
    type Target = Socket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MonitoringSocket {
    pub fn disconnect(&self) -> zmq::Result<()> {
        self.0
            .get_last_endpoint()?
            .map_or(Err(zmq::Error::EFAULT), |last_endpoint| {
                self.0.disconnect(&last_endpoint)
            })
    }

    pub fn recv_multipart(&self, flags: i32) -> zmq::Result<MultipartMessage> {
        self.0.recv_multipart(flags).and_then(|zmq_msg| {
            MultipartMessage::try_from(zmq_msg).map_err(|_| zmq::Error::EMSGSIZE)
        })
    }
}

pub(crate) fn create_socket(
    context: &Context,
    address: &str,
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

    socket.connect(address)?;

    Ok(socket)
}

fn trim_ql_msg(msg: &str) -> String {
    msg.replace("\n", "")
        .replace("\\n", "")
        .replace('\u{0019}', "")
}

pub(crate) fn run_zmq(
    args: CommandLineOptions,
    zmq_receiver: Receiver<String>,
    display_sender: Sender<String>,
) -> Result<()> {
    display_sender.send(format!("ZMQ connecting to {}...", &args.host))?;

    let zmq_context = Context::new();
    let socket = create_socket(&zmq_context, &args.host, &args.password, &args.identity)?;

    let monitor_socket = MonitoringSocket::try_from((&zmq_context, &socket))?;

    let first = AtomicBool::new(true);

    while let Ok(event) = socket.poll(POLLIN, 100) {
        if !CONTINUE_RUNNING.load(Ordering::SeqCst) {
            break;
        }

        match monitor_socket.recv_multipart(DONTWAIT) {
            Ok(zmq_msg) => {
                let event_id = zmq_msg.get_event_id();

                match event_id {
                    SocketEvent::CONNECTED => {
                        if first.load(Ordering::SeqCst) {
                            first.store(false, Ordering::SeqCst);
                            display_sender.send("ZMQ registering with the server.".to_string())?;
                        }
                        if let Err(e) = socket.send("register", DONTWAIT) {
                            display_sender.send(format!("error registering with ZMQ: {e:?}."))?;
                        }
                    }
                    SocketEvent::CONNECT_DELAYED | SocketEvent::CONNECT_RETRIED => {
                        continue;
                    }
                    SocketEvent::HANDSHAKE_SUCCEEDED => {
                        first.store(true, Ordering::SeqCst);
                        display_sender.send(format!("ZMQ connected to {}.", &args.host))?;
                    }
                    SocketEvent::HANDSHAKE_FAILED_AUTH
                    | SocketEvent::CLOSED
                    | SocketEvent::HANDSHAKE_FAILED_PROTOCOL
                    | SocketEvent::HANDSHAKE_FAILED_NO_DETAIL
                    | SocketEvent::MONITOR_STOPPED => break,
                    SocketEvent::DISCONNECTED => {
                        if first.load(Ordering::SeqCst) {
                            first.store(false, Ordering::SeqCst);
                            display_sender.send("Reconnecting ZMQ...".to_string())?;
                        }
                        if let Err(e) = socket.connect(&args.host) {
                            display_sender.send(format!("error reconnecting: {e:?}."))?;
                        }
                    }
                    socket_event => {
                        display_sender.send(format!(
                            "{:#06x?} {:?}",
                            socket_event.to_raw(),
                            socket_event
                        ))?;
                    }
                }
            }
            Err(zmq::Error::EAGAIN) => (),
            Err(e) => {
                display_sender.send(format!("zmq error: {:?}", e))?;
            }
        }

        loop {
            match zmq_receiver.try_recv() {
                Ok(line) => {
                    socket.send(&line, DONTWAIT)?;
                }
                Err(err) => {
                    if err.is_disconnected() {
                        display_sender.send("receiver disconnected".to_string())?;
                        CONTINUE_RUNNING.store(false, Ordering::SeqCst);
                    }
                    break;
                }
            }
        }

        if event == 0 {
            continue;
        }

        while let Ok(zmq_msg) = socket.recv_msg(DONTWAIT) {
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
