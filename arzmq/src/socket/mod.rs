use alloc::sync::Arc;
use core::{iter, marker::PhantomData, ops::ControlFlow};

use bitflags::bitflags;
use derive_more::From;
use num_traits::PrimInt;

use crate::{
    ZmqResult,
    context::Context,
    ffi::RawSocket,
    message::{Message, MultipartMessage, Sendable},
    sealed, zmq_sys_crate,
};

#[cfg(feature = "draft-api")]
mod channel;
#[cfg(feature = "draft-api")]
mod client;
mod dealer;
#[cfg(feature = "draft-api")]
mod dish;
#[cfg(feature = "draft-api")]
mod gather;
pub(crate) mod monitor;
mod pair;
#[cfg(feature = "draft-api")]
mod peer;
mod publish;
mod pull;
mod push;
#[cfg(feature = "draft-api")]
mod radio;
mod reply;
mod request;
mod router;
#[cfg(feature = "draft-api")]
mod scatter;
#[cfg(feature = "draft-api")]
mod server;
mod stream;
mod subscribe;
mod xpublish;
mod xsubscribe;

#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use channel::ChannelSocket;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use client::ClientSocket;
pub use dealer::DealerSocket;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use dish::DishSocket;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use gather::GatherSocket;
use monitor::Monitor;
pub use monitor::{MonitorSocket, MonitorSocketEvent};
pub use pair::PairSocket;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use peer::PeerSocket;
pub use publish::PublishSocket;
pub use pull::PullSocket;
pub use push::PushSocket;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use radio::RadioSocket;
pub use reply::ReplySocket;
pub use request::RequestSocket;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use router::RouterNotify;
pub use router::RouterSocket;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use scatter::ScatterSocket;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use server::ServerSocket;
pub use stream::StreamSocket;
pub use subscribe::SubscribeSocket;
pub use xpublish::XPublishSocket;
pub use xsubscribe::XSubscribeSocket;

use crate::{
    auth::ZapDomain,
    security::{GssApiNametype, SecurityMechanism},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
pub enum SocketType {
    Pair = zmq_sys_crate::ZMQ_PAIR as i32,
    Publish = zmq_sys_crate::ZMQ_PUB as i32,
    Subscribe = zmq_sys_crate::ZMQ_SUB as i32,
    Request = zmq_sys_crate::ZMQ_REQ as i32,
    Reply = zmq_sys_crate::ZMQ_REP as i32,
    Dealer = zmq_sys_crate::ZMQ_DEALER as i32,
    Router = zmq_sys_crate::ZMQ_ROUTER as i32,
    Pull = zmq_sys_crate::ZMQ_PULL as i32,
    Push = zmq_sys_crate::ZMQ_PUSH as i32,
    XPublish = zmq_sys_crate::ZMQ_XPUB as i32,
    XSubscribe = zmq_sys_crate::ZMQ_XSUB as i32,
    Stream = zmq_sys_crate::ZMQ_STREAM as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Server = zmq_sys_crate::ZMQ_SERVER as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Client = zmq_sys_crate::ZMQ_CLIENT as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Radio = zmq_sys_crate::ZMQ_RADIO as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Dish = zmq_sys_crate::ZMQ_DISH as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Gather = zmq_sys_crate::ZMQ_GATHER as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Scatter = zmq_sys_crate::ZMQ_SCATTER as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Datagram = zmq_sys_crate::ZMQ_DGRAM as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Peer = zmq_sys_crate::ZMQ_PEER as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Channel = zmq_sys_crate::ZMQ_CHANNEL as i32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum SocketOption {
    Affinity = zmq_sys_crate::ZMQ_AFFINITY as i32,
    RoutingId = zmq_sys_crate::ZMQ_ROUTING_ID as i32,
    Subscribe = zmq_sys_crate::ZMQ_SUBSCRIBE as i32,
    Unsubscribe = zmq_sys_crate::ZMQ_UNSUBSCRIBE as i32,
    Rate = zmq_sys_crate::ZMQ_RATE as i32,
    RecoveryInterval = zmq_sys_crate::ZMQ_RECOVERY_IVL as i32,
    SendBuffer = zmq_sys_crate::ZMQ_SNDBUF as i32,
    ReceiveBuffer = zmq_sys_crate::ZMQ_RCVBUF as i32,
    ReceiveMore = zmq_sys_crate::ZMQ_RCVMORE as i32,
    FileDescriptor = zmq_sys_crate::ZMQ_FD as i32,
    Events = zmq_sys_crate::ZMQ_EVENTS as i32,
    Type = zmq_sys_crate::ZMQ_TYPE as i32,
    Linger = zmq_sys_crate::ZMQ_LINGER as i32,
    ReconnectInterval = zmq_sys_crate::ZMQ_RECONNECT_IVL as i32,
    Backlog = zmq_sys_crate::ZMQ_BACKLOG as i32,
    ReconnectIntervalMax = zmq_sys_crate::ZMQ_RECONNECT_IVL_MAX as i32,
    MaxMessageSize = zmq_sys_crate::ZMQ_MAXMSGSIZE as i32,
    SendHighWatermark = zmq_sys_crate::ZMQ_SNDHWM as i32,
    ReceiveHighWatermark = zmq_sys_crate::ZMQ_RCVHWM as i32,
    MulticastHops = zmq_sys_crate::ZMQ_MULTICAST_HOPS as i32,
    ReceiveTimeout = zmq_sys_crate::ZMQ_RCVTIMEO as i32,
    SendTimeout = zmq_sys_crate::ZMQ_SNDTIMEO as i32,
    LastEndpoint = zmq_sys_crate::ZMQ_LAST_ENDPOINT as i32,
    RouterMandatory = zmq_sys_crate::ZMQ_ROUTER_MANDATORY as i32,
    TcpKeepalive = zmq_sys_crate::ZMQ_TCP_KEEPALIVE as i32,
    TcpKeepaliveCount = zmq_sys_crate::ZMQ_TCP_KEEPALIVE_CNT as i32,
    TcpKeepaliveIdle = zmq_sys_crate::ZMQ_TCP_KEEPALIVE_IDLE as i32,
    TcpKeepaliveInterval = zmq_sys_crate::ZMQ_TCP_KEEPALIVE_INTVL as i32,
    TcpAcceptFilter = zmq_sys_crate::ZMQ_TCP_ACCEPT_FILTER as i32,
    Immediate = zmq_sys_crate::ZMQ_IMMEDIATE as i32,
    XpubVerbose = zmq_sys_crate::ZMQ_XPUB_VERBOSE as i32,
    RouterRaw = zmq_sys_crate::ZMQ_ROUTER_RAW as i32,
    IPv6 = zmq_sys_crate::ZMQ_IPV6 as i32,
    Mechanism = zmq_sys_crate::ZMQ_MECHANISM as i32,
    PlainServer = zmq_sys_crate::ZMQ_PLAIN_SERVER as i32,
    PlainUsername = zmq_sys_crate::ZMQ_PLAIN_USERNAME as i32,
    PlainPassword = zmq_sys_crate::ZMQ_PLAIN_PASSWORD as i32,
    CurvePublicKey = zmq_sys_crate::ZMQ_CURVE_PUBLICKEY as i32,
    CurveSecretKey = zmq_sys_crate::ZMQ_CURVE_SECRETKEY as i32,
    CurveServer = zmq_sys_crate::ZMQ_CURVE_SERVER as i32,
    CurveServerKey = zmq_sys_crate::ZMQ_CURVE_SERVERKEY as i32,
    ProbeRouter = zmq_sys_crate::ZMQ_PROBE_ROUTER as i32,
    RequestCorrelate = zmq_sys_crate::ZMQ_REQ_CORRELATE as i32,
    RequestRelaxed = zmq_sys_crate::ZMQ_REQ_RELAXED as i32,
    Conflate = zmq_sys_crate::ZMQ_CONFLATE as i32,
    ZapDomain = zmq_sys_crate::ZMQ_ZAP_DOMAIN as i32,
    RouterHandover = zmq_sys_crate::ZMQ_ROUTER_HANDOVER as i32,
    TypeOfService = zmq_sys_crate::ZMQ_TOS as i32,
    IpcFilterProcessId = zmq_sys_crate::ZMQ_IPC_FILTER_PID as i32,
    IpcFilterUserId = zmq_sys_crate::ZMQ_IPC_FILTER_UID as i32,
    IpcFilterGroupId = zmq_sys_crate::ZMQ_IPC_FILTER_GID as i32,
    ConnectRoutingId = zmq_sys_crate::ZMQ_CONNECT_ROUTING_ID as i32,
    GssApiServer = zmq_sys_crate::ZMQ_GSSAPI_SERVER as i32,
    GssApiPrincipal = zmq_sys_crate::ZMQ_GSSAPI_PRINCIPAL as i32,
    GssApiServicePrincipal = zmq_sys_crate::ZMQ_GSSAPI_SERVICE_PRINCIPAL as i32,
    GssApiPlainText = zmq_sys_crate::ZMQ_GSSAPI_PLAINTEXT as i32,
    HandshakeInterval = zmq_sys_crate::ZMQ_HANDSHAKE_IVL as i32,
    SocksProxy = zmq_sys_crate::ZMQ_SOCKS_PROXY as i32,
    XpubNoDrop = zmq_sys_crate::ZMQ_XPUB_NODROP as i32,
    XpubManual = zmq_sys_crate::ZMQ_XPUB_MANUAL as i32,
    XpubWelcomeMessage = zmq_sys_crate::ZMQ_XPUB_WELCOME_MSG as i32,
    StreamNotify = zmq_sys_crate::ZMQ_STREAM_NOTIFY as i32,
    InvertMatching = zmq_sys_crate::ZMQ_INVERT_MATCHING as i32,
    HeartbeatInterval = zmq_sys_crate::ZMQ_HEARTBEAT_IVL as i32,
    HeartbeatTimeToLive = zmq_sys_crate::ZMQ_HEARTBEAT_TTL as i32,
    HeartbeatTimeout = zmq_sys_crate::ZMQ_HEARTBEAT_TIMEOUT as i32,
    XpubVerboser = zmq_sys_crate::ZMQ_XPUB_VERBOSER as i32,
    ConnectTimeout = zmq_sys_crate::ZMQ_CONNECT_TIMEOUT as i32,
    MaxTcpTransmitTimeout = zmq_sys_crate::ZMQ_TCP_MAXRT as i32,
    ThreadSafe = zmq_sys_crate::ZMQ_THREAD_SAFE as i32,
    MulticastMaxTransportDataUnitSize = zmq_sys_crate::ZMQ_MULTICAST_MAXTPDU as i32,
    VmciBufferSize = zmq_sys_crate::ZMQ_VMCI_BUFFER_SIZE as i32,
    VmciBufferMinSize = zmq_sys_crate::ZMQ_VMCI_BUFFER_MIN_SIZE as i32,
    VmciBufferMaxSize = zmq_sys_crate::ZMQ_VMCI_BUFFER_MAX_SIZE as i32,
    VmciConntectTimeout = zmq_sys_crate::ZMQ_VMCI_CONNECT_TIMEOUT as i32,
    UseFd = zmq_sys_crate::ZMQ_USE_FD as i32,
    GssApiPrincipalNametype = zmq_sys_crate::ZMQ_GSSAPI_PRINCIPAL_NAMETYPE as i32,
    GssApiServicePrincipalNametype = zmq_sys_crate::ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE as i32,
    BindToDevice = zmq_sys_crate::ZMQ_BINDTODEVICE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    ZapEnforceDomain = zmq_sys_crate::ZMQ_ZAP_ENFORCE_DOMAIN as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Metadata = zmq_sys_crate::ZMQ_METADATA as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    MulticastLoop = zmq_sys_crate::ZMQ_MULTICAST_LOOP as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    RouterNotify = zmq_sys_crate::ZMQ_ROUTER_NOTIFY as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    XpubManualLastValue = zmq_sys_crate::ZMQ_XPUB_MANUAL_LAST_VALUE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    SocksUsername = zmq_sys_crate::ZMQ_SOCKS_USERNAME as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    SocksPassword = zmq_sys_crate::ZMQ_SOCKS_PASSWORD as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    InBatchSize = zmq_sys_crate::ZMQ_IN_BATCH_SIZE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    OutBatchSize = zmq_sys_crate::ZMQ_OUT_BATCH_SIZE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    OnlyFirstSubscribe = zmq_sys_crate::ZMQ_ONLY_FIRST_SUBSCRIBE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    ReconnectStop = zmq_sys_crate::ZMQ_RECONNECT_STOP as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    HelloMessage = zmq_sys_crate::ZMQ_HELLO_MSG as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    DisconnectMessage = zmq_sys_crate::ZMQ_DISCONNECT_MSG as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    Priority = zmq_sys_crate::ZMQ_PRIORITY as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    BusyPoll = zmq_sys_crate::ZMQ_BUSY_POLL as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    HiccupMessage = zmq_sys_crate::ZMQ_HICCUP_MSG as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    XsubVerboseUnsubscribe = zmq_sys_crate::ZMQ_XSUB_VERBOSE_UNSUBSCRIBE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    TopicsCount = zmq_sys_crate::ZMQ_TOPICS_COUNT as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    NormMode = zmq_sys_crate::ZMQ_NORM_MODE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    NormUnicastNack = zmq_sys_crate::ZMQ_NORM_UNICAST_NACK as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    NormBufferSize = zmq_sys_crate::ZMQ_NORM_BUFFER_SIZE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    NormSegmentSize = zmq_sys_crate::ZMQ_NORM_SEGMENT_SIZE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    NormBlockSize = zmq_sys_crate::ZMQ_NORM_BLOCK_SIZE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    NormNumnParity = zmq_sys_crate::ZMQ_NORM_NUM_PARITY as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    NormNumnAutoParity = zmq_sys_crate::ZMQ_NORM_NUM_AUTOPARITY as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    NormPush = zmq_sys_crate::ZMQ_NORM_PUSH as i32,
}

impl From<SocketOption> for i32 {
    fn from(value: SocketOption) -> Self {
        value as i32
    }
}

pub struct Socket<T: sealed::SocketType> {
    context: Context,
    pub(crate) socket: Arc<RawSocket>,
    marker: PhantomData<T>,
}

impl<T: sealed::SocketType> Socket<T> {
    pub fn from_context(context: &Context) -> ZmqResult<Self> {
        let socket = RawSocket::from_ctx(&context.inner, T::raw_socket_type() as i32)?;
        Ok(Self {
            context: context.clone(),
            socket: socket.into(),
            marker: PhantomData,
        })
    }

    /// # set 0MQ socket options
    ///
    /// Sets a [`SocketOption`] option on the socket. The bytes version is mostly suitable for
    /// binary data options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn set_sockopt_bytes<V>(&self, option: SocketOption, value: V) -> ZmqResult<()>
    where
        V: AsRef<[u8]>,
    {
        self.socket.set_sockopt_bytes(option.into(), value.as_ref())
    }

    /// # set 0MQ socket options
    ///
    /// Sets a [`SocketOption`] option on the socket. The string version is mostly suitable for
    /// character string options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn set_sockopt_string<V>(&self, option: SocketOption, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.socket
            .set_sockopt_string(option.into(), value.as_ref())
    }

    /// # set 0MQ socket options
    ///
    /// Sets a [`SocketOption`] option on the socket. The int version is mostly suitable for
    /// integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn set_sockopt_int<V>(&self, option: SocketOption, value: V) -> ZmqResult<()>
    where
        V: PrimInt + Default,
    {
        self.socket.set_sockopt_int(option.into(), value)
    }

    /// # set 0MQ socket options
    ///
    /// Sets a [`SocketOption`] option on the socket. The bool version is mostly suitable for
    /// 0/1 integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn set_sockopt_bool(&self, option: SocketOption, value: bool) -> ZmqResult<()> {
        self.socket.set_sockopt_bool(option.into(), value)
    }

    /// # get 0MQ socket options
    ///
    /// Gets a [`SocketOption`] option on the socket. The bytes version is mostly suitable for
    /// binary data options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn get_sockopt_bytes(&self, option: SocketOption) -> ZmqResult<Vec<u8>> {
        self.socket.get_sockopt_bytes(option.into())
    }

    /// # get 0MQ socket options
    ///
    /// Gets a [`SocketOption`] option on the socket. The string version is mostly suitable for
    /// character string options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn get_sockopt_string(&self, option: SocketOption) -> ZmqResult<String> {
        self.socket.get_sockopt_string(option.into())
    }

    /// # get 0MQ socket options
    ///
    /// Gets a [`SocketOption`] option on the socket. The int version is mostly suitable for
    /// integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn get_sockopt_int<V>(&self, option: SocketOption) -> ZmqResult<V>
    where
        V: PrimInt + Default,
    {
        self.socket.get_sockopt_int(option.into())
    }

    /// # get 0MQ socket options
    ///
    /// Gets a [`SocketOption`] option on the socket. The bool version is mostly suitable for
    /// 0/1 integer options.
    ///
    /// For convenience, many options have their dedicated method.
    ///
    /// [`SocketOption`]: SocketOption
    pub fn get_sockopt_bool(&self, option: SocketOption) -> ZmqResult<bool> {
        self.socket.get_sockopt_bool(option.into())
    }

    /// # Set I/O thread affinity `ZMQ_AFFINITY`
    ///
    /// The [`Affinity`] option shall set the I/O thread affinity for newly created connections on
    /// the specified [`Socket`].
    ///
    /// Affinity determines which threads from the 0MQ I/O thread pool associated with the
    /// socket’s context shall handle newly created connections. A value of zero specifies no
    /// affinity, meaning that work shall be distributed fairly among all 0MQ I/O threads in the
    /// thread pool. For non-zero values, the lowest bit corresponds to thread 1, second lowest bit
    /// to thread 2 and so on. For example, a value of 3 specifies that subsequent connections on
    /// [`Socket`] shall be handled exclusively by I/O threads 1 and 2.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 0             | N/A                     |
    ///
    /// [`Socket`]: Socket
    /// [`Affinity`]: SocketOption::Affinity
    pub fn set_affinity(&self, value: u64) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Affinity, value)
    }

    /// # Retrieve I/O thread affinity `ZMQ_AFFINITY`
    ///
    /// The [`Affinity`] option shall retrieve the I/O thread affinity for newly
    /// created connections on the specified [`Socket`].
    ///
    /// Affinity determines which threads from the 0MQ I/O thread pool associated with the
    /// socket’s context shall handle newly created connections. A value of zero specifies no
    /// affinity, meaning that work shall be distributed fairly among all 0MQ I/O threads in the
    /// thread pool. For non-zero values, the lowest bit corresponds to thread 1, second lowest bit
    /// to thread 2 and so on. For example, a value of 3 specifies that subsequent connections on
    /// [`Socket`] shall be handled exclusively by I/O threads 1 and 2.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 0             | N/A                     |
    ///
    /// [`Socket`]: Socket
    /// [`Affinity`]: SocketOption::Affinity
    pub fn affinity(&self) -> ZmqResult<u64> {
        self.get_sockopt_int(SocketOption::Affinity)
    }

    /// # Set maximum length of the queue of outstanding connections `ZMQ_BACKLOG`
    ///
    /// The [`Backlog`] option shall set the maximum length of the queue of outstanding peer
    /// connections for the specified [`Socket`]; this only applies to connection-oriented
    /// transports. For details refer to your operating system documentation for the `listen`
    /// function.
    ///
    /// | Default value           | Applicable socket types                       |
    /// | :---------------------: | :-------------------------------------------: |
    /// | 100                     | all, only for connection-oriented transports. |
    ///
    /// [`Socket`]: Socket
    /// [`Backlog`]: SocketOption::Backlog
    pub fn set_backlog(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Backlog, value)
    }

    /// # Retrieve maximum length of the queue of outstanding connections `ZMQ_BACKLOG`
    ///
    /// The [`Backlog`] option shall retrieve the maximum length of the queue of outstanding peer
    /// connections for the specified [`Socket`]; this only applies to connection-oriented
    /// transports. For details refer to your operating system documentation for the `listen`
    /// function.
    ///
    /// | Default value           | Applicable socket types                       |
    /// | :---------------------: | :-------------------------------------------: |
    /// | 100                     | all, only for connection-oriented transports. |
    ///
    /// [`Socket`]: Socket
    /// [`Backlog`]: SocketOption::Backlog
    pub fn backlog(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::Backlog)
    }

    /// # Set connect() timeout `ZMQ_CONNECT_TIMEOUT`
    ///
    /// Sets how long to wait before timing-out a [`connect()`] system call. The [`connect()`]
    /// system call normally takes a long time before it returns a time out error. Setting this
    /// option allows the library to time out the call at an earlier interval.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | 0 ms (disabled)         | all, when using TCP transports. |
    ///
    /// [`connect()`]: #method.connect
    pub fn set_connect_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ConnectTimeout, value)
    }

    /// # Retrieve connect() timeout `ZMQ_CONNECT_TIMEOUT`
    ///
    /// Retrieves how long to wait before timing-out a [`connect()`] system call. The [`connect()`]
    /// system call normally takes a long time before it returns a time out error. Setting this
    /// option allows the library to time out the call at an earlier interval.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | 0 ms (disabled)         | all, when using TCP transports. |
    ///
    /// [`connect()`]: #method.connect
    pub fn connect_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ConnectTimeout)
    }

    /// # Retrieve socket event state `ZMQ_EVENTS`
    ///
    /// The [`events()`] option shall retrieve the event state for the specified 'socket'. The
    /// returned value is a bit mask constructed by OR’ing a combination of the following event
    /// flags:
    ///
    /// * [`POLL_IN`] Indicates that at least one message may be received from the specified socket
    ///   without blocking.
    /// * [`POLL_OUT`] Indicates that at least one message may be sent to the specified socket
    ///   without blocking.
    ///
    /// The combination of a file descriptor returned by the 'ZMQ_FD' option being ready for
    /// reading but no actual events returned by a subsequent retrieval of the [`events()`] option
    /// is valid; applications should simply ignore this case and restart their polling
    /// operation/event loop.
    ///
    /// | Default value | Applicable socket types         |
    /// | :-----------: | :-----------------------------: |
    /// | None          | all                             |
    ///
    /// [`events()`]: #method.events
    /// [`POLL_IN`]: PollEvents::POLL_IN
    /// [`POLL_OUT`]: PollEvents::POLL_OUT
    pub fn events(&self) -> ZmqResult<PollEvents> {
        self.get_sockopt_int::<i16>(SocketOption::Events)
            .map(PollEvents::from_bits_truncate)
    }

    pub fn set_gssapi_plaintext(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::GssApiPlainText, value)
    }

    pub fn set_gssapi_service_principal_nametype(&self, value: GssApiNametype) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::GssApiServicePrincipalNametype, value as i32)
    }

    pub fn gssapi_service_principal_nametype(&self) -> ZmqResult<GssApiNametype> {
        self.get_sockopt_int::<i32>(SocketOption::GssApiServicePrincipalNametype)
            .and_then(GssApiNametype::try_from)
    }

    pub fn gssapi_plaintext(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::GssApiPlainText)
    }

    pub fn set_gssapi_principal<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::GssApiPrincipal, value.as_ref())
    }

    pub fn gssapi_principal(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::GssApiPrincipal)
    }

    pub fn set_gssapi_principal_nametype(&self, value: GssApiNametype) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::GssApiPrincipalNametype, value as i32)
    }

    pub fn gssapi_principal_nametype(&self) -> ZmqResult<GssApiNametype> {
        self.get_sockopt_int::<i32>(SocketOption::GssApiPrincipalNametype)
            .and_then(GssApiNametype::try_from)
    }

    pub fn set_handshake_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HandshakeInterval, value)
    }

    pub fn handshake_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::HandshakeInterval)
    }

    pub fn set_heartbeat_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HeartbeatInterval, value)
    }

    pub fn heartbeat_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::HeartbeatInterval)
    }

    pub fn set_heartbeat_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HeartbeatTimeout, value)
    }

    pub fn heartbeat_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::HeartbeatTimeout)
    }

    pub fn set_heartbeat_ttl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HeartbeatTimeToLive, value)
    }

    pub fn heartbeat_ttl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::HeartbeatTimeToLive)
    }

    pub fn set_immediate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::Immediate, value)
    }

    pub fn immediate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::Immediate)
    }

    pub fn set_ipv6(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::IPv6, value)
    }

    pub fn ipv6(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::IPv6)
    }

    pub fn set_linger(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Linger, value)
    }

    pub fn linger(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::Linger)
    }

    pub fn last_endpoint(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::LastEndpoint)
    }

    pub fn set_maxmsgsize(&self, value: i64) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::MaxMessageSize, value)
    }

    pub fn maxmsgsize(&self) -> ZmqResult<i64> {
        self.get_sockopt_int(SocketOption::MaxMessageSize)
    }

    pub fn set_security_mechanism(&self, security: SecurityMechanism) -> ZmqResult<()> {
        security.apply(self)
    }

    pub fn security_mechanism(&self) -> ZmqResult<SecurityMechanism> {
        SecurityMechanism::try_from(self)
    }

    pub fn set_multicast_hops(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::MulticastHops, value)
    }

    pub fn multicast_hops(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::MulticastHops)
    }

    pub fn set_probe_router(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::ProbeRouter, value)
    }

    pub fn set_rate(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Rate, value)
    }

    pub fn rate(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::Rate)
    }

    pub fn set_receive_buffer(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReceiveBuffer, value)
    }

    pub fn receive_buffer(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReceiveBuffer)
    }

    pub fn set_receive_highwater_mark(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReceiveHighWatermark, value)
    }

    pub fn receive_highwater_mark(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReceiveHighWatermark)
    }

    pub fn set_receive_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReceiveTimeout, value)
    }

    pub fn receive_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReceiveTimeout)
    }

    pub fn set_reconnect_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReconnectInterval, value)
    }

    pub fn reconnect_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReconnectInterval)
    }

    pub fn set_reconnect_interval_max(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReconnectIntervalMax, value)
    }

    pub fn reconnect_interval_max(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReconnectIntervalMax)
    }

    pub fn set_recovery_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::RecoveryInterval, value)
    }

    pub fn recovery_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::RecoveryInterval)
    }

    pub fn set_send_buffer(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::SendBuffer, value)
    }

    pub fn send_buffer(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::SendBuffer)
    }

    pub fn set_send_highwater_mark(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::SendHighWatermark, value)
    }

    pub fn send_highwater_mark(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::SendHighWatermark)
    }

    pub fn set_send_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::SendTimeout, value)
    }

    pub fn send_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::SendTimeout)
    }

    pub fn set_socks_proxy<V>(&self, value: Option<V>) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        match value {
            None => self.set_sockopt_bytes(SocketOption::SocksProxy, vec![]),
            Some(ref_value) => self.set_sockopt_string(SocketOption::SocksProxy, ref_value),
        }
    }

    pub fn socks_proxy(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::SocksProxy)
    }

    pub fn set_tcp_keepalive(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepalive, value)
    }

    pub fn tcp_keepalive(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepalive)
    }

    pub fn set_tcp_keepalive_count(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepaliveCount, value)
    }

    pub fn tcp_keepalive_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepaliveCount)
    }

    pub fn set_tcp_keepalive_idle(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepaliveIdle, value)
    }

    pub fn tcp_keepalive_idle(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepaliveIdle)
    }

    pub fn set_tcp_keepalive_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepaliveInterval, value)
    }

    pub fn tcp_keepalive_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepaliveInterval)
    }

    /// # Set TCP Maximum Retransmit Timeout `ZMQ_TCP_MAXRT`
    ///
    /// On OSes where it is supported, sets how long before an unacknowledged TCP retransmit times
    /// out. The system normally attempts many TCP retransmits following an exponential backoff
    /// strategy. This means that after a network outage, it may take a long time before the
    /// session can be re-established. Setting this option allows the timeout to happen at a
    /// shorter interval.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | 0 ms                    | all, when using TCP transports. |
    pub fn set_tcp_max_retransmit_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::MaxTcpTransmitTimeout, value)
    }

    /// # Retrieve Max TCP Retransmit Timeout `ZMQ_TCP_MAXRT`
    ///
    /// On OSes where it is supported, retrieves how long before an unacknowledged TCP retransmit
    /// times out. The system normally attempts many TCP retransmits following an exponential
    /// backoff strategy. This means that after a network outage, it may take a long time before
    /// the session can be re-established. Setting this option allows the timeout to happen at a
    /// shorter interval.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | 0 ms                    | all, when using TCP transports. |
    pub fn tcp_max_retransmit_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::MaxTcpTransmitTimeout)
    }

    /// # Set the Type-of-Service on socket `ZMQ_TOS`
    ///
    /// Sets the ToS fields (Differentiated services (DS) and Explicit Congestion Notification
    /// (ECN) field of the IP header. The ToS field is typically used to specify a packets
    /// priority. The availability of this option is dependent on intermediate network equipment
    /// that inspect the ToS field and provide a path for low-delay, high-throughput,
    /// highly-reliable service, etc.
    ///
    /// | Default value           | Applicable socket types                      |
    /// | :---------------------: | :------------------------------------------: |
    /// | 0                       | all, only for connection-oriented transports |
    pub fn set_type_of_service(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TypeOfService, value)
    }

    /// # Retrieve the Type-of-Service socket override status `ZMQ_TOS`
    ///
    /// Retrieve the IP_TOS option for the socket.
    ///
    /// | Default value           | Applicable socket types                      |
    /// | :---------------------: | :------------------------------------------: |
    /// | 0                       | all, only for connection-oriented transports |
    pub fn type_of_service(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TypeOfService)
    }

    /// # Set RFC 27 authentication domain `ZMQ_ZAP_DOMAIN`
    ///
    /// Sets the domain for ZAP (ZMQ RFC 27) authentication. A ZAP domain must be specified to
    /// enable authentication. When the ZAP domain is empty, which is the default, ZAP
    /// authentication is disabled.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | empty                   | all, when using TCP transports. |
    pub fn set_zap_domain(&self, domain: ZapDomain) -> ZmqResult<()> {
        domain.apply(self)
    }

    /// # Retrieve RFC 27 authentication domain `ZMQ_ZAP_DOMAIN`
    ///
    /// The [`zap_domain()`] option shall retrieve the last ZAP domain set for the socket. The
    /// returned value shall be a NULL-terminated string and MAY be empty. An empty string means
    /// that ZAP authentication is disabled. The returned size SHALL include the terminating null
    /// byte.
    ///
    /// | Default value           | Applicable socket types         |
    /// | :---------------------: | :-----------------------------: |
    /// | not set                 | all, when using TCP transports. |
    ///
    /// [`zap_domain()`]: #method.zap_domain
    pub fn zap_domain(&self) -> ZmqResult<ZapDomain> {
        self.get_sockopt_string(SocketOption::ZapDomain)
            .map(ZapDomain::from)
    }

    pub fn bind<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.bind(endpoint.as_ref())
    }

    pub fn unbind<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.unbind(endpoint.as_ref())
    }

    pub fn connect<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.connect(endpoint.as_ref())
    }

    pub fn disconnect<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.disconnect(endpoint.as_ref())
    }

    pub fn monitor<F>(&self, events: F) -> ZmqResult<MonitorSocket>
    where
        F: Into<MonitorFlags>,
    {
        let fd = self
            .socket
            .get_sockopt_int::<usize>(SocketOption::FileDescriptor as i32)?;
        let monitor_endpoint = format!("inproc://monitor.s-{fd}");

        self.socket
            .monitor(&monitor_endpoint, events.into().bits() as i32)?;

        let monitor = RawSocket::from_ctx(
            self.context.as_raw(),
            <Monitor as sealed::SocketType>::raw_socket_type() as i32,
        )?;

        monitor.connect(&monitor_endpoint)?;

        Ok(Socket {
            context: self.context.clone(),
            socket: monitor.into(),
            marker: PhantomData,
        })
    }

    pub fn poll<E>(&self, events: E, timeout_ms: i64) -> ZmqResult<i32>
    where
        E: Into<PollEvents>,
    {
        let poll_events = PollEvents::from_bits_truncate(events.into().bits());
        self.socket.poll(poll_events, timeout_ms)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
#[from(u16)]
pub struct MonitorFlags(u16);

bitflags! {
    impl MonitorFlags: u16 {
        const Connected                 = 0b0000_0000_0000_0001;
        const ConnectDelayed            = 0b0000_0000_0000_0010;
        const ConnectRetried            = 0b0000_0000_0000_0100;
        const Listening                 = 0b0000_0000_0000_1000;
        const BindFailed                = 0b0000_0000_0001_0000;
        const Accepted                  = 0b0000_0000_0010_0000;
        const AcceptFailed              = 0b0000_0000_0100_0000;
        const Closed                    = 0b0000_0000_1000_0000;
        const CloseFailed               = 0b0000_0001_0000_0000;
        const Disconnected              = 0b0000_0010_0000_0000;
        const MonitorStopped            = 0b0000_0100_0000_0000;
        const HandshakeFailedNoDetail   = 0b0000_1000_0000_0000;
        const HandshakeSucceeded        = 0b0001_0000_0000_0000;
        const HandshakeFailedProtocol   = 0b0010_0000_0000_0000;
        const HandshakeFailedAuth       = 0b0100_0000_0000_0000;
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecvFlags(i32);

bitflags! {
    impl RecvFlags: i32 {
        const DONT_WAIT = 0b00000001;
    }
}

pub trait Receiver {
    fn recv_msg<F>(&self, flags: F) -> ZmqResult<Message>
    where
        F: Into<RecvFlags> + Copy;
}

pub trait MultipartReceiver: Receiver {
    fn recv_multipart<F>(&self, flags: F) -> ZmqResult<MultipartMessage>
    where
        F: Into<RecvFlags> + Copy,
    {
        iter::repeat_with(|| self.recv_msg(flags))
            .try_fold(
                MultipartMessage::new(),
                |mut parts, zmq_result| match zmq_result {
                    Err(e) => ControlFlow::Break(Err(e)),
                    Ok(zmq_msg) => {
                        let got_more = zmq_msg.get_more();
                        parts.push_back(zmq_msg);
                        if got_more {
                            ControlFlow::Continue(parts)
                        } else {
                            ControlFlow::Break(Ok(parts))
                        }
                    }
                },
            )
            .break_value()
            .unwrap()
    }
}

impl<T: sealed::SocketType + sealed::ReceiverFlag + Unpin> Receiver for Socket<T> {
    fn recv_msg<F>(&self, flags: F) -> ZmqResult<Message>
    where
        F: Into<RecvFlags> + Copy,
    {
        self.socket
            .recv(flags.into().bits())
            .map(Message::from_raw_msg)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SendFlags(i32);

bitflags! {
    impl SendFlags: i32 {
        const DONT_WAIT = 0b00000001;
        const SEND_MORE = 0b00000010;
    }
}

pub trait Sender {
    fn send_msg<F>(&self, msg: Message, flags: F) -> ZmqResult<()>
    where
        F: Into<SendFlags> + Copy;
}

pub trait MultipartSender: Sender {
    fn send_multipart<F>(&self, iter: MultipartMessage, flags: F) -> ZmqResult<()>
    where
        F: Into<SendFlags> + Copy,
    {
        let mut last_part: Option<Message> = None;
        for part in iter {
            let maybe_last = last_part.take();
            if let Some(last) = maybe_last {
                self.send_msg(last, flags.into() | SendFlags::SEND_MORE)?;
            }
            last_part = Some(part);
        }
        if let Some(last) = last_part {
            self.send_msg(last, flags)
        } else {
            Ok(())
        }
    }
}

impl<T: sealed::SocketType + sealed::SenderFlag + Unpin> Sender for Socket<T> {
    fn send_msg<F>(&self, msg: Message, flags: F) -> ZmqResult<()>
    where
        F: Into<SendFlags> + Copy,
    {
        msg.send(self, flags.into().bits())
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PollEvents(i16);

bitflags! {
    impl PollEvents: i16 {
        const POLL_IN = 0b0000_0001;
        const POLL_OUT = 0b0000_0010;
        const POLL_ERR = 0b0000_0100;
        const POLL_PRI = 0b0000_1000;
    }
}
