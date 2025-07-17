use core::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use arzmq::{
    builder::ContextBuilder,
    futures::{AsyncMonitorReceiver, AsyncReceiver, AsyncSender},
    message::Message,
    security::SecurityMechanism,
    socket::{DealerSocket, MonitorFlags, MonitorSocket, MonitorSocketEvent, SendFlags, Socket},
};
use tokio::{
    select,
    sync::{
        RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
};
use uuid::Uuid;

use crate::{CONTINUE_RUNNING, cmd_line::CommandLineOptions};

struct MonitoredDealer {
    dealer: RwLock<DealerSocket>,
    monitor: RwLock<MonitorSocket>,
}

unsafe impl Send for MonitoredDealer {}
unsafe impl Sync for MonitoredDealer {}

impl MonitoredDealer {
    fn new() -> Result<Self> {
        let context = ContextBuilder::new()
            .blocky(false)
            .max_sockets(10)
            .io_threads(2)
            .build()?;
        let dealer = Socket::from_context(&context)?;
        let monitor = dealer.monitor(
            MonitorFlags::Connected
                | MonitorFlags::HandshakeSucceeded
                | MonitorFlags::HandshakeFailedAuth
                | MonitorFlags::HandshakeFailedProtocol
                | MonitorFlags::HandshakeFailedNoDetail
                | MonitorFlags::MonitorStopped
                | MonitorFlags::Disconnected
                | MonitorFlags::Closed,
        )?;

        Ok(Self {
            dealer: dealer.into(),
            monitor: monitor.into(),
        })
    }

    async fn configure(&self, password: &str, identity: &str) -> Result<()> {
        let dealer = self.dealer.read().await;

        dealer.set_security_mechanism(SecurityMechanism::PlainClient {
            username: "rcon".into(),
            password: password.into(),
        })?;

        let identity_str = if identity.is_empty() {
            let identity = Uuid::new_v4();
            identity.to_string().replace("-", "")
        } else {
            identity.to_string()
        };
        dealer.set_routing_id(identity_str)?;

        dealer.set_hello_message("register")?;
        dealer.set_immediate(true)?;

        dealer.set_receive_timeout(0)?;
        dealer.set_receive_highwater_mark(0)?;
        dealer.set_send_timeout(0)?;
        dealer.set_send_highwater_mark(0)?;

        dealer.set_heartbeat_ivl(600_000)?;
        dealer.set_heartbeat_timeout(600_000)?;

        dealer.set_zap_domain("rcon".into())?;

        Ok(())
    }

    async fn connect(&self, address: &str) -> Result<()> {
        self.dealer.read().await.connect(address)?;

        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        let dealer = self.dealer.read().await;
        let last_endpoint = dealer.last_endpoint()?;
        dealer.disconnect(&last_endpoint)?;

        Ok(())
    }

    async fn send<M: Into<Message>, F: Into<SendFlags>>(&self, msg: M, flags: F) -> Option<()> {
        self.dealer
            .read()
            .await
            .send_msg_async(msg.into(), flags.into())
            .await?;

        Some(())
    }

    async fn recv_msg(&self) -> Option<Message> {
        let dealer = self.dealer.read().await;
        dealer.recv_msg_async().await
    }

    async fn check_monitor(&self) -> Option<MonitorSocketEvent> {
        let monitor = self.monitor.read().await;
        monitor.recv_monitor_event_async().await
    }
}

fn trim_ql_msg(msg: &str) -> String {
    msg.replace("\n", "")
        .replace("\\n", "")
        .replace('\u{0019}', "")
}

static FIRST_TIME: AtomicBool = AtomicBool::new(true);

async fn check_monitor(
    monitored_dealer: &MonitoredDealer,
    sender: &UnboundedSender<String>,
    endpoint: &str,
) -> Result<()> {
    match monitored_dealer.check_monitor().await {
        Some(MonitorSocketEvent::Connected) => {
            if FIRST_TIME.load(Ordering::Acquire) {
                FIRST_TIME.store(false, Ordering::Release);
                sender.send("ZMQ registering with the server.".to_string())?;
            }
        }

        Some(MonitorSocketEvent::HandshakeSucceeded) => {
            FIRST_TIME.store(true, Ordering::Release);
            sender.send(format!("ZMQ connected to {}.", &endpoint))?;
        }

        Some(
            event @ (MonitorSocketEvent::HandshakeFailedAuth(_)
            | MonitorSocketEvent::HandshakeFailedProtocol(_)
            | MonitorSocketEvent::HandshakeFailedNoDetail(_)
            | MonitorSocketEvent::MonitorStopped),
        ) => {
            sender.send(format!("ZMQ socket error: {event:?}"))?;
            CONTINUE_RUNNING.store(false, Ordering::Release);
        }

        Some(MonitorSocketEvent::Disconnected | MonitorSocketEvent::Closed) => {
            if FIRST_TIME.load(Ordering::Acquire) {
                FIRST_TIME.store(false, Ordering::Release);
                sender.send("Reconnecting ZMQ...".to_string())?;
            }
            if let Err(e) = monitored_dealer.connect(endpoint).await {
                sender.send(format!("error reconnecting: {e:?}."))?;
            }
        }

        Some(event) => {
            sender.send(format!("ZMQ socket error: {event:?}",))?;
        }

        _ => (),
    };

    Ok(())
}

pub(crate) async fn run_zmq(
    args: CommandLineOptions,
    mut zmq_receiver: UnboundedReceiver<String>,
    display_sender: UnboundedSender<String>,
) -> Result<()> {
    display_sender.send(format!("ZMQ connecting to {}...", &args.host))?;

    let monitored_dealer = MonitoredDealer::new()?;
    monitored_dealer
        .configure(&args.password, &args.identity)
        .await?;

    monitored_dealer.connect(&args.host).await?;

    while CONTINUE_RUNNING.load(Ordering::Acquire) && !zmq_receiver.is_closed() {
        select!(
            biased;

            Some(zmq_msg) = monitored_dealer.recv_msg() => {
                let zmq_str = zmq_msg.to_string();
                display_sender.send(trim_ql_msg(&zmq_str))?;
            }

            Some(line) = zmq_receiver.recv(), if !zmq_receiver.is_empty() => {
                monitored_dealer.send(&line, SendFlags::DONT_WAIT).await;
            },

            Ok(()) = check_monitor(&monitored_dealer, &display_sender, &args.host) => (),

            else => ()
        );
    }

    drop(zmq_receiver);

    monitored_dealer.disconnect().await?;

    drop(display_sender);

    Ok(())
}
