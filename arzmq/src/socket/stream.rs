use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOptions, SocketType},
};

/// # A stream socket `ZMQ_STREAM`
///
/// A socket of type [`Stream`] is used to send and receive TCP data from a non-0MQ peer, when
/// using the tcp:// transport. A [`Stream`] socket can act as client and/or server, sending
/// and/or receiving TCP data asynchronously.
///
/// When receiving TCP data, a [`Stream`] socket shall prepend a message part containing the
/// routing id of the originating peer to the message before passing it to the application.
/// Messages received are fair-queued from among all connected peers.
///
/// When sending TCP data, a [`Stream`] socket shall remove the first part of the message and use
/// it to determine the routing id of the peer the message shall be routed to, and unroutable
/// messages shall cause an `Err(`[`HostUnreachable`]`)` or `Err(`[`Again`]`)` error.
///
/// To open a connection to a server, use the [`connect()`] call, and then fetch the socket routing
/// id using [`routing_id()`] option.
///
/// To close a specific connection, send the routing id frame followed by a zero-length message.
///
/// When a connection is made, a zero-length message will be received by the application.
/// Similarly, when the peer disconnects (or the connection is lost), a zero-length message will
/// be received by the application.
///
/// You must send one routing id frame followed by one data frame. The [`SEND_MORE`] flag is
/// required for routing id frames but is ignored on data frames.
///
/// [`Stream`]: StreamSocket
/// [`HostUnreachable`]: crate::ZmqError::HostUnreachable
/// [`Again`]: crate::ZmqError::Again
/// [`connect()`]: #method.connect
/// [`routing_id()`]: #method.routing_id
/// [`SEND_MORE`]: super::SendFlags::SEND_MORE
pub type StreamSocket = Socket<Stream>;

pub struct Stream {}

impl sealed::SenderFlag for Stream {}
impl sealed::ReceiverFlag for Stream {}

impl sealed::SocketType for Stream {
    fn raw_socket_type() -> SocketType {
        SocketType::Stream
    }
}

unsafe impl Sync for Socket<Stream> {}
unsafe impl Send for Socket<Stream> {}

impl Socket<Stream> {
    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::RoutingId as i32)
    }

    pub fn set_connect_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::ConnectRoutingId as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_stream_notify(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::StreamNotify as i32, value)
    }
}
