use core::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Error, Result};
use azmq::{Monitor, MonitorSocketEvent, Subscriber, ZmqSocket};
use serde_json::Value;
use tokio::{
    select,
    sync::{Mutex, mpsc::UnboundedSender},
};
use uuid::Uuid;
use zmq::Message;

use crate::{CONTINUE_RUNNING, cmd_line::CommandLineOptions};

struct MonitoredSubscriber {
    subscriber: Mutex<ZmqSocket<Subscriber>>,
    monitor: Mutex<ZmqSocket<Monitor>>,
}

unsafe impl Send for MonitoredSubscriber {}
unsafe impl Sync for MonitoredSubscriber {}

impl MonitoredSubscriber {
    fn new() -> Result<Self> {
        let subscriber = ZmqSocket::try_new()?;
        let monitor = subscriber.monitor(zmq::SocketEvent::ALL)?;

        Ok(Self {
            subscriber: subscriber.into(),
            monitor: monitor.into(),
        })
    }

    async fn configure(&self, password: &str, identity: &str) -> Result<()> {
        let subscriber = self.subscriber.lock().await;
        subscriber.set_plain_username(Some("stats"))?;
        if !password.is_empty() {
            subscriber.set_plain_password(Some(password))?;
        } else {
            subscriber.set_plain_password(None::<&str>)?;
        }

        let identity_str = if identity.is_empty() {
            let identity = Uuid::new_v4();
            identity.to_string().replace("-", "")
        } else {
            identity.to_string()
        };

        subscriber.set_identity(identity_str.as_bytes())?;

        subscriber.set_rcvtimeo(0)?;
        subscriber.set_rcvhwm(0)?;
        subscriber.set_sndtimeo(0)?;
        subscriber.set_sndhwm(0)?;

        subscriber.set_heartbeat_ivl(600_000)?;
        subscriber.set_heartbeat_timeout(600_000)?;

        subscriber.set_zap_domain("stats")?;

        Ok(())
    }

    async fn connect(&self, address: &str) -> Result<()> {
        let socket = self.subscriber.lock().await;
        socket.as_ref().connect(address)?;

        socket.as_ref().set_subscribe("".as_bytes())?;

        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        let subscriber = self.subscriber.lock().await;
        subscriber
            .last_endpoint()?
            .map_err(|_err| Error::from(zmq::Error::EFAULT))
            .and_then(|last_endpoint| subscriber.disconnect(&last_endpoint))?;

        Ok(())
    }

    async fn recv_msg(&self) -> Option<Message> {
        let subscriber = self.subscriber.lock().await;
        subscriber.recv_async().await.ok()
    }

    async fn check_monitor(&self) -> Result<MonitorSocketEvent> {
        let monitor = self.monitor.lock().await;
        monitor.recv().await
    }
}

fn format_ql_json(msg: &str, args: &CommandLineOptions) -> String {
    serde_json::from_str::<Value>(msg)
        .and_then(|parsed_json| {
            if args.pretty_print {
                serde_json::to_string_pretty(&parsed_json)
            } else {
                serde_json::to_string(&parsed_json)
            }
        })
        .unwrap_or(msg.to_string())
}

static FIRST_TIME: AtomicBool = AtomicBool::new(true);

async fn check_monitor(
    monitored_dealer: &MonitoredSubscriber,
    sender: &UnboundedSender<String>,
    endpoint: &str,
) -> Result<()> {
    match monitored_dealer.check_monitor().await {
        Ok(MonitorSocketEvent::HandshakeSucceeded) => {
            FIRST_TIME.store(true, Ordering::Release);
            sender.send(format!("ZMQ connected to {}.", &endpoint))?;
        }

        Ok(
            event @ (MonitorSocketEvent::HandshakeFailedAuth(_)
            | MonitorSocketEvent::HandshakeFailedProtocol(_)
            | MonitorSocketEvent::HandshakeFailedNoDetail(_)
            | MonitorSocketEvent::MonitorStopped),
        ) => {
            sender.send(format!("ZMQ socket error: {event:?}"))?;
            CONTINUE_RUNNING.store(false, Ordering::Release);
        }

        Ok(MonitorSocketEvent::Disconnected | MonitorSocketEvent::Closed) => {
            if FIRST_TIME.load(Ordering::Acquire) {
                FIRST_TIME.store(false, Ordering::Release);
                sender.send("Reconnecting ZMQ...".to_string())?;
            }
            if let Err(e) = monitored_dealer.connect(endpoint).await {
                sender.send(format!("error reconnecting: {e:?}."))?;
            }
        }

        Ok(
            MonitorSocketEvent::ConnectDelayed
            | MonitorSocketEvent::ConnectRetried(_)
            | MonitorSocketEvent::Connected,
        ) => (),

        Ok(event) => {
            sender.send(format!("ZMQ socket error: {event:?}",))?;
        }

        Err(err)
            if err
                .downcast_ref::<zmq::Error>()
                .is_some_and(|&zmq_error| zmq_error == zmq::Error::EAGAIN) => {}

        Err(err) => {
            sender.send(format!("ZMQ error: {err}"))?;
        }
    };

    Ok(())
}

pub(crate) async fn run_zmq(
    args: CommandLineOptions,
    display_sender: UnboundedSender<String>,
) -> Result<()> {
    display_sender.send(format!("ZMQ connecting to {}...", &args.host))?;

    let monitored_dealer = MonitoredSubscriber::new()?;
    monitored_dealer
        .configure(&args.password, &args.identity)
        .await?;

    monitored_dealer.connect(&args.host).await?;

    while CONTINUE_RUNNING.load(Ordering::Acquire) {
        select!(
            biased;

            Some(zmq_msg) = monitored_dealer.recv_msg() => {
                if let Some(zmq_str) = zmq_msg.as_str() {
                    display_sender.send(format_ql_json(zmq_str, &args))?;
                }
            }

            Ok(()) = check_monitor(&monitored_dealer, &display_sender, &args.host) => (),

            else => ()
        );
    }

    monitored_dealer.disconnect().await?;

    drop(display_sender);

    Ok(())
}
