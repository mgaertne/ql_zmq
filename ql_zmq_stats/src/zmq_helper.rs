use core::{
    future::{Future, poll_fn},
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

use anyhow::{Error, Result, anyhow};
use derive_more::AsRef;
use futures::FutureExt;
use serde_json::Value;
use tap::TryConv;
use tokio::{
    select,
    sync::{Mutex, mpsc::UnboundedSender},
};
use uuid::Uuid;
use zmq::{Context, Message, Socket, SocketEvent, SocketType};

use crate::{CONTINUE_RUNNING, cmd_line::CommandLineOptions};

#[derive(Debug, PartialEq)]
enum MonitorSocketEvent {
    Connected,
    ConnectDelayed,
    ConnectRetried(u32),
    Listening,
    Accepted,
    AcceptFailed(zmq::Error),
    Closed,
    CloseFailed(u32),
    Disconnected,
    MonitorStopped,
    HandshakeFailedNoDetail(u32),
    HandshakeSucceeded,
    HandshakeFailedProtocol(u32),
    HandshakeFailedAuth(u32),
    UnSupported(SocketEvent, u32),
}

impl TryFrom<Vec<Vec<u8>>> for MonitorSocketEvent {
    type Error = Error;

    fn try_from(raw_multipart: Vec<Vec<u8>>) -> Result<Self, Self::Error> {
        if raw_multipart.len() != 2 {
            return Err(anyhow!("invalid msg received"));
        }

        let Some(first_msg) = raw_multipart.first() else {
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

        match event_id {
            SocketEvent::CONNECTED => Ok(Self::Connected),
            SocketEvent::CONNECT_DELAYED => Ok(Self::ConnectDelayed),
            SocketEvent::CONNECT_RETRIED => Ok(Self::ConnectRetried(event_value)),
            SocketEvent::LISTENING => Ok(Self::Listening),
            SocketEvent::ACCEPTED => Ok(Self::Accepted),
            SocketEvent::ACCEPT_FAILED => {
                Ok(Self::AcceptFailed(zmq::Error::from_raw(event_value as i32)))
            }
            SocketEvent::CLOSED => Ok(Self::Closed),
            SocketEvent::CLOSE_FAILED => Ok(Self::CloseFailed(event_value)),
            SocketEvent::DISCONNECTED => Ok(Self::Disconnected),
            SocketEvent::MONITOR_STOPPED => Ok(Self::MonitorStopped),
            SocketEvent::HANDSHAKE_FAILED_NO_DETAIL => {
                Ok(Self::HandshakeFailedNoDetail(event_value))
            }
            SocketEvent::HANDSHAKE_SUCCEEDED => Ok(Self::HandshakeSucceeded),
            SocketEvent::HANDSHAKE_FAILED_PROTOCOL => {
                Ok(Self::HandshakeFailedProtocol(event_value))
            }
            SocketEvent::HANDSHAKE_FAILED_AUTH => Ok(Self::HandshakeFailedAuth(event_value)),
            event_id => Ok(Self::UnSupported(event_id, 0)),
        }
    }
}

#[derive(AsRef)]
#[as_ref(Socket)]
struct SubscriberSocket(Socket);

unsafe impl Send for SubscriberSocket {}
unsafe impl Sync for SubscriberSocket {}

impl TryFrom<&Context> for SubscriberSocket {
    type Error = Error;

    fn try_from(context: &Context) -> Result<Self> {
        let socket = context.socket(SocketType::SUB)?;

        Ok(Self(socket))
    }
}

impl Future for SubscriberSocket {
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

impl SubscriberSocket {
    fn configure(&self, password: &str, identity: &str) -> Result<()> {
        self.as_ref().set_plain_username(Some("stats"))?;
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

        self.as_ref().set_zap_domain("stats")?;

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

        monitor.connect(&monitor_endpoint)?;

        Ok(MonitoringSocket(monitor))
    }
}

#[derive(AsRef)]
#[as_ref(Socket)]
struct MonitoringSocket(Socket);

unsafe impl Send for MonitoringSocket {}
unsafe impl Sync for MonitoringSocket {}

impl TryFrom<(&Context, &SubscriberSocket)> for MonitoringSocket {
    type Error = Error;

    fn try_from((ctx, socket): (&Context, &SubscriberSocket)) -> Result<Self> {
        socket.monitor(ctx, SocketEvent::ALL)
    }
}

impl Future for MonitoringSocket {
    type Output = Result<MonitorSocketEvent>;

    fn poll(self: Pin<&mut Self>, _ctx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(
            (*self)
                .as_ref()
                .recv_multipart(zmq::DONTWAIT)
                .map_err(Error::from)
                .and_then(MonitorSocketEvent::try_from),
        )
    }
}

struct MonitoredSubscriber {
    subscriber: Mutex<SubscriberSocket>,
    monitor: Mutex<MonitoringSocket>,
}

unsafe impl Send for MonitoredSubscriber {}
unsafe impl Sync for MonitoredSubscriber {}

impl MonitoredSubscriber {
    fn new() -> Result<Self> {
        let context = Context::new();
        let socket = (&context).try_conv::<SubscriberSocket>()?;
        let monitor = (&context, &socket).try_conv::<MonitoringSocket>()?.into();

        Ok(Self {
            subscriber: socket.into(),
            monitor,
        })
    }

    async fn configure(&self, password: &str, identity: &str) -> Result<()> {
        self.subscriber.lock().await.configure(password, identity)
    }

    async fn connect(&self, address: &str) -> Result<()> {
        let socket = self.subscriber.lock().await;
        socket.as_ref().connect(address)?;

        socket.as_ref().set_subscribe("".as_bytes())?;

        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        self.subscriber.lock().await.disconnect()?;

        Ok(())
    }

    async fn recv_msg(&self) -> Result<Message> {
        let mut socket = self.subscriber.lock().await;
        poll_fn(|ctx| socket.poll_unpin(ctx)).await
    }

    async fn check_monitor(&self) -> Result<MonitorSocketEvent> {
        let mut monitor = self.monitor.lock().await;
        poll_fn(|ctx| (*monitor).poll_unpin(ctx)).await
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

            Ok(zmq_msg) = monitored_dealer.recv_msg() => {
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
