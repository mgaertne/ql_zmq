use anyhow::{Error, Result, anyhow};
use zmq::{SocketEvent, SocketType};

use crate::{ZmqSocket, sealed::ZmqSocketType};

#[derive(Debug, PartialEq)]
pub enum MonitorSocketEvent {
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

pub struct Monitor {}

unsafe impl Sync for ZmqSocket<Monitor> {}
unsafe impl Send for ZmqSocket<Monitor> {}

impl ZmqSocketType for Monitor {
    fn raw_socket_type() -> SocketType {
        zmq::PAIR
    }
}

impl ZmqSocket<Monitor> {
    pub async fn recv(&self) -> Result<MonitorSocketEvent> {
        self.socket
            .recv_multipart(zmq::DONTWAIT)
            .map_err(Error::from)
            .and_then(MonitorSocketEvent::try_from)
    }
}
