#[cfg(feature = "draft-api")]
use bitflags::bitflags;

use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOptions, SocketType},
};

pub struct Router {}

impl sealed::SenderFlag for Router {}
impl sealed::ReceiverFlag for Router {}

impl sealed::SocketType for Router {
    fn raw_socket_type() -> SocketType {
        SocketType::Router
    }
}

unsafe impl Sync for Socket<Router> {}
unsafe impl Send for Socket<Router> {}

#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub struct RouterNotify(i32);

#[cfg(feature = "draft-api")]
bitflags! {
    impl RouterNotify: i32 {
        const NotifyConnect    = 0b0000_0000_0000_0001;
        const NotifyDisconnect = 0b0000_0000_0000_0010;
    }
}

/// # A router socket `ZMQ_ROUTER`
///
/// A socket of type [`Router`] is an advanced socket type used for extending request/reply
/// sockets. When receiving messages a [`Router`] socket shall prepend a message part containing
/// the routing id of the originating peer to the message before passing it to the application.
/// Messages received are fair-queued from among all connected peers. When sending messages a
/// [`Router`] socket shall remove the first part of the message and use it to determine the
/// [`routing_id()`](method@super::Socket::routing_id()) of the peer the message shall be
/// routed to. If the peer does not exist anymore, or has never existed, the message shall be
/// silently discarded. However, if
/// [`RouterMandatory`](variant@SocketOptions::RouterMandatory) socket option is set to
/// `true`, the socket shall fail with
/// `Err(`[`ZmqError::HostUnreachable`](crate::ZmqError::HostUnreachable)`)` in both cases.
///
/// When a [`Router`] socket enters the 'mute' state due to having reached the high water mark for
/// all peers, then any messages sent to the socket shall be dropped until the mute state ends.
/// Likewise, any messages routed to a peer for which the individual high water mark has been
/// reached shall also be dropped. If,
/// [`RouterMandatory`](variant@SocketOptions::RouterMandatory) is set to `true`, the socket
/// shall block or return `Err(`[`ZmqError::Again`](crate::ZmqError::Again)`)` in both cases.
///
/// When a [`Router`] socket has [`RouterMandatory`](variant@SocketOptions::RouterMandatory)
/// flag set to `true`, the socket shall generate
/// [`ZmqPollEvents::ZMQ_POLLIN`](const@super::PollEvents::ZMQ_POLLIN) events upon reception
/// of messages from one or more peers. Likewise, the socket shall generate
/// [`ZmqPollEvents::ZMQ_POLLOUT`](const@super::PollEvents::ZMQ_POLLOUT) events when at least
/// one message can be sent to one or more peers.
///
/// When a [`Request`](super::Request) socket is connected to a [`Router`] socket, in addition to
/// the routing id of the originating peer each message received shall contain an empty delimiter
/// message part. Hence, the entire structure of each received message as seen by the application
/// becomes: one or more routing id parts, delimiter part, one or more body parts. When sending
/// replies to a [`Request`](super::Request) socket the application must include the delimiter
/// part.
impl Socket<Router> {
    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::RoutingId as i32)
    }

    pub fn set_connect_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::ConnectRoutingId as i32, value)
    }

    pub fn set_router_handover(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::RouterHandover as i32, value)
    }

    pub fn set_router_mandatory(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::RouterMandatory as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_disconnect_message<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::DisconnectMessage as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_hello_message<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::HelloMessage as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_router_notify(&self, value: RouterNotify) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::RouterNotify as i32, value.bits())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn router_notify(&self) -> ZmqResult<RouterNotify> {
        self.get_sockopt_int(SocketOptions::RouterNotify as i32)
            .map(RouterNotify::from_bits_truncate)
    }
}
