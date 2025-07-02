use core::{
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::{Error, Result, anyhow};
use futures::future::FutureExt;

use crate::{
    MonitorFlags, ZmqRecvFlags, ZmqSocket,
    sealed::{ZmqReceiver, ZmqSocketType},
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum HandshakeProtocolError {
    ZmtpUnspecified = 268435456,
    ZmtpUnexpectedCommand = 268435457,
    ZmtpInvalidSequence = 268435458,
    ZmtpKeyEchange = 268435459,
    ZmtpMalformedCommandUnspecified = 268435473,
    ZmtpMalformedCommandMessage = 268435474,
    ZmtpMalformedCommandHello = 268435475,
    ZmtpMalformedCommandInitiate = 268435476,
    ZmtpMalformedCommandError = 268435477,
    ZmtpMalformedCommandReady = 268435478,
    ZmtpMalformedCommandWelcome = 268435479,
    ZmtpInvalidMetadata = 268435480,
    ZmtpCryptographic = 285212673,
    ZmtpMechanismMismatch = 285212674,
    ZapUnspecified = 536870912,
    ZapMalformedReply = 536870913,
    ZapBadRequestId = 536870914,
    ZapBadVersion = 536870915,
    ZapInvalidStatusCode = 536870916,
    ZapInvalidMetadata = 536870917,
    UnsupportedError(u32),
}

impl From<u32> for HandshakeProtocolError {
    fn from(value: u32) -> Self {
        match value {
            268435456 => Self::ZmtpUnspecified,
            268435457 => Self::ZmtpUnexpectedCommand,
            268435458 => Self::ZmtpInvalidSequence,
            268435459 => Self::ZmtpKeyEchange,
            268435473 => Self::ZmtpMalformedCommandUnspecified,
            268435474 => Self::ZmtpMalformedCommandMessage,
            268435475 => Self::ZmtpMalformedCommandHello,
            268435476 => Self::ZmtpMalformedCommandInitiate,
            268435477 => Self::ZmtpMalformedCommandError,
            268435478 => Self::ZmtpMalformedCommandReady,
            268435479 => Self::ZmtpMalformedCommandWelcome,
            268435480 => Self::ZapInvalidMetadata,
            285212673 => Self::ZmtpCryptographic,
            285212674 => Self::ZmtpMechanismMismatch,
            536870912 => Self::ZapUnspecified,
            536870913 => Self::ZapMalformedReply,
            536870914 => Self::ZapBadRequestId,
            536870915 => Self::ZapBadVersion,
            536870916 => Self::ZapInvalidStatusCode,
            536870917 => Self::ZapInvalidMetadata,
            other => Self::UnsupportedError(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MonitorSocketEvent {
    Connected,
    ConnectDelayed,
    ConnectRetried(u32),
    Listening,
    Accepted,
    AcceptFailed(zmq::Error),
    Closed,
    CloseFailed(zmq::Error),
    Disconnected,
    MonitorStopped,
    HandshakeFailedNoDetail(zmq::Error),
    HandshakeSucceeded,
    HandshakeFailedProtocol(HandshakeProtocolError),
    HandshakeFailedAuth(u32),
    UnSupported(MonitorFlags, u32),
}

impl<T: Deref<Target = [u8]>> TryFrom<Vec<T>> for MonitorSocketEvent {
    type Error = Error;

    fn try_from(zmq_msgs: Vec<T>) -> Result<Self, Self::Error> {
        if zmq_msgs.len() != 2 {
            return Err(anyhow!("invalid msg received"));
        }

        let Some(first_msg) = zmq_msgs.first() else {
            return Err(anyhow!("invalid msg received"));
        };

        if first_msg.deref().len() != 6 {
            return Err(anyhow!("invalid msg received"));
        }

        let Some(event_id) = first_msg
            .deref()
            .first_chunk::<2>()
            .map(|raw_event_id| u16::from_le_bytes(*raw_event_id))
            .map(MonitorFlags::from)
        else {
            return Err(anyhow!("invalid first two bytes"));
        };

        let Some(event_value) = first_msg
            .deref()
            .last_chunk::<4>()
            .map(|raw_event_value| u32::from_le_bytes(*raw_event_value))
        else {
            return Err(anyhow!("invalid last four bytes"));
        };

        match event_id {
            MonitorFlags::Connected => Ok(Self::Connected),
            MonitorFlags::ConnectDelayed => Ok(Self::ConnectDelayed),
            MonitorFlags::ConnectRetried => Ok(Self::ConnectRetried(event_value)),
            MonitorFlags::Listening => Ok(Self::Listening),
            MonitorFlags::Accepted => Ok(Self::Accepted),
            MonitorFlags::AcceptFailed => {
                Ok(Self::AcceptFailed(zmq::Error::from_raw(event_value as i32)))
            }
            MonitorFlags::Closed => Ok(Self::Closed),
            MonitorFlags::CloseFailed => {
                Ok(Self::CloseFailed(zmq::Error::from_raw(event_value as i32)))
            }
            MonitorFlags::Disconnected => Ok(Self::Disconnected),
            MonitorFlags::MonitorStopped => Ok(Self::MonitorStopped),
            MonitorFlags::HandshakeFailedNoDetail => Ok(Self::HandshakeFailedNoDetail(
                zmq::Error::from_raw(event_value as i32),
            )),
            MonitorFlags::HandshakeSucceeded => Ok(Self::HandshakeSucceeded),
            MonitorFlags::HandshakeFailedProtocol => {
                Ok(Self::HandshakeFailedProtocol(event_value.into()))
            }
            MonitorFlags::HandshakeFailedAuth => Ok(Self::HandshakeFailedAuth(event_value)),
            event_id => Ok(Self::UnSupported(event_id, 0)),
        }
    }
}

pub struct Monitor {}

impl ZmqReceiver for Monitor {}

unsafe impl Sync for ZmqSocket<Monitor> {}
unsafe impl Send for ZmqSocket<Monitor> {}

impl ZmqSocketType for Monitor {
    fn raw_socket_type() -> zmq::SocketType {
        zmq::SocketType::PAIR
    }
}

impl ZmqSocket<Monitor> {
    pub async fn recv_monitor_event(&self) -> Option<MonitorSocketEvent> {
        MonitorSocketEventFuture { receiver: self }.now_or_never()
    }
}

struct MonitorSocketEventFuture<'a, T: ZmqSocketType + ZmqReceiver + Unpin> {
    receiver: &'a ZmqSocket<T>,
}

impl Future for MonitorSocketEventFuture<'_, Monitor> {
    type Output = MonitorSocketEvent;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self
            .receiver
            .socket
            .recv_multipart(ZmqRecvFlags::DONT_WAIT.bits())
            .map(MonitorSocketEvent::try_from)
        {
            Ok(Ok(event)) => Poll::Ready(event),
            _ => Poll::Pending,
        }
    }
}
