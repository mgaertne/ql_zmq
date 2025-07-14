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

mod dealer;
pub(crate) mod monitor;
mod pair;
mod publish;
mod pull;
mod push;
mod reply;
mod request;
mod router;
mod stream;
mod subscribe;
mod xpublish;
mod xsubscribe;

pub use dealer::DealerSocket;
use monitor::Monitor;
pub use monitor::{MonitorSocket, MonitorSocketEvent};
pub use pair::PairSocket;
pub use publish::PublishSocket;
pub use pull::PullSocket;
pub use push::PushSocket;
pub use reply::ReplySocket;
pub use request::RequestSocket;
#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub use router::RouterNotify;
pub use router::RouterSocket;
pub use stream::StreamSocket;
pub use subscribe::SubscribeSocket;
pub use xpublish::XPublishSocket;
pub use xsubscribe::XSubscribeSocket;

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
pub enum SocketOptions {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum Mechanism {
    Null = zmq_sys_crate::ZMQ_NULL as i32,
    Plain = zmq_sys_crate::ZMQ_PLAIN as i32,
    Curve = zmq_sys_crate::ZMQ_CURVE as i32,
    GssApi = zmq_sys_crate::ZMQ_GSSAPI as i32,
    Unsupported(i32),
}

impl From<i32> for Mechanism {
    fn from(value: i32) -> Self {
        match value {
            _ if value == zmq_sys_crate::ZMQ_NULL as i32 => Mechanism::Null,
            _ if value == zmq_sys_crate::ZMQ_PLAIN as i32 => Mechanism::Plain,
            _ if value == zmq_sys_crate::ZMQ_CURVE as i32 => Mechanism::Curve,
            _ if value == zmq_sys_crate::ZMQ_GSSAPI as i32 => Mechanism::GssApi,
            value => Mechanism::Unsupported(value),
        }
    }
}

impl From<SocketOptions> for i32 {
    fn from(value: SocketOptions) -> Self {
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

    pub fn set_sockopt_bytes<O, V>(&self, option: O, value: V) -> ZmqResult<()>
    where
        O: Into<SocketOptions>,
        V: AsRef<[u8]>,
    {
        self.socket
            .set_sockopt_bytes(option.into() as i32, value.as_ref())
    }

    pub fn set_sockopt_string<O, V>(&self, option: O, value: V) -> ZmqResult<()>
    where
        O: Into<SocketOptions>,
        V: AsRef<str>,
    {
        self.socket
            .set_sockopt_string(option.into() as i32, value.as_ref())
    }

    pub fn set_sockopt_int<O, V>(&self, option: O, value: V) -> ZmqResult<()>
    where
        O: Into<SocketOptions>,
        V: PrimInt + Default,
    {
        self.socket.set_sockopt_int(option.into() as i32, value)
    }

    pub fn set_sockopt_bool<O>(&self, option: O, value: bool) -> ZmqResult<()>
    where
        O: Into<SocketOptions>,
    {
        self.socket.set_sockopt_bool(option.into() as i32, value)
    }

    pub fn get_sockopt_bytes<O>(&self, option: O) -> ZmqResult<Vec<u8>>
    where
        O: Into<SocketOptions>,
    {
        self.socket.get_sockopt_bytes(option.into() as i32)
    }

    pub fn get_sockopt_string<O>(&self, option: O) -> ZmqResult<String>
    where
        O: Into<SocketOptions>,
    {
        self.socket.get_sockopt_string(option.into() as i32)
    }

    pub fn get_sockopt_int<O, V>(&self, option: O) -> ZmqResult<V>
    where
        O: Into<SocketOptions>,
        V: PrimInt + Default,
    {
        self.socket.get_sockopt_int(option.into() as i32)
    }

    pub fn get_sockopt_bool<O>(&self, option: O) -> ZmqResult<bool>
    where
        O: Into<SocketOptions>,
    {
        self.socket.get_sockopt_bool(option.into() as i32)
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
    /// [`Affinity`]: SocketOptions::Affinity
    pub fn set_affinity(&self, value: u64) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::Affinity, value)
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
    /// [`Affinity`]: SocketOptions::Affinity
    pub fn affinity(&self) -> ZmqResult<u64> {
        self.get_sockopt_int(SocketOptions::Affinity)
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
    /// [`Backlog`]: SocketOptions::Backlog
    pub fn set_backlog(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::Backlog, value)
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
    /// [`Backlog`]: SocketOptions::Backlog
    pub fn backlog(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::Backlog)
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
        self.set_sockopt_int(SocketOptions::ConnectTimeout, value)
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
        self.get_sockopt_int(SocketOptions::ConnectTimeout)
    }

    /// # Set CURVE public key `ZMQ_CURVE_PUBLICKEY`
    ///
    /// Sets the socket’s long term public key. You must set this on CURVE client sockets, see
    /// zmq_curve. You can provide the key as 32 binary bytes, or as a 40-character string encoded
    /// in the Z85 encoding format and terminated in a null byte. The public key must always be
    /// used with the matching secret key. To generate a public/secret key pair, use
    /// zmq_curve_keypair To derive the public key from a secret key, use zmq_curve_public.
    ///
    /// | Default value | Applicable socket types         |
    /// | :-----------: | :-----------------------------: |
    /// | None          | all, when using TCP transports. |
    pub fn set_curve_publickey<V: AsRef<[u8]>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(SocketOptions::CurvePublicKey, value.as_ref())
    }

    pub fn curve_publickey(&self) -> ZmqResult<Vec<u8>> {
        self.get_sockopt_bytes(SocketOptions::CurvePublicKey)
    }

    pub fn set_curve_secretkey<V: AsRef<[u8]>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(SocketOptions::CurveSecretKey, value.as_ref())
    }

    pub fn curve_secretkey(&self) -> ZmqResult<Vec<u8>> {
        self.get_sockopt_bytes(SocketOptions::CurveSecretKey)
    }

    pub fn set_curve_serverkey<V: AsRef<[u8]>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(SocketOptions::CurveServerKey, value.as_ref())
    }

    pub fn curve_serverkey(&self) -> ZmqResult<Vec<u8>> {
        self.get_sockopt_bytes(SocketOptions::CurveServerKey)
    }

    pub fn events(&self) -> ZmqResult<PollEvents> {
        self.get_sockopt_int::<SocketOptions, i16>(SocketOptions::Events)
            .map(PollEvents::from_bits_truncate)
    }

    pub fn set_gssapi_plaintext(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::GssApiPlainText, value)
    }

    pub fn gssapi_plaintext(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::GssApiPlainText)
    }

    pub fn set_gssapi_principal<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::GssApiPrincipal, value.as_ref())
    }

    pub fn gssapi_principal(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::GssApiPrincipal)
    }

    pub fn set_gssapi_server(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::GssApiServer, value)
    }

    pub fn gssapi_server(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::GssApiServer)
    }

    pub fn set_gssapi_service_principal<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::GssApiServicePrincipal, value.as_ref())
    }

    pub fn gssapi_service_principal(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::GssApiServicePrincipal)
    }

    pub fn set_handshake_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::HandshakeInterval, value)
    }

    pub fn handshake_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::HandshakeInterval)
    }

    pub fn set_heartbeat_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::HeartbeatInterval, value)
    }

    pub fn heartbeat_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::HeartbeatInterval)
    }

    pub fn set_heartbeat_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::HeartbeatTimeout, value)
    }

    pub fn heartbeat_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::HeartbeatTimeout)
    }

    pub fn set_heartbeat_ttl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::HeartbeatTimeToLive, value)
    }

    pub fn heartbeat_ttl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::HeartbeatTimeToLive)
    }

    pub fn set_immediate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::Immediate, value)
    }

    pub fn immediate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::Immediate)
    }

    pub fn set_ipv6(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::IPv6, value)
    }

    pub fn ipv6(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::IPv6)
    }

    pub fn set_linger(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::Linger, value)
    }

    pub fn linger(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::Linger)
    }

    pub fn last_endpoint(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::LastEndpoint)
    }

    pub fn set_maxmsgsize(&self, value: i64) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::MaxMessageSize, value)
    }

    pub fn maxmsgsize(&self) -> ZmqResult<i64> {
        self.get_sockopt_int(SocketOptions::MaxMessageSize)
    }

    pub fn mechanism(&self) -> ZmqResult<Mechanism> {
        self.get_sockopt_int::<SocketOptions, i32>(SocketOptions::Mechanism)
            .map(Mechanism::from)
    }

    pub fn set_multicast_hops(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::MulticastHops, value)
    }

    pub fn multicast_hops(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::MulticastHops)
    }

    pub fn set_plain_password<V: AsRef<str>>(&self, value: Option<V>) -> ZmqResult<()> {
        match value {
            None => self.set_sockopt_bytes(SocketOptions::PlainPassword, vec![]),
            Some(ref_value) => self.set_sockopt_string(SocketOptions::PlainPassword, ref_value),
        }
    }

    pub fn plain_password(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::PlainPassword)
    }

    pub fn set_plain_server(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::PlainServer, value)
    }

    pub fn plain_server(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::PlainServer)
    }

    pub fn set_plain_username<V: AsRef<str>>(&self, value: Option<V>) -> ZmqResult<()> {
        match value {
            None => self.set_sockopt_bytes(SocketOptions::PlainUsername, vec![]),
            Some(ref_value) => self.set_sockopt_string(SocketOptions::PlainUsername, ref_value),
        }
    }

    pub fn plain_username(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::PlainUsername)
    }

    pub fn set_probe_router(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::ProbeRouter, value)
    }

    pub fn set_rate(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::Rate, value)
    }

    pub fn rate(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::Rate)
    }

    pub fn set_rcvbuf(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::ReceiveBuffer, value)
    }

    pub fn rcvbuf(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::ReceiveBuffer)
    }

    pub fn set_rcvhwm(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::ReceiveHighWatermark, value)
    }

    pub fn rcvhwm(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::ReceiveHighWatermark)
    }

    pub fn set_rcvtimeo(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::ReceiveTimeout, value)
    }

    pub fn rcvtimeo(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::ReceiveTimeout)
    }

    pub fn set_reconnect_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::ReconnectInterval, value)
    }

    pub fn reconnect_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::ReconnectInterval)
    }

    pub fn set_reconnect_ivl_max(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::ReconnectIntervalMax, value)
    }

    pub fn reconnect_ivl_max(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::ReconnectIntervalMax)
    }

    pub fn set_recovery_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::RecoveryInterval, value)
    }

    pub fn recovery_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::RecoveryInterval)
    }

    pub fn set_req_correlate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::RequestCorrelate, value)
    }

    pub fn set_req_relaxed(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::RequestRelaxed, value)
    }

    pub fn set_sndbuf(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::SendBuffer, value)
    }

    pub fn sndbuf(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::SendBuffer)
    }

    pub fn set_sndhwm(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::SendHighWatermark, value)
    }

    pub fn sndhwm(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::SendHighWatermark)
    }

    pub fn set_sndtimeo(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::SendTimeout, value)
    }

    pub fn sndtimeo(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::SendTimeout)
    }

    pub fn set_socks_proxy<V: AsRef<str>>(&self, value: Option<V>) -> ZmqResult<()> {
        match value {
            None => self.set_sockopt_bytes(SocketOptions::SocksProxy, vec![]),
            Some(ref_value) => self.set_sockopt_string(SocketOptions::SocksProxy, ref_value),
        }
    }

    pub fn socks_proxy(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::SocksProxy)
    }

    pub fn set_tcp_keepalive(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::TcpKeepalive, value)
    }

    pub fn tcp_keepalive(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::TcpKeepalive)
    }

    pub fn set_tcp_keepalive_cnt(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::TcpKeepaliveCount, value)
    }

    pub fn tcp_keepalive_cnt(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::TcpKeepaliveCount)
    }

    pub fn set_tcp_keepalive_idle(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::TcpKeepaliveIdle, value)
    }

    pub fn tcp_keepalive_idle(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::TcpKeepaliveIdle)
    }

    pub fn set_tcp_keepalive_intvl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::TcpKeepaliveInterval, value)
    }

    pub fn tcp_keepalive_intvl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::TcpKeepaliveInterval)
    }

    pub fn set_type_of_service(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(SocketOptions::TypeOfService, value)
    }

    pub fn type_of_service(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::TypeOfService)
    }

    pub fn set_zap_domain<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::ZapDomain, value)
    }

    pub fn zap_domain(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::ZapDomain)
    }

    pub fn bind<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.bind(endpoint.as_ref())
    }

    pub fn unbind<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.unbind(endpoint.as_ref())
    }

    pub fn connect<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.connect(endpoint.as_ref())
    }

    pub fn disconnect<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.disconnect(endpoint.as_ref())
    }

    pub fn monitor<F: Into<MonitorFlags>>(&self, events: F) -> ZmqResult<MonitorSocket> {
        let fd = self
            .socket
            .get_sockopt_int::<usize>(SocketOptions::FileDescriptor as i32)?;
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

    pub fn poll<E: Into<PollEvents>>(&self, events: E, timeout_ms: i64) -> ZmqResult<i32> {
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

pub trait Receiver<F>
where
    F: Into<RecvFlags> + Copy,
{
    fn recv_msg(&self, flags: F) -> ZmqResult<Message>;
    fn recv_multipart(&self, flags: F) -> ZmqResult<MultipartMessage> {
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

impl<T: sealed::SocketType + sealed::ReceiverFlag + Unpin, F: Into<RecvFlags> + Copy> Receiver<F>
    for Socket<T>
{
    fn recv_msg(&self, flags: F) -> ZmqResult<Message> {
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

pub trait Sender<S: sealed::SocketType + sealed::SenderFlag + Unpin> {
    fn send_msg(&self, msg: Message, flags: SendFlags) -> ZmqResult<()>;

    fn send_multipart(&self, iter: MultipartMessage, flags: SendFlags) -> ZmqResult<()> {
        let mut last_part: Option<Message> = None;
        for part in iter {
            let maybe_last = last_part.take();
            if let Some(last) = maybe_last {
                self.send_msg(last, flags | SendFlags::SEND_MORE)?;
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

impl<T: sealed::SocketType + sealed::SenderFlag + Unpin> Sender<T> for Socket<T> {
    fn send_msg(&self, msg: Message, flags: SendFlags) -> ZmqResult<()> {
        msg.send(self, flags.bits())
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PollEvents(i16);

bitflags! {
    impl PollEvents: i16 {
        const ZMQ_POLLIN = 0b0000_0001;
        const ZMQ_POLLOUT = 0b0000_0010;
        const ZMQ_POLLERR = 0b0000_0100;
        const ZMQ_POLLPRI = 0b0000_1000;
    }
}
