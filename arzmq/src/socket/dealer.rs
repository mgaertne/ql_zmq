use super::{MultipartReceiver, MultipartSender, Socket, SocketOption, SocketType};
use crate::{ZmqResult, sealed};

/// # A dealer socket `ZMQ_DEALER`
///
/// A socket of type [`Dealer`] is an advanced pattern used for extending request/reply sockets.
/// Each message sent is round-robined among all connected peers, and each message received is
/// fair-queued from all connected peers.
///
/// When a [`Dealer`] socket enters the 'mute' state due to having reached the high water mark for
/// all peers, or, for connection-oriented transports, if the [`immediate()`] option is set and
/// there are no peers at /// all, then any [`send_msg()`] operations on the socket shall block
/// until the mute state ends or at least one peer becomes available for sending; messages are not
/// discarded.
///
/// When a [`Dealer`] socket is connected to a [`Reply`](type@super::ReplySocket) socket each
/// message sent must consist of an empty message part, the delimiter, followed by one or more body
/// parts.
///
/// [`Dealer`]: DealerSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #impl-Sender<T>-for-Socket<T>
pub type DealerSocket = Socket<Dealer>;

pub struct Dealer {}

impl sealed::SenderFlag for Dealer {}
impl sealed::ReceiverFlag for Dealer {}

impl sealed::SocketType for Dealer {
    fn raw_socket_type() -> SocketType {
        SocketType::Dealer
    }
}

unsafe impl Sync for Socket<Dealer> {}
unsafe impl Send for Socket<Dealer> {}

impl MultipartSender for Socket<Dealer> {}
impl MultipartReceiver for Socket<Dealer> {}

impl Socket<Dealer> {
    /// # Keep only last message `ZMQ_CONFLATE`
    ///
    /// If set, a socket shall keep only one message in its inbound/outbound queue, this message
    /// being the last message received/the last message to be sent. Ignores [`recvhwm()`] and
    /// [`sndhwm()`] options. Does not support multi-part messages, in particular, only one part of
    /// it is kept in the socket internal queue.
    ///
    /// # Note
    ///
    /// If [`recv_msg()`] is not called on the inbound socket, the queue and memory will grow with
    /// each message received. Use [`events()`] to trigger the conflation of the messages.
    ///
    /// [`recvhwm()`]: #method.recvhwm
    /// [`sndhwm()`]: #method.sndhwm
    /// [`recv_msg()`]: #method.recv_msg
    /// [`events()`]: #method.events
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::Conflate, value)
    }

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
    /// [`set_router_handover()`]: #method.set_router_handover
    pub fn set_routing_id<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::RoutingId, value)
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
        self.get_sockopt_string(SocketOption::RoutingId)
    }

    /// # set a hiccup message that the socket will generate when connected peer temporarily disconnect `ZMQ_HICCUP_MSG`
    ///
    /// When set, the socket will generate a hiccup message when connect peer has been
    /// disconnected. You may set this on [`Dealer`], [`Client`] and [`Peer`] sockets. The
    /// combination with [`set_heartbeat_ivl()`] is powerful and simplify protocols, when
    /// heartbeat recognize a connection drop it will generate a hiccup message that can match the
    /// protocol of the application.
    ///
    /// [`Dealer`]: DealerSocket
    /// [`Client`]: super::ClientSocket
    /// [`Peer`]: super::PeerSocket
    /// [`set_heartbeat_ivl()`]: #method.set_heartbeat_ivl
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_hiccup_message<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::HiccupMessage, value)
    }

    /// # set an hello message that will be sent when a new peer connect `ZMQ_HELLO_MSG`
    ///
    /// When set, the socket will automatically send an hello message when a new connection is made
    /// or accepted. You may set this on [`Dealer`], [`Router`], [`Client`], [`Server`] and [`Peer`]
    /// sockets. The combination with [`set_heartbeat_ivl()`] is powerful and simplify
    /// protocols, as now heartbeat and sending the hello message can be left out of protocols and
    /// be handled by zeromq.
    ///
    /// [`Dealer`]: DealerSocket
    /// [`Router`]: super::RouterSocket
    /// [`Client`]: super::ClientSocket
    /// [`Server`]: super::ServerSocket
    /// [`Peer`]: super::PeerSocket
    /// [`set_heartbeat_ivl()`]: #method.set_heartbeat_ivl
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_hello_message<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::HelloMessage, value)
    }
}
