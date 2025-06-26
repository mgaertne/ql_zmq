use core::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use rzmq::{
    Context, Socket, SocketType,
    socket::{PLAIN_PASSWORD, PLAIN_USERNAME, RCVTIMEO, ROUTING_ID, SocketEvent, ZAP_DOMAIN},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, error::TryRecvError};
use uuid::Uuid;

use crate::{CONTINUE_RUNNING, cmd_line::CommandLineOptions};

pub(crate) fn create_socket(
    context: &Context,
    password: &str,
    identity: &str,
) -> Result<Socket, rzmq::ZmqError> {
    let socket = context.socket(SocketType::Dealer)?;

    socket.set_option(PLAIN_USERNAME, Some("rcon"))?;
    if !password.is_empty() {
        socket.set_option(PLAIN_PASSWORD, Some(password))?;
    } else {
        socket.set_option(PLAIN_PASSWORD, None)?;
    }

    let identity_str = if identity.is_empty() {
        let identity = Uuid::new_v4();
        identity.to_string().replace("-", "")
    } else {
        identity.to_string()
    };

    socket.set_option(ROUTING_ID, identity_str.as_bytes())?;

    socket.set_option(ZAP_DOMAIN, "rcon")?;

    socket.set_option(RCVTIMEO, 100)?;

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

    let zmq_context = Context::new()?;
    let socket = create_socket(&zmq_context, &args.password, &args.identity)?;

    let monitor_socket = socket.monitor_default()?;

    socket.connect(&args.host)?;

    let first = AtomicBool::new(true);

    while let Ok(event) = socket.recv().await {
        if !CONTINUE_RUNNING.load(Ordering::SeqCst) {
            break;
        }

        match monitor_socket.recv_multipart() {
            Ok(zmq_msg) => {
                let event_id = zmq_msg.get_event_id();

                match event_id {
                    SocketEvent::ConnectDelayed { .. } => {
                        if first.load(Ordering::SeqCst) {
                            first.store(false, Ordering::SeqCst);
                            display_sender.send("ZMQ registering with the server.".to_string())?;
                        }
                        if let Err(e) = socket.send("register").await {
                            display_sender.send(format!("error registering with ZMQ: {e:?}."))?;
                        }
                    }
                    SocketEvent::ConnectDelayed { .. } | SocketEvent::ConnectRetried { .. } => {
                        continue;
                    }
                    SocketEvent::HandshakeSucceeded { .. } => {
                        first.store(true, Ordering::SeqCst);
                        display_sender.send(format!("ZMQ connected to {}.", &args.host))?;
                    }
                    SocketEvent::HandshakeFailed { .. } | SocketEvent::Closed { .. } => break,
                    SocketEvent::Disconnected { .. } => {
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
            Err(e) => {
                display_sender.send(format!("zmq error: {e:?}"))?;
            }
        }

        loop {
            match zmq_receiver.try_recv() {
                Ok(line) => {
                    socket.send(&line).await?;
                }
                Err(TryRecvError::Disconnected) => {
                    display_sender.send("receiver disconnected".to_string())?;
                    CONTINUE_RUNNING.store(false, Ordering::SeqCst);
                    break;
                }
                _ => {
                    break;
                }
            }
        }

        if event == 0 {
            continue;
        }

        while let Ok(zmq_msg) = socket.recv().await {
            if let Some(zmq_data) = zmq_msg.data() {
                let zmq_str = String::from(zmq_data);
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
