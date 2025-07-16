use crate::{
    ZmqResult, sealed,
    socket::{MultipartReceiver, MultipartSender, RecvFlags, Socket, SocketOptions, SocketType},
};

/// # A Reply socket `ZMQ_REP`
///
/// A socket of type [`Reply`] is used by a service to receive requests from and send replies to a
/// client. This socket type allows only an alternating sequence of [`recv_msg()`] and subsequent
/// [`send_msg()`] calls. Each request received is fair-queued from among all clients, and each
/// reply sent is routed to the client that issued the last request. If the original requester does
/// not exist any more the reply is silently discarded.
///
/// [`Reply`]: ReplySocket
/// [`send_msg()`]: #impl-Sender<T>-for-Socket<T>
/// [`recv_msg()`]: #impl-Receiver<T>-for-Socket<T>
pub type ReplySocket = Socket<Reply>;

pub struct Reply {}

impl sealed::SenderFlag for Reply {}
impl sealed::ReceiverFlag for Reply {}

impl sealed::SocketType for Reply {
    fn raw_socket_type() -> SocketType {
        SocketType::Reply
    }
}

unsafe impl Sync for Socket<Reply> {}
unsafe impl Send for Socket<Reply> {}

impl MultipartSender<Reply> for Socket<Reply> {}
impl<F: Into<RecvFlags> + Copy> MultipartReceiver<F> for Socket<Reply> {}

impl Socket<Reply> {
    /// # Set socket routing id `ZMQ_ROUTING_ID`
    ///
    /// The [`set_routing_id()`] option shall set the routing id of the specified 'socket' when
    /// connecting to a [`Router`] socket.
    ///
    /// A routing id must be at least one byte and at most 255 bytes long. Identities starting with
    /// a zero byte are reserved for use by the 0MQ infrastructure.
    ///
    /// If two clients use the same routing id when connecting to a [`Router`], the results shall
    /// depend on the [`set_router_handover()`] option setting. If that is not set (or set to the
    /// default of zero), the [`Router`] socket shall reject clients trying to connect with an
    /// already-used routing id. If that option is set to `true`, the [`Router`]socket shall
    /// hand-over the connection to the new client and disconnect the existing one.
    ///
    /// [`set_routing_id()`]: #method.set_routing_id
    /// [`Router`]: super::RouterSocket
    /// [`set_router_handover()`]: super::RouterSocket::set_router_handover
    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::RoutingId, value)
    }

    /// # Retrieve socket routing id `ZMQ_ROUTING_ID`
    ///
    /// The [`routing_id()`] option shall retrieve the routing id of the specified 'socket'.
    /// Routing ids are used only by the request/reply pattern. Specifically, it can be used in
    /// tandem with [`Router`] socket to route messages to the peer with a specific routing id.
    ///
    /// A routing id must be at least one byte and at most 255 bytes long. Identities starting
    /// with a zero byte are reserved for use by the 0MQ infrastructure.
    ///
    /// [`routing_id()`]: #method.routing_id
    /// [`Router`]: super::RouterSocket
    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::RoutingId)
    }
}
