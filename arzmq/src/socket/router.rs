#[cfg(feature = "draft-api")]
use bitflags::bitflags;

use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOptions, SocketType},
};

/// # A router socket `ZMQ_ROUTER`
///
/// A socket of type [`Router`] is an advanced socket type used for extending request/reply
/// sockets. When receiving messages a [`Router`] socket shall prepend a message part containing
/// the routing id of the originating peer to the message before passing it to the application.
/// Messages received are fair-queued from among all connected peers. When sending messages a
/// [`Router`] socket shall remove the first part of the message and use it to determine the
/// [`routing_id()`] of the peer the message shall be routed to. If the peer does not exist
/// anymore, or has never existed, the message shall be silently discarded. However, if
/// [`RouterMandatory`] socket option is set to `true`, the socket shall fail with
/// `Err(`[`HostUnreachable`]`)` in both cases.
///
/// When a [`Router`] socket enters the 'mute' state due to having reached the high water mark for
/// all peers, then any messages sent to the socket shall be dropped until the mute state ends.
/// Likewise, any messages routed to a peer for which the individual high water mark has been
/// reached shall also be dropped. If, [`RouterMandatory`] is set to `true`, the socket shall
/// block or return `Err(`[`Again`]`)` in both cases.
///
/// When a [`Router`] socket has [`RouterMandatory`] flag set to `true`, the socket shall generate
/// [`ZMQ_POLLIN`] events upon reception of messages from one or more peers. Likewise, the socket
/// shall generate [`ZMQ_POLLOUT`] events when at least one message can be sent to one or more
/// peers.
///
/// When a [`Request`] socket is connected to a [`Router`] socket, in addition to the routing id of
/// the originating peer each message received shall contain an empty delimiter message part.
/// Hence, the entire structure of each received message as seen by the application becomes: one
/// or more routing id parts, delimiter part, one or more body parts. When sending replies to a
/// [`Request`] socket the application must include the delimiter
/// part.
///
/// [`Router`]: RouterSocket
/// [`Request`]: super::RequestSocket
/// [`routing_id()`]: #method.routing_id
/// [`RouterMandatory`]: SocketOptions::RouterMandatory
/// [`HostUnreachable`]: crate::ZmqError::HostUnreachable
/// [`Again`]: crate::ZmqError::Again
/// [`ZMQ_POLLIN`]: super::PollEvents::ZMQ_POLLIN
/// [`ZMQ_POLLOUT`]: super::PollEvents::ZMQ_POLLOUT
pub type RouterSocket = Socket<Router>;

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

impl Socket<Router> {
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
    /// [`Router`]: RouterSocket
    /// [`set_router_handover()`]: #method.set_router_handover
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
    /// [`Router`]: RouterSocket
    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::RoutingId)
    }

    /// # Assign the next outbound routing id `ZMQ_CONNECT_ROUTING_ID`
    ///
    /// The [`set_connect_routing_id()`] option sets the peer id of the peer connected via the next
    /// [`connect()`] call, such that that connection is immediately ready for data transfer with
    /// the given routing id. This option applies only to the first subsequent call to
    /// [`connect()`], [`connect()`] calls thereafter use the default connection behaviour.
    ///
    /// Typical use is to set this socket option ahead of each [`connect()`] call. Each connection
    /// MUST be assigned a unique routing id. Assigning a routing id that is already in use is not
    /// allowed.
    ///
    /// Useful when connecting [`Router`] to [`Router`], or [`Stream`] to [`Stream`], as it allows
    /// for immediate sending to peers. Outbound routing id framing requirements for [`Router`] and
    /// [`Stream`] sockets apply.
    ///
    /// The routing id must be from 1 to 255 bytes long and MAY NOT start with a zero byte (such
    /// routing ids are reserved for internal use by the 0MQ infrastructure).
    ///
    /// [`Stream`]: super::StreamSocket
    /// [`Router`]: RouterSocket
    /// [`connect()`]: #method.connect
    /// [`set_connect_routing_id()`]: #method.set_connect_routing_id
    pub fn set_connect_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::ConnectRoutingId, value)
    }

    /// # handle duplicate client routing ids on [`Router`] sockets `ZMQ_ROUTER_HANDOVER`
    ///
    /// If two clients use the same routing id when connecting to a [`Router`], the results shall
    /// depend on the [`set_router_handover()`] option setting. If that is not set (or set to the
    /// default of `false`), the [`Router`] socket shall reject clients trying to connect with an
    /// already-used routing id. If that option is set to `true`, the [`Router`] socket shall
    /// hand-over the connection to the new client and disconnect the existing one.
    ///
    /// [`Router`]: RouterSocket
    /// [`set_router_handover()`]: #method.set_router_handover
    pub fn set_router_handover(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::RouterHandover, value)
    }

    /// # accept only routable messages on [`Router`] sockets `ZMQ_ROUTER_MANDATORY`
    ///
    /// Sets the [`Router`] socket behaviour when an unroutable message is encountered. A value of
    /// `false` is the default and discards the message silently when it cannot be routed or the
    /// peers [`sndhwm()`] is reached. A value of `true` returns an [`HostUnreachable`] error code
    /// if the message cannot be routed or [`Again`] error code if the [`sndhwm()`] is reached and
    /// [`DONT_WAIT`] was used. Without [`DONT_WAIT`] it will block until the [`sndtimeo()`] is
    /// reached or a spot in the send queue opens up.
    ///
    /// When [`set_router_mandatory()`] is set to `true`, [`POLLOUT`] events will be generated if
    /// one or more messages can be sent to at least one of the peers. If
    /// [`set_router_mandatory()`] is set to `false`, the socket will generate a [`POLLOUT`] event
    /// on every call to [`poll()`] resp. [`poll_wait_all()`].
    ///
    /// [`Router`]: RouterSocket
    /// [`sndhwm()`]: #method.sndhwm
    /// [`sndtimeo()`]: #method.sndtimeo
    /// [`HostUnreachable`]: crate::ZmqError::HostUnreachable
    /// [`Again`]: crate::ZmqError::Again
    /// [`DONT_WAIT`]: super::SendFlags::DONT_WAIT
    /// [`set_router_mandatory()`]: #method.set_router_mandatory
    /// [`POLLOUT`]: super::PollEvents::ZMQ_POLLOUT
    /// [`poll()`]: #method.poll
    /// [`poll_wait_all()`]: todo!()
    pub fn set_router_mandatory(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::RouterMandatory, value)
    }

    /// # set a disconnect message that the socket will generate when accepted peer disconnect `ZMQ_DISCONNECT_MSG`
    ///
    /// When set, the socket will generate a disconnect message when accepted peer has been
    /// disconnected. You may set this on [`Router`], [`Server`] and [`Peer`] sockets. The
    /// combination with [`set_heartbeat_ivl()`] is powerful and simplify protocols, when heartbeat
    /// recognize a connection drop it will generate a disconnect message that can match the
    /// protocol of the application.
    ///
    /// [`Router`]: RouterSocket
    /// [`Server`]: todo!()
    /// [`Peer`]: todo!()
    /// [`set_heartbeat_ivl()`]: #method.set_heartbeat_ivl
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_disconnect_message<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::DisconnectMessage, value)
    }

    /// # set an hello message that will be sent when a new peer connect `ZMQ_HELLO_MSG`
    ///
    /// When set, the socket will automatically send an hello message when a new connection is made
    /// or accepted. You may set this on [`Dealer`], [`Router`], [`Client`], [`Server`] and [`Peer`]
    /// sockets. The combination with [`set_heartbeat_ivl()`] is powerful and simplify
    /// protocols, as now heartbeat and sending the hello message can be left out of protocols and
    /// be handled by zeromq.
    ///
    /// [`Dealer`]: super::DealerSocket
    /// [`Router`]: RouterSocket
    /// [`Client`]: todo!()
    /// [`Server`]: todo!()
    /// [`Peer`]: todo!()
    /// [`set_heartbeat_ivl()`]: #method.set_heartbeat_ivl
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_hello_message<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::HelloMessage, value)
    }

    /// # Send connect and disconnect notifications `ZMQ_ROUTER_NOTIFY`
    ///
    /// Enable connect and disconnect notifications on a [`Router`]] socket. When enabled, the
    /// socket delivers a zero-length message (with routing-id as first frame) when a peer
    /// connects or disconnects. It’s possible to notify both events for a peer by OR-ing the flag
    /// values. This option only applies to stream oriented (tcp, ipc) transports.
    ///
    /// [`Router`]: RouterSocket
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_router_notify(&self, value: RouterNotify) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::RouterNotify, value.bits())
    }

    /// Retrieve router socket notification settings `ZMQ_ROUTER_NOTIFY`
    ///
    /// Retrieve the current notification settings of a router socket. The returned value is a
    /// bitmask composed of [`NotifyConnect`] and [`NotifyDisconnect`] flags, meaning connect and
    /// disconnect notifications are enabled, respectively. A value of `0` means the notifications
    /// are off.
    ///
    /// [`Router`]: RouterSocket
    /// [`NotifyConnect`]: RouterNotify::NotifyConnect
    /// [`NotifyDisconnect`]: RouterNotify::NotifyDisconnect
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn router_notify(&self) -> ZmqResult<RouterNotify> {
        self.get_sockopt_int(SocketOptions::RouterNotify)
            .map(RouterNotify::from_bits_truncate)
    }
}
