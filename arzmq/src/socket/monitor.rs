#[cfg(feature = "futures")]
use core::{pin::Pin, task::Context, task::Poll};

#[cfg(feature = "futures")]
use async_trait::async_trait;
#[cfg(feature = "futures")]
use futures::FutureExt;

use super::{MonitorFlags, MultipartReceiver, RecvFlags, SocketType};
use crate::{
    ZmqError, ZmqResult, message::MultipartMessage, sealed, socket::Socket, zmq_sys_crate,
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum HandshakeProtocolError {
    ZmtpUnspecified = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED,
    ZmtpUnexpectedCommand = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
    ZmtpInvalidSequence = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE,
    ZmtpKeyEchange = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE,
    ZmtpMalformedCommandUnspecified =
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED,
    ZmtpMalformedCommandMessage = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE,
    ZmtpMalformedCommandHello = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO,
    ZmtpMalformedCommandInitiate =
        zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE,
    ZmtpMalformedCommandError = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR,
    ZmtpMalformedCommandReady = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY,
    ZmtpMalformedCommandWelcome = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME,
    ZmtpInvalidMetadata = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA,
    ZmtpCryptographic = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC,
    ZmtpMechanismMismatch = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH,
    ZapUnspecified = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED,
    ZapMalformedReply = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY,
    ZapBadRequestId = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID,
    ZapBadVersion = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION,
    ZapInvalidStatusCode = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE,
    ZapInvalidMetadata = zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA,
    UnsupportedError(u32),
}

impl From<u32> for HandshakeProtocolError {
    fn from(value: u32) -> Self {
        match value {
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED => Self::ZmtpUnspecified,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND => {
                Self::ZmtpUnexpectedCommand
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE => Self::ZmtpInvalidSequence,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE => Self::ZmtpKeyEchange,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED => {
                Self::ZmtpMalformedCommandUnspecified
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE => {
                Self::ZmtpMalformedCommandMessage
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO => {
                Self::ZmtpMalformedCommandHello
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE => {
                Self::ZmtpMalformedCommandInitiate
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR => {
                Self::ZmtpMalformedCommandError
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY => {
                Self::ZmtpMalformedCommandReady
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME => {
                Self::ZmtpMalformedCommandWelcome
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA => Self::ZapInvalidMetadata,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC => Self::ZmtpCryptographic,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH => {
                Self::ZmtpMechanismMismatch
            }
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED => Self::ZapUnspecified,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY => Self::ZapMalformedReply,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID => Self::ZapBadRequestId,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION => Self::ZapBadVersion,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE => Self::ZapInvalidStatusCode,
            zmq_sys_crate::ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA => Self::ZapInvalidMetadata,
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
    AcceptFailed(ZmqError),
    Closed,
    CloseFailed(ZmqError),
    Disconnected,
    MonitorStopped,
    HandshakeFailedNoDetail(ZmqError),
    HandshakeSucceeded,
    HandshakeFailedProtocol(HandshakeProtocolError),
    HandshakeFailedAuth(u32),
    UnSupported(MonitorFlags, u32),
}

impl TryFrom<MultipartMessage> for MonitorSocketEvent {
    type Error = ZmqError;

    fn try_from(zmq_msgs: MultipartMessage) -> Result<Self, Self::Error> {
        if zmq_msgs.len() != 2 {
            return Err(ZmqError::InvalidArgument);
        }

        let Some(first_msg) = zmq_msgs.get(0) else {
            return Err(ZmqError::InvalidArgument);
        };

        if first_msg.len() != 6 {
            return Err(ZmqError::InvalidArgument);
        }

        let Some(event_id) = first_msg
            .bytes()
            .first_chunk::<2>()
            .map(|raw_event_id| u16::from_le_bytes(*raw_event_id))
            .map(MonitorFlags::from)
        else {
            return Err(ZmqError::InvalidArgument);
        };

        let Some(event_value) = first_msg
            .bytes()
            .last_chunk::<4>()
            .map(|raw_event_value| u32::from_le_bytes(*raw_event_value))
        else {
            return Err(ZmqError::InvalidArgument);
        };

        match event_id {
            MonitorFlags::Connected => Ok(Self::Connected),
            MonitorFlags::ConnectDelayed => Ok(Self::ConnectDelayed),
            MonitorFlags::ConnectRetried => Ok(Self::ConnectRetried(event_value)),
            MonitorFlags::Listening => Ok(Self::Listening),
            MonitorFlags::Accepted => Ok(Self::Accepted),
            MonitorFlags::AcceptFailed => {
                Ok(Self::AcceptFailed(ZmqError::from(event_value as i32)))
            }
            MonitorFlags::Closed => Ok(Self::Closed),
            MonitorFlags::CloseFailed => Ok(Self::CloseFailed(ZmqError::from(event_value as i32))),
            MonitorFlags::Disconnected => Ok(Self::Disconnected),
            MonitorFlags::MonitorStopped => Ok(Self::MonitorStopped),
            MonitorFlags::HandshakeFailedNoDetail => Ok(Self::HandshakeFailedNoDetail(
                ZmqError::from(event_value as i32),
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

/// # A monitor socket `ZMQ_PAIR`
pub type MonitorSocket = Socket<Monitor>;

pub struct Monitor {}

impl sealed::ReceiverFlag for Monitor {}

unsafe impl Sync for Socket<Monitor> {}
unsafe impl Send for Socket<Monitor> {}

impl MultipartReceiver for Socket<Monitor> {}

impl sealed::SocketType for Monitor {
    fn raw_socket_type() -> SocketType {
        SocketType::Pair
    }
}

impl Socket<Monitor> {}

#[cfg_attr(feature = "futures", async_trait)]
pub trait MonitorReceiver {
    fn recv_monitor_event(&self) -> ZmqResult<MonitorSocketEvent>;

    #[cfg(feature = "futures")]
    #[doc(cfg(feature = "futures"))]
    async fn recv_monitor_event_async(&self) -> Option<MonitorSocketEvent>;
}

#[cfg_attr(feature = "futures", async_trait)]
impl MonitorReceiver for MonitorSocket {
    fn recv_monitor_event(&self) -> ZmqResult<MonitorSocketEvent> {
        self.recv_multipart(RecvFlags::DONT_WAIT)
            .and_then(MonitorSocketEvent::try_from)
    }

    #[cfg(feature = "futures")]
    async fn recv_monitor_event_async(&self) -> Option<MonitorSocketEvent> {
        MonitorSocketEventFuture { receiver: self }.now_or_never()
    }
}

#[cfg(feature = "futures")]
struct MonitorSocketEventFuture<'a> {
    receiver: &'a MonitorSocket,
}

#[cfg(feature = "futures")]
impl Future for MonitorSocketEventFuture<'_> {
    type Output = MonitorSocketEvent;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.receiver.recv_monitor_event() {
            Ok(event) => Poll::Ready(event),
            _ => Poll::Pending,
        }
    }
}
