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

#[doc(cfg(zmq_have_gssapi))]
use crate::security::GssApiNametype;
use crate::{auth::ZapDomain, security::SecurityMechanism};

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
    #[cfg(feature = "curve")]
    #[doc(cfg(feature = "curve"))]
    CurvePublicKey = zmq_sys_crate::ZMQ_CURVE_PUBLICKEY as i32,
    #[cfg(feature = "curve")]
    #[doc(cfg(feature = "curve"))]
    CurveSecretKey = zmq_sys_crate::ZMQ_CURVE_SECRETKEY as i32,
    #[cfg(feature = "curve")]
    #[doc(cfg(feature = "curve"))]
    CurveServer = zmq_sys_crate::ZMQ_CURVE_SERVER as i32,
    #[cfg(feature = "curve")]
    #[doc(cfg(feature = "curve"))]
    CurveServerKey = zmq_sys_crate::ZMQ_CURVE_SERVERKEY as i32,
    ProbeRouter = zmq_sys_crate::ZMQ_PROBE_ROUTER as i32,
    RequestCorrelate = zmq_sys_crate::ZMQ_REQ_CORRELATE as i32,
    RequestRelaxed = zmq_sys_crate::ZMQ_REQ_RELAXED as i32,
    Conflate = zmq_sys_crate::ZMQ_CONFLATE as i32,
    ZapDomain = zmq_sys_crate::ZMQ_ZAP_DOMAIN as i32,
    RouterHandover = zmq_sys_crate::ZMQ_ROUTER_HANDOVER as i32,
    TypeOfService = zmq_sys_crate::ZMQ_TOS as i32,
    #[doc(cfg(zmq_have_ipc))]
    IpcFilterProcessId = zmq_sys_crate::ZMQ_IPC_FILTER_PID as i32,
    #[doc(cfg(zmq_have_ipc))]
    IpcFilterUserId = zmq_sys_crate::ZMQ_IPC_FILTER_UID as i32,
    #[doc(cfg(zmq_have_ipc))]
    IpcFilterGroupId = zmq_sys_crate::ZMQ_IPC_FILTER_GID as i32,
    ConnectRoutingId = zmq_sys_crate::ZMQ_CONNECT_ROUTING_ID as i32,
    #[doc(cfg(zmq_have_gssapi))]
    GssApiServer = zmq_sys_crate::ZMQ_GSSAPI_SERVER as i32,
    #[doc(cfg(zmq_have_gssapi))]
    GssApiPrincipal = zmq_sys_crate::ZMQ_GSSAPI_PRINCIPAL as i32,
    #[doc(cfg(zmq_have_gssapi))]
    GssApiServicePrincipal = zmq_sys_crate::ZMQ_GSSAPI_SERVICE_PRINCIPAL as i32,
    #[doc(cfg(zmq_have_gssapi))]
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
    #[doc(cfg(zmq_have_vmci))]
    VmciBufferSize = zmq_sys_crate::ZMQ_VMCI_BUFFER_SIZE as i32,
    #[doc(cfg(zmq_have_vmci))]
    VmciBufferMinSize = zmq_sys_crate::ZMQ_VMCI_BUFFER_MIN_SIZE as i32,
    #[doc(cfg(zmq_have_vmci))]
    VmciBufferMaxSize = zmq_sys_crate::ZMQ_VMCI_BUFFER_MAX_SIZE as i32,
    #[doc(cfg(zmq_have_vmci))]
    VmciConntectTimeout = zmq_sys_crate::ZMQ_VMCI_CONNECT_TIMEOUT as i32,
    UseFd = zmq_sys_crate::ZMQ_USE_FD as i32,
    #[doc(cfg(zmq_have_gssapi))]
    GssApiPrincipalNametype = zmq_sys_crate::ZMQ_GSSAPI_PRINCIPAL_NAMETYPE as i32,
    #[doc(cfg(zmq_have_gssapi))]
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
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    NormMode = zmq_sys_crate::ZMQ_NORM_MODE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    NormUnicastNack = zmq_sys_crate::ZMQ_NORM_UNICAST_NACK as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    NormBufferSize = zmq_sys_crate::ZMQ_NORM_BUFFER_SIZE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    NormSegmentSize = zmq_sys_crate::ZMQ_NORM_SEGMENT_SIZE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    NormBlockSize = zmq_sys_crate::ZMQ_NORM_BLOCK_SIZE as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    NormNumnParity = zmq_sys_crate::ZMQ_NORM_NUM_PARITY as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
    NormNumnAutoParity = zmq_sys_crate::ZMQ_NORM_NUM_AUTOPARITY as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(all(feature = "draft-api", zmq_have_norm)))]
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
    /// The [`events()`] option shall retrieve the event state for the specified `Socket`. The
    /// returned value is a bit mask constructed by OR’ing a combination of the following event
    /// flags:
    ///
    /// * [`POLL_IN`] Indicates that at least one message may be received from the specified socket
    ///   without blocking.
    /// * [`POLL_OUT`] Indicates that at least one message may be sent to the specified socket
    ///   without blocking.
    ///
    /// The combination of a file descriptor returned by the [`FileDescriptor`] option being ready
    /// for reading but no actual events returned by a subsequent retrieval of the [`events()`]
    /// option is valid; applications should simply ignore this case and restart their polling
    /// operation/event loop.
    ///
    /// | Default value | Applicable socket types         |
    /// | :-----------: | :-----------------------------: |
    /// | None          | all                             |
    ///
    /// [`events()`]: #method.events
    /// [`POLL_IN`]: PollEvents::POLL_IN
    /// [`POLL_OUT`]: PollEvents::POLL_OUT
    /// [`FileDescriptor`]: SocketOption::FileDescriptor
    pub fn events(&self) -> ZmqResult<PollEvents> {
        self.get_sockopt_int::<i16>(SocketOption::Events)
            .map(PollEvents::from_bits_truncate)
    }

    /// # Disable GSSAPI encryption `ZMQ_GSSAPI_PLAINTEXT`
    ///
    /// Defines whether communications on the socket will be encrypted. A value of `true` means
    /// that communications will be plaintext. A value of `false` means communications will be
    /// encrypted.
    ///
    /// | Default value | Applicable socket types               |
    /// | :-----------: | :-----------------------------------: |
    /// | false         | all, when using TCP or IPC transports |
    #[doc(cfg(zmq_have_gssapi))]
    pub fn set_gssapi_plaintext(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::GssApiPlainText, value)
    }

    /// # Retrieve GSSAPI plaintext or encrypted status `ZMQ_GSSAPI_PLAINTEXT`
    ///
    /// Returns the [`gssapi_plaintext()`] option, if any, previously set on the socket. A value of
    /// `true` means that communications will be plaintext. A value of `false` means communications
    /// will be encrypted.
    ///
    /// | Default value | Applicable socket types               |
    /// | :-----------: | :-----------------------------------: |
    /// | false         | all, when using TCP or IPC transports |
    ///
    /// [`gssapi_plaintext()`]: #method.gssapi_plaintext
    #[doc(cfg(zmq_have_gssapi))]
    pub fn gssapi_plaintext(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::GssApiPlainText)
    }

    /// # Set name type of service principal `ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE`
    ///
    /// Sets the name type of the GSSAPI service principal. A value of [`NtHostbased`] means the
    /// name specified with [`GssApiServicePrincipal`] is interpreted as a host based name. A value
    /// of [`NtUsername`] means it is interpreted as a local user name. A value of
    /// [`NtKrb5Principal`] means it is interpreted as an unparsed principal name string (valid
    /// only with the krb5 GSSAPI mechanism).
    ///
    /// | Default value   | Applicable socket types               |
    /// | :-------------: | :-----------------------------------: |
    /// | [`NtHostbased`] | all, when using TCP or IPC transports |
    ///
    /// [`GssApiServicePrincipal`]: SocketOption::GssApiServicePrincipal
    /// [`NtHostbased`]: GssApiNametype::NtHostbased
    /// [`NtUsername`]: GssApiNametype::NtUsername
    /// [`NtKrb5Principal`]: GssApiNametype::NtKrb5Principal
    #[doc(cfg(zmq_have_gssapi))]
    pub fn set_gssapi_service_principal_nametype(&self, value: GssApiNametype) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::GssApiServicePrincipalNametype, value as i32)
    }

    /// # Retrieve nametype for service principal `ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE`
    ///
    /// Returns the [`GssApiServicePrincipalNametype`] option, if any, previously set on the socket.
    /// A value of [`NtHostbased`] means the name specified with [`GssApiServicePrincipal`] is
    /// interpreted as a host based name. A value of [`NtUsername`] means it is interpreted as a
    /// local user name. A value of [`NtKrb5Principal`] means it is interpreted as an unparsed
    /// principal name string (valid only with the krb5 GSSAPI mechanism).
    ///
    /// | Default value   | Applicable socket types               |
    /// | :-------------: | :-----------------------------------: |
    /// | [`NtHostbased`] | all, when using TCP or IPC transports |
    ///
    /// [`GssApiServicePrincipalNametype`]: SocketOption::GssApiServicePrincipalNametype
    /// [`GssApiServicePrincipal`]: SocketOption::GssApiServicePrincipal
    /// [`NtHostbased`]: GssApiNametype::NtHostbased
    /// [`NtUsername`]: GssApiNametype::NtUsername
    /// [`NtKrb5Principal`]: GssApiNametype::NtKrb5Principal
    #[doc(cfg(zmq_have_gssapi))]
    pub fn gssapi_service_principal_nametype(&self) -> ZmqResult<GssApiNametype> {
        self.get_sockopt_int::<i32>(SocketOption::GssApiServicePrincipalNametype)
            .and_then(GssApiNametype::try_from)
    }

    /// # Set name of GSSAPI principal `ZMQ_GSSAPI_PRINCIPAL`
    ///
    /// Sets the name of the principal for whom GSSAPI credentials should be acquired.
    ///
    /// | Default value   | Applicable socket types              |
    /// | :-------------: | :----------------------------------: |
    /// | not set         | all, when using TCP or IPC transport |
    #[doc(cfg(zmq_have_gssapi))]
    pub fn set_gssapi_principal<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::GssApiPrincipal, value.as_ref())
    }

    /// # Retrieve the name of the GSSAPI principal `ZMQ_GSSAPI_PRINCIPAL`
    ///
    /// The [`gssapi_principal()`] option shall retrieve the principal name set for the GSSAPI
    /// security mechanism. The returned value shall be a NULL-terminated string and MAY be empty.
    /// The returned size SHALL include the terminating null byte.
    ///
    /// | Default value   | Applicable socket types              |
    /// | :-------------: | :----------------------------------: |
    /// | not set         | all, when using TCP or IPC transport |
    ///
    /// [`gssapi_principal()`]: #method.gssapi_principal
    #[doc(cfg(zmq_have_gssapi))]
    pub fn gssapi_principal(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::GssApiPrincipal)
    }

    /// # Set name type of principal `ZMQ_GSSAPI_PRINCIPAL_NAMETYPE`
    ///
    /// Sets the name type of the GSSAPI principal. A value of [`NtHostbased`] means the name
    /// specified with [`GssApiPrincipal`] is interpreted as a host based name. A value of
    /// [`NtUsername`] means it is interpreted as a local user name. A value of [`NtKrb5Principal`]
    /// means it is interpreted as an unparsed principal name string (valid only with the krb5
    /// GSSAPI mechanism).
    ///
    /// | Default value   | Applicable socket types               |
    /// | :-------------: | :-----------------------------------: |
    /// | [`NtHostbased`] | all, when using TCP or IPC transports |
    ///
    /// [`GssApiPrincipal`]: SocketOption::GssApiPrincipal
    /// [`NtHostbased`]: GssApiNametype::NtHostbased
    /// [`NtUsername`]: GssApiNametype::NtUsername
    /// [`NtKrb5Principal`]: GssApiNametype::NtKrb5Principal
    #[doc(cfg(zmq_have_gssapi))]
    pub fn set_gssapi_principal_nametype(&self, value: GssApiNametype) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::GssApiPrincipalNametype, value as i32)
    }

    /// # Retrieve nametype for service principal `ZMQ_GSSAPI_PRINCIPAL_NAMETYPE`
    ///
    /// Returns the [`GssApiPrincipalNametype`] option, if any, previously set on the socket. A
    /// value of [`NtHostbased`] means the name specified with [`GssApiPrincipal`] is
    /// interpreted as a host based name. A value of [`NtUsername`] means it is interpreted as a
    /// local user name. A value of [`NtKrb5Principal`] means it is interpreted as an unparsed
    /// principal name string (valid only with the krb5 GSSAPI mechanism).
    ///
    /// | Default value   | Applicable socket types               |
    /// | :-------------: | :-----------------------------------: |
    /// | [`NtHostbased`] | all, when using TCP or IPC transports |
    ///
    /// [`GssApiPrincipalNametype`]: SocketOption::GssApiPrincipalNametype
    /// [`GssApiPrincipal`]: SocketOption::GssApiPrincipal
    /// [`NtHostbased`]: GssApiNametype::NtHostbased
    /// [`NtUsername`]: GssApiNametype::NtUsername
    /// [`NtKrb5Principal`]: GssApiNametype::NtKrb5Principal
    #[doc(cfg(zmq_have_gssapi))]
    pub fn gssapi_principal_nametype(&self) -> ZmqResult<GssApiNametype> {
        self.get_sockopt_int::<i32>(SocketOption::GssApiPrincipalNametype)
            .and_then(GssApiNametype::try_from)
    }

    /// # Set maximum handshake interval `ZMQ_HANDSHAKE_IVL`
    ///
    /// The [`HandshakeInterval`] option shall set the maximum handshake interval for the `Socket`.
    /// Handshaking is the exchange of socket configuration information (socket type, routing id,
    /// security) that occurs when a connection is first opened, only for connection-oriented
    /// transports. If handshaking does not complete within the configured time, the connection
    /// shall be closed. The value `0` means no handshake time limit.
    ///
    /// | Default value | Applicable socket types                                     |
    /// | :-----------: | :---------------------------------------------------------: |
    /// | 30_000 ms     | all but [`Stream`], only for connection-oriented transports |
    ///
    /// [`HandshakeInterval`]: SocketOption::HandshakeInterval
    /// [`Stream`]: StreamSocket
    pub fn set_handshake_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HandshakeInterval, value)
    }

    /// # Retrieve maximum handshake interval `ZMQ_HANDSHAKE_IVL`
    ///
    /// The [`HandshakeInterval`] option shall retrieve the maximum handshake interval for the
    /// `Socket`. Handshaking is the exchange of socket configuration information (socket type,
    /// routing id, security) that occurs when a connection is first opened, only for
    /// connection-oriented transports. If handshaking does not complete within the configured
    /// time, the connection shall be closed. The value `0` means no handshake time limit.
    ///
    /// | Default value | Applicable socket types                                     |
    /// | :-----------: | :---------------------------------------------------------: |
    /// | 30_000 ms     | all but [`Stream`], only for connection-oriented transports |
    ///
    /// [`HandshakeInterval`]: SocketOption::HandshakeInterval
    /// [`Stream`]: StreamSocket
    pub fn handshake_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::HandshakeInterval)
    }

    /// # Set interval between sending ZMTP heartbeats `ZMQ_HEARTBEAT_IVL`
    ///
    /// The [`HeartbeatInterval`] option shall set the interval between sending ZMTP heartbeats for
    /// the `Socket`. If this option is set and is greater than `0`, then a `PING` ZMTP command
    /// will be sent every [`heartbeat_interval()`] milliseconds.
    ///
    /// | Default value | Applicable socket types                        |
    /// | :-----------: | :--------------------------------------------: |
    /// | ß ms          | all, when using connection-oriented transports |
    ///
    /// [`HeartbeatInterval`]: SocketOption::HeartbeatInterval
    /// [`heartbeat_interval()`]: #method.heartbeat_interval
    pub fn set_heartbeat_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HeartbeatInterval, value)
    }

    /// # Set timeout for ZMTP heartbeats `ZMQ_HEARTBEAT_TIMEOUT`
    ///
    /// The [`HeartbeatTimeout`] option shall set how long to wait before timing-out a connection
    /// after sending a `PING` ZMTP command and not receiving any traffic. This option is only
    /// valid if [`HeartbeatInterval`] is also set, and is greater than `0`. The connection will
    /// time out if there is no traffic received after sending the `PING` command, but the received
    /// traffic does not have to be a `PONG` command - any received traffic will cancel the timeout.
    ///
    /// | Default value                                     | Applicable socket types                        |
    /// | :-----------------------------------------------: | :--------------------------------------------: |
    /// | 0 ms normally, [`HeartbeatInterval`] if it is set | all, when using connection-oriented transports |
    ///
    /// [`HeartbeatTimeout`]: SocketOption::HeartbeatTimeout
    /// [`HeartbeatInterval`]: SocketOption::HeartbeatInterval
    pub fn set_heartbeat_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HeartbeatTimeout, value)
    }

    /// # Set the TTL value for ZMTP heartbeats `ZMQ_HEARTBEAT_TTL`
    ///
    /// The [`HeartbeatTimeToLive`] option shall set the timeout on the remote peer for ZMTP
    /// heartbeats. If this option is greater than 0, the remote side shall time out the connection
    /// if it does not receive any more traffic within the TTL period. This option does not have
    /// any effect if [`HeartbeatInterval`] is not set or is `0`. Internally, this value is rounded
    /// down to the nearest decisecond, any value less than `100` will have no effect.
    ///
    /// | Default value                           | Applicable socket types                        |
    /// | :-------------------------------------: | :--------------------------------------------: |
    /// | 6_553_599 (which is 2^16-1 deciseconds) | all, when using connection-oriented transports |
    ///
    /// [`HeartbeatTimeToLive`]: SocketOption::HeartbeatTimeToLive
    /// [`HeartbeatInterval`]: SocketOption::HeartbeatInterval
    pub fn set_heartbeat_timetolive(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::HeartbeatTimeToLive, value)
    }

    /// # Queue messages only to completed connections `ZMQ_IMMEDIATE`
    ///
    /// By default queues will fill on outgoing connections even if the connection has not
    /// completed. This can lead to "lost" messages on sockets with round-robin routing
    /// ([`Request`], [`Push`], [`Dealer`]). If this option is set to `true`, messages shall be
    /// queued only to completed connections. This will cause the socket to block if there are no
    /// other connections, but will prevent queues from filling on pipes awaiting connection.
    ///
    /// | Default value | Applicable socket types                        |
    /// | :-----------: | :--------------------------------------------: |
    /// | false         | all, when using connection-oriented transports |
    ///
    /// [`Request`]: RequestSocket
    /// [`Push`]: PushSocket
    /// [`Dealer`]: DealerSocket
    pub fn set_immediate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::Immediate, value)
    }

    /// # Retrieve attach-on-connect value `ZMQ_IMMEDIATE`
    ///
    /// Retrieve the state of the attach on connect value. If set to 1`true`, will delay the
    /// attachment of a pipe on connect until the underlying connection has completed. This will
    /// cause the socket to block if there are no other connections, but will prevent queues from
    /// filling on pipes awaiting connection.
    ///
    /// | Default value | Applicable socket types                        |
    /// | :-----------: | :--------------------------------------------: |
    /// | false         | all, when using connection-oriented transports |
    pub fn immediate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::Immediate)
    }

    /// # Enable IPv6 on socket `ZMQ_IPV6`
    ///
    /// Set the IPv6 option for the socket. A value of `true` means IPv6 is enabled on the socket,
    /// while `false` means the socket will use only IPv4. When IPv6 is enabled the socket will
    /// connect to, or accept connections from, both IPv4 and IPv6 hosts.
    ///
    /// | Default value | Applicable socket types         |
    /// | :-----------: | :-----------------------------: |
    /// | false         | all, when using TCP transports. |
    pub fn set_ipv6(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::IPv6, value)
    }

    /// # Retrieve IPv6 socket status `ZMQ_IPV6`
    ///
    /// Retrieve the IPv6 option for the socket. A value of `true` means IPv6 is enabled on the
    /// socket, while `false` means the socket will use only IPv4. When IPv6 is enabled the socket
    /// will connect to, or accept connections from, both IPv4 and IPv6 hosts.
    ///
    /// | Default value | Applicable socket types         |
    /// | :-----------: | :-----------------------------: |
    /// | false         | all, when using TCP transports. |
    pub fn ipv6(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOption::IPv6)
    }

    /// # Set linger period for socket shutdown `ZMQ_LINGER`
    ///
    /// The [`Linger`] option shall set the linger period for the `Socket`. The linger period
    /// determines how long pending messages which have yet to be sent to a peer shall linger in
    /// memory after a socket is disconnected with [`disconnect()`] or closed, and further affects
    /// the termination of the socket’s context. The following outlines the different behaviours:
    ///
    /// * A value of `-1` specifies an infinite linger period. Pending messages shall not be
    ///   discarded after a call to [`disconnect()`]; attempting to terminate the socket’s context
    ///   shall block until all pending messages have been sent to a peer.
    /// * The value of `0` specifies no linger period. Pending messages shall be discarded
    ///   immediately after a call to [`disconnect()`].
    /// * Positive values specify an upper bound for the linger period in milliseconds. Pending
    ///   messages shall not be discarded after a call to [`disconnect()`]; attempting to terminate
    ///   the socket’s context shall block until either all pending messages have been sent to a
    ///   peer, or the linger period expires, after which any pending messages shall be discarded.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1 (infinite) | all                     |
    ///
    /// [`disconnect()`]: #method.disconnect
    /// [`Linger`]: SocketOption::Linger
    pub fn set_linger(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Linger, value)
    }

    /// # Retrieve linger period for socket shutdown `ZMQ_LINGER`
    ///
    /// The [`Linger`] option shall retrieve the linger period for the `Socket`. The linger period
    /// determines how long pending messages which have yet to be sent to a peer shall linger in
    /// memory after a socket is closed, and further affects the termination of the socket’s
    /// context. The following outlines the different behaviours:
    ///
    /// * A value of `-1` specifies an infinite linger period. Pending messages shall not be
    ///   discarded after a call to [`disconnect()`]; attempting to terminate the socket’s context
    ///   shall block until all pending messages have been sent to a peer.
    /// * The value of `0` specifies no linger period. Pending messages shall be discarded
    ///   immediately after a call to [`disconnect()`].
    /// * Positive values specify an upper bound for the linger period in milliseconds. Pending
    ///   messages shall not be discarded after a call to [`disconnect()`]; attempting to terminate
    ///   the socket’s context shall block until either all pending messages have been sent to a
    ///   peer, or the linger period expires, after which any pending messages shall be discarded.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1 (infinite) | all                     |
    ///
    /// [`disconnect()`]: #method.disconnect
    /// [`Linger`]: SocketOption::Linger
    pub fn linger(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::Linger)
    }

    /// # Retrieve the last endpoint set `ZMQ_LAST_ENDPOINT`
    ///
    /// The [`LastEndpoint`] option shall retrieve the last endpoint bound for TCP and IPC
    /// transports. The returned value will be a string in the form of a ZMQ DSN. Note that if the
    /// TCP host is INADDR_ANY, indicated by a *, then the returned address will be `0.0.0.0`
    /// (for IPv4). Note: not supported on GNU/Hurd with IPC due to non-working getsockname().
    ///
    /// | Default value | Applicable socket types                 |
    /// | :-----------: | :-------------------------------------: |
    /// | None          | all, when binding TCP or IPC transports |
    ///
    /// [`LastEndpoint`]: SocketOption::LastEndpoint
    pub fn last_endpoint(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::LastEndpoint)
    }

    /// # Maximum acceptable inbound message size `ZMQ_MAXMSGSIZE`
    ///
    /// Limits the size of the inbound message. If a peer sends a message larger than
    /// [`MaxMessageSize`] it is disconnected. Value of `-1` means 'no limit'.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1 (bytes)    | all                     |
    ///
    /// [`MaxMessageSize`]: SocketOption::MaxMessageSize
    pub fn set_max_message_size(&self, value: i64) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::MaxMessageSize, value)
    }

    /// # Maximum acceptable inbound message size `ZMQ_MAXMSGSIZE`
    ///
    /// The option shall retrieve limit for the inbound messages. If a peer sends a message larger
    /// than [`MaxMessageSize`] it is disconnected. Value of `-1` means 'no limit'.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1 (bytes)    | all                     |
    ///
    /// [`MaxMessageSize`]: SocketOption::MaxMessageSize
    pub fn max_message_size(&self) -> ZmqResult<i64> {
        self.get_sockopt_int(SocketOption::MaxMessageSize)
    }

    /// # Set the security mechanism `ZMQ_MECHANISM`
    ///
    /// Sets the security [`Mechanism`] option for the socket based on the provided
    /// [`SecurityMechanism`].
    ///
    /// [`Mechanism`]: SocketOption::Mechanism
    /// [ SecurityMechanism`]: SecurityMechanism
    pub fn set_security_mechanism(&self, security: SecurityMechanism) -> ZmqResult<()> {
        security.apply(self)
    }

    /// # Retrieve current security mechanism `ZMQ_MECHANISM`
    ///
    /// The [`Mechanism`] option shall retrieve the current security mechanism for the socket.
    ///
    /// [`Mechanism`]: SocketOption::Mechanism
    pub fn security_mechanism(&self) -> ZmqResult<SecurityMechanism> {
        SecurityMechanism::try_from(self)
    }

    /// # Maximum network hops for multicast packets `ZMQ_MULTICAST_HOPS`
    ///
    /// Sets the time-to-live field in every multicast packet sent from this socket. The default
    /// is `1` which means that the multicast packets don’t leave the local network.
    ///
    /// | Default value | Applicable socket types              |
    /// | :-----------: | :----------------------------------: |
    /// | 1             | all, when using multicast transports |
    pub fn set_multicast_hops(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::MulticastHops, value)
    }

    /// # Maximum network hops for multicast packets `ZMQ_MULTICAST_HOPS`
    ///
    /// The option shall retrieve time-to-live used for outbound multicast packets. The default of
    /// `1?  means that the multicast packets don’t leave the local network.
    ///
    /// | Default value | Applicable socket types              |
    /// | :-----------: | :----------------------------------: |
    /// | 1             | all, when using multicast transports |
    pub fn multicast_hops(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::MulticastHops)
    }

    /// # Set multicast data rate `ZMQ_RATE`
    ///
    /// The [`Rate`] option shall set the maximum send or receive data rate for multicast
    /// transports such as `pgm` using the `Socket`.
    ///
    /// | Default value      | Applicable socket types              |
    /// | :----------------: | :----------------------------------: |
    /// | 100 (kilobits/sec) | all, when using multicast transports |
    ///
    /// [`Rate`]: SocketOption::Rate
    pub fn set_rate(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::Rate, value)
    }

    /// # Retrieve multicast data rate `ZMQ_RATE`
    ///
    /// The [`Rate`] option shall retrieve the maximum send or receive data rate for multicast
    /// transports using the `Socket`.
    ///
    /// | Default value      | Applicable socket types              |
    /// | :----------------: | :----------------------------------: |
    /// | 100 (kilobits/sec) | all, when using multicast transports |
    ///
    /// [`Rate`]: SocketOption::Rate
    pub fn rate(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::Rate)
    }

    /// # Set kernel receive buffer size `ZMQ_RCVBUF`
    /// The [`ReceiveBuffer`] option shall set the underlying kernel receive buffer size for the
    /// `Socket` to the specified size in bytes. A value of `-1` means leave the OS default
    /// unchanged. For details refer to your operating system documentation for the  SO_RCVBUF`
    /// socket option.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1            | all                     |
    ///
    /// [`ReceiveBuffer`]: SocketOption::ReceiveBuffer
    pub fn set_receive_buffer(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReceiveBuffer, value)
    }

    /// # Retrieve kernel receive buffer size `ZMQ_RCVBUF`
    ///
    /// The [`ReceiveBuffer`] option shall retrieve the underlying kernel receive buffer size for
    /// the `Socket`. For details refer to your operating system documentation for the `SO_RCVBUF`
    /// socket option.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1            | all                     |
    ///
    /// [`ReceiveBuffer`]: SocketOption::ReceiveBuffer
    pub fn receive_buffer(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReceiveBuffer)
    }

    /// # Set high water mark for inbound messages `ZMQ_RCVHWM`
    ///
    /// The [`ReceiveHighWatermark`] option shall set the high water mark for inbound messages on
    /// the `Ssocket`. The high water mark is a hard limit on the maximum number of outstanding
    /// messages 0MQ shall queue in memory for any single peer that the specified `Socket` is
    /// communicating with. A value of zero means no limit.
    ///
    /// If this limit has been reached the socket shall enter an exceptional state and depending on
    /// the socket type, 0MQ shall take appropriate action such as blocking or dropping sent
    /// messages. Refer to the individual socket descriptions for details on the exact action taken
    /// for each socket type.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 1000          | all                     |
    ///
    /// [`ReceiveHighWatermark`]: SocketOption::ReceiveHighWatermark
    pub fn set_receive_highwater_mark(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReceiveHighWatermark, value)
    }

    /// # Retrieve high water mark for inbound messages `ZMQ_RCVHWM`
    ///
    /// The [`ReceiveHighWatermark`] option shall return the high water mark for inbound messages
    /// on the `Socket`. The high water mark is a hard limit on the maximum number of outstanding
    /// messages 0MQ shall queue in memory for any single peer that the specified `Socket` is
    /// communicating with. A value of zero means no limit.
    ///
    /// If this limit has been reached the socket shall enter an exceptional state and depending on
    /// the socket type, 0MQ shall take appropriate action such as blocking or dropping sent
    /// messages. Refer to the individual socket descriptions for details on the exact action taken
    /// for each socket type.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 1000          | all                     |
    ///
    /// [`ReceiveHighWatermark`]: SocketOption::ReceiveHighWatermark
    pub fn receive_highwater_mark(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReceiveHighWatermark)
    }

    ///# Maximum time before a recv operation returns with [`Again`] `ZMQ_RCVTIMEO`
    ///
    /// Sets the timeout for receive operation on the socket. If the value is 0, [`recv_msg()`]
    /// will return immediately, with a [`Again`] error if there is no message to receive. If the
    /// value is `-1`, it will block until a message is available. For all other values, it will
    /// wait for a message for that amount of time before returning with an [`Again`] error.
    ///
    /// | Default value    | Applicable socket types |
    /// | :--------------: | :---------------------: |
    /// | -1 ms (infinite) | all                     |
    ///
    /// [`Again`]: crate::ZmqError::Again
    /// [`recv_msg()`]: #method.recv_msg
    pub fn set_receive_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReceiveTimeout, value)
    }

    /// # Maximum time before a socket operation returns with [`Again`] `ZMQ_RCVTIMEO`
    ///
    /// Retrieve the timeout for recv operation on the socket. If the value is `0`, [`recv_msg()`]
    /// will return immediately, with a [`Again`] error if there is no message to receive. If the
    /// value is `-1`, it will block until a message is available. For all other values, it will
    /// wait for a message for that amount of time before returning with an [`Again`] error.
    ///
    /// | Default value    | Applicable socket types |
    /// | :--------------: | :---------------------: |
    /// | -1 ms (infinite) | all                     |
    ///
    /// [`Again`]: crate::ZmqError::Again
    /// [`recv_msg()`]: #method.recv_msg
    pub fn receive_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReceiveTimeout)
    }

    /// # Set reconnection interval `ZMQ_RECONNECT_IVL`
    /// The [`ReconnectInterval`] option shall set the initial reconnection interval for the
    /// `Socket`. The reconnection interval is the period 0MQ shall wait between attempts to
    /// reconnect disconnected peers when using connection-oriented transports. The value `-1?
    /// means no reconnection.
    ///
    /// | Default value | Applicable socket types                      |
    /// | :-----------: | :------------------------------------------: |
    /// | 100 ms        | all, only for connection-oriented transports |
    ///
    /// [`ReconnectInterval`]: SocketOption::ReconnectInterval
    pub fn set_reconnect_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReconnectInterval, value)
    }

    /// # Retrieve reconnection interval `ZMQ_RECONNECT_IVL`
    ///
    /// The [`ReconnectInterval`] option shall retrieve the initial reconnection interval for the
    /// `Socket`. The reconnection interval is the period 0MQ shall wait between attempts to
    /// reconnect disconnected peers when using connection-oriented transports. The value `-1`
    /// means no reconnection.
    ///
    /// | Default value | Applicable socket types                      |
    /// | :-----------: | :------------------------------------------: |
    /// | 100 ms        | all, only for connection-oriented transports |
    ///
    /// [`ReconnectInterval`]: SocketOption::ReconnectInterval
    pub fn reconnect_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReconnectInterval)
    }

    /// # Set max reconnection interval `ZMQ_RECONNECT_IVL_MAX`
    ///
    /// The [`ReconnectIntervalMax`] option shall set the max reconnection interval for the
    /// `Socket`. 0MQ shall wait at most the configured interval between reconnection attempts. The
    /// interval grows exponentionally (i.e.: it is doubled) with each attempt until it reaches
    /// [`ReconnectIntervalMax`]. Default value means that the reconnect interval is based
    /// exclusively on [`ReconnectInterval`] and no exponential backoff is performed.
    ///
    /// | Default value                             | Applicable socket types                      |
    /// | :---------------------------------------: | :------------------------------------------: |
    /// | 0 ms ([`ReconnectInterval`] will be used) | all, only for connection-oriented transports |
    ///
    /// [`ReconnectIntervalMax`]: SocketOption::ReconnectIntervalMax
    /// [`ReconnectInterval`]: SocketOption::ReconnectInterval
    pub fn set_reconnect_interval_max(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReconnectIntervalMax, value)
    }

    /// # Retrieve max reconnection interval `ZMQ_RECONNECT_IVL_MAX`
    ///
    /// The [`ReconnectIntervalMax`] option shall retrieve the max reconnection interval for the
    /// `Socket`. 0MQ shall wait at most the configured interval between reconnection attempts. The
    /// interval grows exponentionally (i.e.: it is doubled) with each attempt until it reaches
    /// [`ReconnectIntervalMax`]. Default value means that the reconnect interval is based
    /// exclusively on [`ReconnectInterval`] and no exponential backoff is performed.
    ///
    /// | Default value                             | Applicable socket types                      |
    /// | :---------------------------------------: | :------------------------------------------: |
    /// | 0 ms ([`ReconnectInterval`] will be used) | all, only for connection-oriented transports |
    ///
    /// [`ReconnectIntervalMax`]: SocketOption::ReconnectIntervalMax
    /// [`ReconnectInterval`]: SocketOption::ReconnectInterval
    pub fn reconnect_interval_max(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::ReconnectIntervalMax)
    }

    /// # Set condition where reconnection will stop `ZMQ_RECONNECT_STOP`
    ///
    /// The [`ReconnectStop`] option shall set the conditions under which automatic reconnection
    /// will stop. This can be useful when a process binds to a wild-card port, where the OS
    /// supplies an ephemeral port.
    ///
    /// | Default value | Applicable socket types                      |
    /// | :-----------: | :------------------------------------------: |
    /// | 0             | all, only for connection-oriented transports |
    ///
    /// [`ReconnectStop`]: SocketOption::ReconnectStop
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_reconnect_stop(&self, value: ReconnectStop) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::ReconnectStop, value.bits())
    }

    /// # ZMQ_RECONNECT_STOP: Retrieve condition where reconnection will stop
    ///
    /// The [`ReconnectStop`] option shall retrieve the conditions under which automatic
    /// reconnection will stop.
    ///
    /// | Default value | Applicable socket types                      |
    /// | :-----------: | :------------------------------------------: |
    /// | 0             | all, only for connection-oriented transports |
    ///
    /// [`ReconnectStop`]: SocketOption::ReconnectStop
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn reconnect_stop(&self) -> ZmqResult<ReconnectStop> {
        self.get_sockopt_int(SocketOption::ReconnectStop)
            .map(ReconnectStop::from_bits_truncate)
    }

    /// # Set multicast recovery interval `ZMQ_RECOVERY_IVL`
    ///
    /// The [`RecoveryInterval`] option shall set the recovery interval for multicast transports
    /// using the `Socket`. The recovery interval determines the maximum time in milliseconds that
    /// a receiver can be absent from a multicast group before unrecoverable data loss will occur.
    ///
    /// | Default value | Applicable socket types              |
    /// | :-----------: | :----------------------------------: |
    /// | 10_000 ms     | all, when using multicast transports |
    ///
    /// [`RecoveryInterval`]: SocketOption::RecoveryInterval
    pub fn set_recovery_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::RecoveryInterval, value)
    }

    /// # Get multicast recovery interval `ZMQ_RECOVERY_IVL`
    ///
    /// The [`RecoveryInterval`] option shall retrieve the recovery interval for multicast
    /// transports using the `Socket`. The recovery interval determines the maximum time in
    /// milliseconds that a receiver can be absent from a multicast group before unrecoverable data
    /// loss will occur.
    ///
    /// | Default value | Applicable socket types              |
    /// | :-----------: | :----------------------------------: |
    /// | 10_000 ms     | all, when using multicast transports |
    ///
    /// [`RecoveryInterval`]: SocketOption::RecoveryInterval
    pub fn recovery_interval(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::RecoveryInterval)
    }

    /// # Set kernel transmit buffer size `ZMQ_SNDBUF`
    ///
    /// The [`SendBuffer`] option shall set the underlying kernel transmit buffer size for the
    /// `Socket` to the specified size in bytes. A value of `-1` means leave the OS default
    /// unchanged. For details please refer to your operating system documentation for the
    /// `SO_SNDBUF` socket option.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1            | all                     |
    ///
    /// [`SendBuffer`]: SocketOption::SendBuffer
    pub fn set_send_buffer(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::SendBuffer, value)
    }

    /// # Retrieve kernel transmit buffer size `ZMQ_SNDBUF`
    ///
    /// The [`SendBuffer`] option shall retrieve the underlying kernel transmit buffer size for the
    /// `Socket`. For details refer to your operating system documentation for the `SO_SNDBUF`
    /// socket option.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | -1            | all                     |
    ///
    /// [`SendBuffer`]: SocketOption::SendBuffer
    pub fn send_buffer(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::SendBuffer)
    }

    /// # Set high water mark for outbound messages `ZMQ_SNDHWM`
    ///
    /// The [`SendHighWatermark`] option shall set the high water mark for outbound messages on the
    /// `Socket`. The high water mark is a hard limit on the maximum number of outstanding messages
    /// 0MQ shall queue in memory for any single peer that the `Socket` is communicating with. A
    /// value of zero means no limit.
    ///
    /// If this limit has been reached the socket shall enter an exceptional state and depending on
    /// the socket type, 0MQ shall take appropriate action such as blocking or dropping sent
    /// messages. Refer to the individual socket descriptions for details on the exact action taken
    /// for each socket type.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 1000          | all                     |
    ///
    /// [`SendHighWatermark`]: SocketOption::SendHighWatermark
    pub fn set_send_highwater_mark(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::SendHighWatermark, value)
    }

    /// # Retrieves high water mark for outbound messages `ZMQ_SNDHWM`
    ///
    /// The [`SendHighWatermark`] option shall return the high water mark for outbound messages on
    /// the `Socket`. The high water mark is a hard limit on the maximum number of outstanding
    /// messages 0MQ shall queue in memory for any single peer that the `Socket` is communicating
    /// with. A value of zero means no limit.
    ///
    /// If this limit has been reached the socket shall enter an exceptional state and depending on
    /// the socket type, 0MQ shall take appropriate action such as blocking or dropping sent
    /// messages. Refer to the individual socket descriptions for details on the exact action taken
    /// for each socket type.
    ///
    /// | Default value | Applicable socket types |
    /// | :-----------: | :---------------------: |
    /// | 1000          | all                     |
    ///
    /// [`SendHighWatermark`]: SocketOption::SendHighWatermark
    pub fn send_highwater_mark(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::SendHighWatermark)
    }

    /// # Maximum time before a send operation returns with [`Again`] `ZMQ_SNDTIMEO`
    ///
    /// Sets the timeout for send operation on the socket. If the value is `0`, [`send_msg()`] will
    /// return immediately, with a [`Again`] error if the message cannot be sent. If the value is
    /// `-1`, it will block until the message is sent. For all other values, it will try to send
    /// the message for that amount of time before returning with an EAGAIN error.
    ///
    /// | Default value    | Applicable socket types |
    /// | :--------------: | :---------------------: |
    /// | -1 ms (infinite) | all                     |
    ///
    /// [`Again`]: crate::ZmqError::Again
    /// [`send_msg()`]: #method.send_msg
    pub fn set_send_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::SendTimeout, value)
    }

    /// # Maximum time before a socket operation returns with [`Again`] `ZMQ_SNDTIMEO`
    ///
    /// Retrieve the timeout for send operation on the socket. If the value is `0`, [`send_msg()`]
    /// will return immediately, with a [`Again`] error if the message cannot be sent. If the value
    /// is `-1`, it will block until the message is sent. For all other values, it will try to send
    /// the message for that amount of time before returning with an [`Again`] error.
    ///
    /// | Default value    | Applicable socket types |
    /// | :--------------: | :---------------------: |
    /// | -1 ms (infinite) | all                     |
    ///
    /// [`Again`]: crate::ZmqError::Again
    /// [`send_msg()`]: #method.send_msg
    pub fn send_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::SendTimeout)
    }

    /// # Set SOCKS5 proxy address `ZMQ_SOCKS_PROXY`
    ///
    /// Sets the SOCKS5 proxy address that shall be used by the socket for the TCP connection(s).
    /// Supported authentication methods are: no authentication or basic authentication when setup
    /// with [`SocksUsername`]. If the endpoints are domain names instead of addresses they shall
    /// not be resolved and they shall be forwarded unchanged to the SOCKS proxy service in the
    /// client connection request message (address type 0x03 domain name).
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    ///
    /// [`SocksUsername`]: SocketOption::SocksUsername
    pub fn set_socks_proxy<V>(&self, value: Option<V>) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        match value {
            None => self.set_sockopt_bytes(SocketOption::SocksProxy, vec![]),
            Some(ref_value) => self.set_sockopt_string(SocketOption::SocksProxy, ref_value),
        }
    }

    /// # Retrieve SOCKS5 proxy address `ZMQ_SOCKS_PROXY`
    ///
    /// The [`SocksProxy`] option shall retrieve the SOCKS5 proxy address in string format. The
    /// returned value MAY be empty.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    ///
    /// [`SocksProxy`]: SocketOption::SocksProxy
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn socks_proxy(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOption::SocksProxy)
    }

    /// # Set SOCKS username and select basic authentication `ZMQ_SOCKS_USERNAME`
    ///
    /// Sets the username for authenticated connection to the SOCKS5 proxy. If you set this to a
    /// non-null and non-empty value, the authentication method used for the SOCKS5 connection
    /// shall be basic authentication. In this case, use [`set_socks_password()`] option in order
    /// to set the password. If you set this to a null value or empty value, the authentication
    /// method shall be no authentication, the default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    ///
    /// [`set_socks_password()`]: #method.set_socks_password
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_socks_username<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::SocksUsername, value.as_ref())
    }

    /// # Set SOCKS basic authentication password `ZMQ_SOCKS_PASSWORD`
    ///
    /// Sets the password for authenticating to the SOCKS5 proxy server. This is used only when the
    /// SOCK5 authentication method has been set to basic authentication through the
    /// [`set_socks_username()`] option. Setting this to a null value (the default) is equivalent
    /// to an empty password string.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | not set       | all, when using TCP transport |
    ///
    /// [set_socks_username()`]: #method.set_socks_username
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_socks_password<V>(&self, value: V) -> ZmqResult<()>
    where
        V: AsRef<str>,
    {
        self.set_sockopt_string(SocketOption::SocksPassword, value.as_ref())
    }

    /// # Override `SO_KEEPALIVE` socket option `ZMQ_TCP_KEEPALIVE`
    ///
    /// Override `SO_KEEPALIVE` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn set_tcp_keepalive(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepalive, value)
    }

    /// # Override `SO_KEEPALIVE` socket option `ZMQ_TCP_KEEPALIVE`
    ///
    /// Override `SO_KEEPALIVE` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn tcp_keepalive(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepalive)
    }

    /// # Override `TCP_KEEPCNT` socket option `ZMQ_TCP_KEEPALIVE_CNT`
    ///
    /// Override `TCP_KEEPCNT` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn set_tcp_keepalive_count(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepaliveCount, value)
    }

    ///  Override `TCP_KEEPCNT` socket option `ZMQ_TCP_KEEPALIVE_CNT`
    ///
    /// Override `TCP_KEEPCNT` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn tcp_keepalive_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepaliveCount)
    }

    /// # Override `TCP_KEEPIDLE` (or `TCP_KEEPALIVE` on some OS) `ZMQ_TCP_KEEPALIVE_IDLE`
    ///
    /// Override `TCP_KEEPIDLE` (or `TCP_KEEPALIVE` on some OS) socket option (where supported by
    /// OS). The default value of `-1` means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn set_tcp_keepalive_idle(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepaliveIdle, value)
    }

    /// # Override `TCP_KEEPIDLE` (or `TCP_KEEPALIVE` on some OS) `ZMQ_TCP_KEEPALIVE_IDLE`
    ///
    /// Override `TCP_KEEPIDLE` (or `TCP_KEEPALIVE` on some OS) socket option (where supported by
    /// OS). The default value of `-1` means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn tcp_keepalive_idle(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOption::TcpKeepaliveIdle)
    }

    /// # Override `TCP_KEEPINTVL` socket option `ZMQ_TCP_KEEPALIVE_INTVL`
    ///
    /// Override `TCP_KEEPINTVL` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
    pub fn set_tcp_keepalive_interval(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOption::TcpKeepaliveInterval, value)
    }

    /// # Override `TCP_KEEPINTVL` socket option `ZMQ_TCP_KEEPALIVE_INTVL`
    ///
    /// Override `TCP_KEEPINTVL` socket option (where supported by OS). The default value of `-1`
    /// means to skip any overrides and leave it to OS default.
    ///
    /// | Default value | Applicable socket types       |
    /// | :-----------: | :---------------------------: |
    /// | -1            | all, when using TCP transport |
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

    /// # accept incoming connections on a socket
    ///
    /// The [`bind()`] function binds the `Socket` to a local `endpoint` and then accepts incoming
    /// connections on that endpoint.
    ///
    /// The `endpoint` is a string consisting of a `transport://` followed by an `address`. The
    /// `transport` specifies the underlying protocol to use. The `address` specifies the
    /// transport-specific address to bind to.
    ///
    /// 0MQ provides the following transports:
    ///
    /// * `tcp` unicast transport using TCP
    /// * `ipc` local inter-process communication transport
    /// * `inproc` local in-process (inter-thread) communication transport
    /// * `pgm`, `epgm` reliable multicast transport using PGM
    /// * `vmci` virtual machine communications interface (VMCI)
    /// * `udp` unreliable unicast and multicast using UDP
    ///
    /// Every 0MQ socket type except [`Pair`] and [`Channel`] supports one-to-many and many-to-one
    /// semantics.
    ///
    /// The `ipc`, `tcp`, `vmci` and `udp` transports accept wildcard addresses.
    ///
    /// [`bind()`]: #method.bind
    /// [`Pair`]: PairSocket
    /// [`Channel`]: ChannelSocket
    pub fn bind<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.bind(endpoint.as_ref())
    }

    /// # Stop accepting connections on a socket
    ///
    /// The [`unbind()`] function shall unbind a socket from the endpoint specified by the
    /// `endpoint` argument.
    ///
    /// Additionally the incoming message queue associated with the endpoint will be discarded.
    /// This means that after unbinding an endpoint it is possible to received messages originating
    /// from that same endpoint if they were already present in the incoming message queue before
    /// unbinding.
    ///
    /// The `endpoint` argument is as described in [`bind()`].
    ///
    /// ## Unbinding wild-card address from a socket
    ///
    /// When wild-card * `endpoint` was used in [`bind()`], the caller should use real `endpoint`
    /// obtained from the [`last_endpoint()`] socket option to unbind this `endpoint` from a socket.
    ///
    /// [`unbind()`]: #method.unbind
    /// [`bind()`]: #method.bind
    /// [`last_endpoint()`]: #method.last_endpoint
    pub fn unbind<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.unbind(endpoint.as_ref())
    }

    /// # create outgoing connection from socket
    ///
    /// The [`connect()`] function connects the `Socket` to an `endpoint` and then accepts incoming
    /// connections on that endpoint.
    ///
    /// The `endpoint` is a string consisting of a `transport://` followed by an `address`. The
    /// `transport` specifies the underlying protocol to use. The `address` specifies the
    /// transport-specific address to connect to.
    ///
    /// 0MQ provides the the following transports:
    ///
    /// * `tcp` unicast transport using TCP
    /// * `ipc` local inter-process communication transport
    /// * `inproc` local in-process (inter-thread) communication transport
    /// * `pgm`, `epgm` reliable multicast transport using PGM
    /// * `vmci` virtual machine communications interface (VMCI)
    /// * `udp` unreliable unicast and multicast using UDP
    ///
    /// Every 0MQ socket type except [`Pair`] and [`Channel`] supports one-to-many and many-to-one
    /// semantics.
    ///
    /// [`connect()`]: #method.connect
    /// [`Pair`]: PairSocket
    /// [`Channel`]: ChannelSocket
    pub fn connect<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.connect(endpoint.as_ref())
    }

    /// # Disconnect a socket from an endpoint
    ///
    /// The [`disconnect()`] function shall disconnect a socket from the endpoint specified by the
    /// `endpoint` argument. Note the actual disconnect system call might occur at a later time.
    ///
    /// Upon disconnection the will also stop receiving messages originating from this endpoint.
    /// Moreover, the socket will no longer be able to queue outgoing messages to this endpoint.
    /// The outgoing message queue associated with the endpoint will be discarded. However, if the
    /// socket’s [`linger()`] period is non-zero, libzmq will still attempt to transmit these discarded
    /// messages, until the linger period expires.
    ///
    /// The `endpoint` argument is as described in [`connect()`]
    ///
    /// [`disconnect()`]: #method.disconnect
    /// [`linger()`]: #method.linger
    /// [`connect()`]: #method.connect
    pub fn disconnect<E>(&self, endpoint: E) -> ZmqResult<()>
    where
        E: AsRef<str>,
    {
        self.socket.disconnect(endpoint.as_ref())
    }

    /// # monitor socket events
    ///
    /// The [`monitor()`] method lets an application thread track socket events (like connects) on
    /// a ZeroMQ socket. Each call to this method creates a [`Monitor`] socket connected to the
    /// socket.
    ///
    /// The `events` argument is a bitmask of the socket events you wish to monitor. To monitor all
    /// events, use the event value [`MonitorFlags::all()`]. NOTE: as new events are added, the
    /// catch-all value will start returning them. An application that relies on a strict and fixed
    /// sequence of events must not use [`MonitorFlags::all()`] in order to guarantee compatibility
    /// with future versions.
    ///
    /// [`monitor()`]: #method.monitor
    /// [`Monitor`]: MonitorSocket
    /// [`MonitorFlags::all()`]: MonitorFlags::all
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

    /// # input/output multiplexing
    ///
    /// Poll this socket for input/output events.
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
        /// For 0MQ sockets, at least one message may be received from the `Socket` without
        /// blocking. For standard sockets this is equivalent to the `POLLIN` flag of the `poll()`
        /// system call and generally means that at least one byte of data may be read from `fd`
        /// without blocking.
        const POLL_IN = 0b0000_0001;
        /// For 0MQ sockets, at least one message may be sent to the `Socket` without blocking. For
        /// standard sockets this is equivalent to the `POLLOUT` flag of the `poll()` system call
        /// and generally means that at least one byte of data may be written to `fd` without
        /// blocking.
        const POLL_OUT = 0b0000_0010;
        /// For standard sockets, this flag is passed to the underlying `poll()` system call and
        /// generally means that some sort of error condition is present on the socket specified by
        /// `fd`. For 0MQ sockets this flag has no effect if set in `events`.
        const POLL_ERR = 0b0000_0100;
        /// For 0MQ sockets this flags is of no use. For standard sockets this means there isurgent data to read. Refer to the POLLPRI flag for more information. For filedescriptor, refer to your use case: as an example, GPIO interrupts are signaled througha POLLPRI event. This flag has no effect on Windows.
        const POLL_PRI = 0b0000_1000;
    }
}

#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReconnectStop(i32);

#[cfg(feature = "draft-api")]
bitflags! {
    impl ReconnectStop: i32 {
        /// The [`CONNECTION_REFUSED`] option will stop reconnection when 0MQ receives the
        /// [`ConnectionRefused`] return code from the connect. This indicates that there is no
        /// code bound to the specified endpoint.
        ///
        /// [`CONNECTION_REFUSED`]: ReconnectStop::CONNECTION_REFUSED
        /// [`ConnectionRefused`]: crate::ZmqError::ConnectionRefused
        const CONNECTION_REFUSED = zmq_sys_crate::ZMQ_RECONNECT_STOP_CONN_REFUSED as i32;
        /// The [`HANDSHAKE_FAILED`] option will stop reconnection if the 0MQ handshake fails. This
        /// can be used to detect and/or prevent errant connection attempts to non-0MQ sockets.
        /// Note that when specifying this option you may also want to set [`HandshakeInterval`]
        /// — the default handshake interval is 30000 (30 seconds), which is typically too large.
        ///
        /// [`HANDSHAKE_FAILED`]: ReconnectStop::HANDSHAKE_FAILED
        /// [`HandshakeInterval`]: SocketOption::HandshakeInterval
        const HANDSHAKE_FAILED = zmq_sys_crate::ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED as i32;
        /// The [`AFTER_DISCONNECT`] option will stop reconnection when `disconnect()` has been
        /// called. This can be useful when the user’s request failed (server not ready), as the
        /// socket does not need to continue to reconnect after user disconnect actively.
        ///
        /// [`AFTER_DISCONNECT`]: ReconnectStop::AFTER_DISCONNECT
        const AFTER_DISCONNECT = zmq_sys_crate::ZMQ_RECONNECT_STOP_AFTER_DISCONNECT as i32;
}}
