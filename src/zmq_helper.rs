use core::sync::atomic::Ordering;

use anyhow::Result;
use rzmq::{
    Context, Msg, Socket, SocketType, ZmqError,
    socket::{PLAIN_PASSWORD, PLAIN_USERNAME, RCVTIMEO, ROUTING_ID, SocketEvent, ZAP_DOMAIN},
};
use rzmq::socket::SNDTIMEO;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, error::TryRecvError};
use uuid::Uuid;

use crate::{CONTINUE_RUNNING, cmd_line::CommandLineOptions};

pub(crate) async fn create_socket(
    context: &Context,
    password: &str,
    identity: &str,
) -> Result<Socket, rzmq::ZmqError> {
    let socket = context.socket(SocketType::Dealer)?;

    socket.set_option(PLAIN_USERNAME, "rcon").await?;
    if !password.is_empty() {
        socket.set_option(PLAIN_PASSWORD, password).await?;
    } else {
        socket.set_option(PLAIN_PASSWORD, "").await?;
    }

    let identity_str = if identity.is_empty() {
        let identity = Uuid::new_v4();
        identity.to_string().replace("-", "")
    } else {
        identity.to_string()
    };

    socket
        .set_option(ROUTING_ID, identity_str.as_bytes())
        .await?;

    socket.set_option(ZAP_DOMAIN, "rcon").await?;

    socket.set_option(RCVTIMEO, 100).await?;
    socket.set_option(SNDTIMEO, 100).await?;

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
    let socket = create_socket(&zmq_context, &args.password, &args.identity).await?;

    let monitor_socket = socket.monitor_default().await?;

    socket.connect(&args.host).await?;

    'outer: loop {
        if !CONTINUE_RUNNING.load(Ordering::SeqCst) {
            break 'outer;
        }

        match socket.recv().await {
            Ok(event) => {
                match monitor_socket.recv().await {
                    Ok(SocketEvent::Connected { .. }) => {
                        display_sender.send("ZMQ registering with the server.".to_string())?;
                        if let Err(e) = socket.send(Msg::from_static("register".as_bytes())).await {
                            display_sender.send(format!("error registering with ZMQ: {e:?}."))?;
                        }
                    }
                    Ok(SocketEvent::ConnectDelayed { .. })
                    | Ok(SocketEvent::ConnectRetried { .. }) => {
                        continue 'outer;
                    }
                    Ok(SocketEvent::HandshakeSucceeded { .. }) => {
                        display_sender.send(format!("ZMQ connected to {}.", &args.host))?;
                    }
                    Ok(SocketEvent::HandshakeFailed { .. }) | Ok(SocketEvent::Closed { .. }) => {
                        break 'outer;
                    }
                    Ok(SocketEvent::Disconnected { endpoint }) => {
                        if let Err(e) = socket.connect(&endpoint).await {
                            display_sender.send(format!("error reconnecting: {e:?}."))?;
                        }
                    }
                    Ok(socket_event) => {
                        display_sender.send(format!("{socket_event:?}",))?;
                    }
                    Err(e) => {
                        display_sender.send(format!("zmq error: {e:?}"))?;
                    }
                }

                'inner: loop {
                    match zmq_receiver.try_recv() {
                        Ok(line) => {
                            socket.send(Msg::from_vec(line.into_bytes())).await?;
                        }
                        Err(TryRecvError::Disconnected) => {
                            display_sender.send("receiver disconnected".to_string())?;
                            CONTINUE_RUNNING.store(false, Ordering::SeqCst);
                            break 'inner;
                        }
                        _ => {
                            break 'inner;
                        }
                    }
                }

                if event.data().is_none() {
                    continue 'outer;
                }

                while let Ok(zmq_msg) = socket.recv().await {
                    if let Some(zmq_data) = zmq_msg.data() {
                        let zmq_str = String::from_utf8_lossy(zmq_data);
                        display_sender.send(trim_ql_msg(&zmq_str))?;
                    }
                }
            }
            Err(ZmqError::Timeout) => {
            },
            Err(e) => {
                display_sender.send(format!("ZmqError: {e:?}"))?;
            }
        }
    }

    drop(zmq_receiver);

    if CONTINUE_RUNNING.load(Ordering::SeqCst) {
        display_sender.send("Exiting ZMQ...".to_string())?;
    }

    monitor_socket.close()?;
    socket.close().await?;

    drop(display_sender);

    CONTINUE_RUNNING.store(false, Ordering::SeqCst);

    Ok(())
}
