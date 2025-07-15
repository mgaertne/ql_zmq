use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOptions, SocketType},
};

/// # A server socket `ZMQ_SERVER`
///
/// A [`Server`] socket talks to a set of [`Client`] sockets. A [`Server`] socket can only reply to
/// an incoming message: the [`Client`] peer must always initiate a conversation.
///
/// Each received message has a [`routing_id()`] that is a 32-bit unsigned integer. The application
/// can fetch this with [`routing_id()`] To send a message to a given [`Client`] peer the
/// application must set the peer’s [`routing_id()`] on the message, using [`set_routing_id()`].
///
/// If the [`routing_id()`] is not specified, or does not refer to a connected client peer, the
/// send call will fail with [`HostUnreachable`]. If the outgoing buffer for the client peer is
/// full, the send call shall block, unless [`DONT_WAIT`] is used in the send, in which case it
/// shall fail with [`Again`]]. The [`Server`] socket shall not drop messages in any case.
///
/// [`Client`]: super::ClientSocket
/// [`Server`]: ServerSocket
/// [`routing_id()`]: crate::message::Message::routing_id
/// [`set_routing_id()`]: crate::message::Message::set_routing_id
/// [`HostUnreachable`]: crate::ZmqError::HostUnreachable
/// [`Again`]: crate::ZmqError::Again
/// [`DONT_WAIT`]: super::SendFlags::DONT_WAIT
pub type ServerSocket = Socket<Server>;

pub struct Server {}

impl sealed::SenderFlag for Server {}
impl sealed::ReceiverFlag for Server {}

impl sealed::SocketType for Server {
    fn raw_socket_type() -> SocketType {
        SocketType::Server
    }
}

unsafe impl Sync for Socket<Server> {}
unsafe impl Send for Socket<Server> {}

impl Socket<Server> {
    /// # set an hello message that will be sent when a new peer connect `ZMQ_HELLO_MSG`
    ///
    /// When set, the socket will automatically send an hello message when a new connection is made
    /// or accepted. You may set this on [`Dealer`], [`Router`], [`Client`], [`Server`] and [`Peer`]
    /// sockets. The combination with [`set_heartbeat_ivl()`] is powerful and simplify
    /// protocols, as now heartbeat and sending the hello message can be left out of protocols and
    /// be handled by zeromq.
    ///
    /// [`Dealer`]: super::DealerSocket
    /// [`Router`]: super::RouterSocket
    /// [`Client`]: super::ClientSocket
    /// [`Server`]: ServerSocket
    /// [`Peer`]: super::PeerSocket
    /// [`set_heartbeat_ivl()`]: #method.set_heartbeat_ivl
    pub fn set_hello_message<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::HelloMessage, value)
    }

    /// # set a disconnect message that the socket will generate when accepted peer disconnect `ZMQ_DISCONNECT_MSG`
    ///
    /// When set, the socket will generate a disconnect message when accepted peer has been
    /// disconnected. You may set this on [`Router`], [`Server`] and [`Peer`] sockets. The
    /// combination with [`set_heartbeat_ivl()`] is powerful and simplify protocols, when heartbeat
    /// recognize a connection drop it will generate a disconnect message that can match the
    /// protocol of the application.
    ///
    /// [`Router`]: super::RouterSocket
    /// [`Server`]: ServerSocket
    /// [`Peer`]: super::PeerSocket
    /// [`set_heartbeat_ivl()`]: #method.set_heartbeat_ivl
    pub fn set_disconnect_message<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::DisconnectMessage, value)
    }
}
