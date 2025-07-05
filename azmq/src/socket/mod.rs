use alloc::sync::Arc;
use core::{iter, ops::ControlFlow, pin::Pin, task::Poll};

use async_trait::async_trait;
use bitflags::bitflags;
use derive_more::From;
use futures::{StreamExt, TryStreamExt, future::FutureExt};
use num_traits::PrimInt;

use crate::{
    ZmqResult,
    context::ZmqContext,
    ffi::RawSocket,
    message::{ZmqMessage, ZmqSendable},
    sealed::{ZmqReceiverFlag, ZmqSenderFlag, ZmqSocketType},
    zmq_sys_crate,
};

mod dealer;
mod monitor;
mod subscriber;

pub use dealer::Dealer;
pub use monitor::{AsyncMonitorReceiver, Monitor, MonitorSocketEvent};
pub use subscriber::Subscriber;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum ZmqSocketOptions {
    Affinity = zmq_sys_crate::ZMQ_AFFINITY as i32,
    Backlog = zmq_sys_crate::ZMQ_BACKLOG as i32,
    BindToDevice = zmq_sys_crate::ZMQ_BINDTODEVICE as i32,
    ConnectTimeout = zmq_sys_crate::ZMQ_CONNECT_TIMEOUT as i32,
    CurvePublicKey = zmq_sys_crate::ZMQ_CURVE_PUBLICKEY as i32,
    CurveSecretKey = zmq_sys_crate::ZMQ_CURVE_SECRETKEY as i32,
    CurveServerKey = zmq_sys_crate::ZMQ_CURVE_SERVERKEY as i32,
    Events = zmq_sys_crate::ZMQ_EVENTS as i32,
    GssApiPlainText = zmq_sys_crate::ZMQ_GSSAPI_PLAINTEXT as i32,
    GssApiPrincipal = zmq_sys_crate::ZMQ_GSSAPI_PRINCIPAL as i32,
    GssApiServer = zmq_sys_crate::ZMQ_GSSAPI_SERVER as i32,
    GssApiServicePrincipal = zmq_sys_crate::ZMQ_GSSAPI_SERVICE_PRINCIPAL as i32,
    HandshakeInterval = zmq_sys_crate::ZMQ_HANDSHAKE_IVL as i32,
    HeartbeatInterval = zmq_sys_crate::ZMQ_HEARTBEAT_IVL as i32,
    HeartbeatTimeout = zmq_sys_crate::ZMQ_HEARTBEAT_TIMEOUT as i32,
    HeartbeatTimeToLive = zmq_sys_crate::ZMQ_HEARTBEAT_TTL as i32,
    Identity = 5i32,
    Immediate = zmq_sys_crate::ZMQ_IMMEDIATE as i32,
    IPv6 = zmq_sys_crate::ZMQ_IPV6 as i32,
    Linger = zmq_sys_crate::ZMQ_LINGER as i32,
    LastEndpoint = zmq_sys_crate::ZMQ_LAST_ENDPOINT as i32,
    MaxMessageSize = zmq_sys_crate::ZMQ_MAXMSGSIZE as i32,
    Mechanism = zmq_sys_crate::ZMQ_MECHANISM as i32,
    MulticastHops = zmq_sys_crate::ZMQ_MULTICAST_HOPS as i32,
    PlainPassword = zmq_sys_crate::ZMQ_PLAIN_PASSWORD as i32,
    PlainServer = zmq_sys_crate::ZMQ_PLAIN_SERVER as i32,
    PlainUsername = zmq_sys_crate::ZMQ_PLAIN_USERNAME as i32,
    ProbeRouter = zmq_sys_crate::ZMQ_PROBE_ROUTER as i32,
    Rate = zmq_sys_crate::ZMQ_RATE as i32,
    ReceiveBuffer = zmq_sys_crate::ZMQ_RCVBUF as i32,
    ReceiveHighWatermark = zmq_sys_crate::ZMQ_RCVHWM as i32,
    ReceiveTimeout = zmq_sys_crate::ZMQ_RCVTIMEO as i32,
    ReconnectInterval = zmq_sys_crate::ZMQ_RECONNECT_IVL as i32,
    ReconnectIntervalMax = zmq_sys_crate::ZMQ_RECONNECT_IVL_MAX as i32,
    RecoveryInterval = zmq_sys_crate::ZMQ_RECOVERY_IVL as i32,
    RequestCorrelate = zmq_sys_crate::ZMQ_REQ_CORRELATE as i32,
    RequestRelaxed = zmq_sys_crate::ZMQ_REQ_RELAXED as i32,
    RouterHandover = zmq_sys_crate::ZMQ_ROUTER_HANDOVER as i32,
    RouterMandatory = zmq_sys_crate::ZMQ_ROUTER_MANDATORY as i32,
    SendBuffer = zmq_sys_crate::ZMQ_SNDBUF as i32,
    SendHighWatermark = zmq_sys_crate::ZMQ_SNDHWM as i32,
    SendTimeout = zmq_sys_crate::ZMQ_SNDTIMEO as i32,
    SocksProxy = zmq_sys_crate::ZMQ_SOCKS_PROXY as i32,
    TcpKeepalive = zmq_sys_crate::ZMQ_TCP_KEEPALIVE as i32,
    TcpKeepaliveCount = zmq_sys_crate::ZMQ_TCP_KEEPALIVE_CNT as i32,
    TcpKeepaliveIdle = zmq_sys_crate::ZMQ_TCP_KEEPALIVE_IDLE as i32,
    TcpKeepaliveInterval = zmq_sys_crate::ZMQ_TCP_KEEPALIVE_INTVL as i32,
    TypeOfService = zmq_sys_crate::ZMQ_TOS as i32,
    ZapDomain = zmq_sys_crate::ZMQ_ZAP_DOMAIN as i32,
    FileDescriptor = zmq_sys_crate::ZMQ_FD as i32,
    Conflate = zmq_sys_crate::ZMQ_CONFLATE as i32,
    Subscribe = zmq_sys_crate::ZMQ_SUBSCRIBE as i32,
    Unsubscribe = zmq_sys_crate::ZMQ_UNSUBSCRIBE as i32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum ZmqMechanism {
    Null = zmq_sys_crate::ZMQ_NULL as i32,
    Plain = zmq_sys_crate::ZMQ_PLAIN as i32,
    Curve = zmq_sys_crate::ZMQ_CURVE as i32,
    GssApi = zmq_sys_crate::ZMQ_GSSAPI as i32,
    Unsupported(i32),
}

impl From<i32> for ZmqMechanism {
    fn from(value: i32) -> Self {
        match value {
            _ if value == zmq_sys_crate::ZMQ_NULL as i32 => ZmqMechanism::Null,
            _ if value == zmq_sys_crate::ZMQ_PLAIN as i32 => ZmqMechanism::Plain,
            _ if value == zmq_sys_crate::ZMQ_CURVE as i32 => ZmqMechanism::Curve,
            _ if value == zmq_sys_crate::ZMQ_GSSAPI as i32 => ZmqMechanism::GssApi,
            value => ZmqMechanism::Unsupported(value),
        }
    }
}

impl From<ZmqSocketOptions> for i32 {
    fn from(value: ZmqSocketOptions) -> Self {
        value as i32
    }
}

pub struct ZmqSocket<T: ZmqSocketType> {
    context: ZmqContext,
    pub(crate) socket: Arc<RawSocket<T>>,
}

impl<T: ZmqSocketType + Unpin> ZmqSocket<T> {
    pub fn from_context(context: &ZmqContext) -> ZmqResult<Self> {
        let socket = RawSocket::<T>::from_ctx(&context.inner)?;
        Ok(Self {
            context: context.clone(),
            socket: socket.into(),
        })
    }

    pub fn set_sockopt_bytes<V: AsRef<[u8]>>(&self, option: i32, value: V) -> ZmqResult<()> {
        self.socket.set_sockopt_bytes(option, value.as_ref())
    }

    pub fn set_sockopt_string<V: AsRef<str>>(&self, option: i32, value: V) -> ZmqResult<()> {
        self.socket.set_sockopt_string(option, value.as_ref())
    }

    pub fn set_sockopt_int<V>(&self, option: i32, value: V) -> ZmqResult<()>
    where
        V: PrimInt + Default,
    {
        self.socket.set_sockopt_int(option, value)
    }

    pub fn set_sockopt_bool(&self, option: i32, value: bool) -> ZmqResult<()> {
        self.socket.set_sockopt_bool(option, value)
    }

    pub fn get_sockopt_bytes(&self, option: i32) -> ZmqResult<Vec<u8>> {
        self.socket.get_sockopt_bytes(option)
    }

    pub fn get_sockopt_string(&self, option: i32) -> ZmqResult<String> {
        self.socket.get_sockopt_string(option)
    }

    pub fn get_sockopt_int<V>(&self, option: i32) -> ZmqResult<V>
    where
        V: PrimInt + Default,
    {
        self.socket.get_sockopt_int(option)
    }

    pub fn get_sockopt_bool(&self, option: i32) -> ZmqResult<bool> {
        self.socket.get_sockopt_bool(option)
    }

    pub fn set_affinity(&self, value: u64) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::Affinity as i32, value)
    }

    pub fn affinity(&self) -> ZmqResult<u64> {
        self.get_sockopt_int(ZmqSocketOptions::Affinity as i32)
    }

    pub fn set_connect_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::ConnectTimeout as i32, value)
    }

    pub fn connect_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::ConnectTimeout as i32)
    }

    pub fn set_curve_publickey<V: AsRef<[u8]>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(ZmqSocketOptions::CurvePublicKey as i32, value.as_ref())
    }

    pub fn curve_publickey(&self) -> ZmqResult<Vec<u8>> {
        self.get_sockopt_bytes(ZmqSocketOptions::CurvePublicKey as i32)
    }

    pub fn set_curve_secretkey<V: AsRef<[u8]>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(ZmqSocketOptions::CurveSecretKey as i32, value.as_ref())
    }

    pub fn curve_secretkey(&self) -> ZmqResult<Vec<u8>> {
        self.get_sockopt_bytes(ZmqSocketOptions::CurveSecretKey as i32)
    }

    pub fn set_curve_serverkey<V: AsRef<[u8]>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(ZmqSocketOptions::CurveServerKey as i32, value.as_ref())
    }

    pub fn curve_serverkey(&self) -> ZmqResult<Vec<u8>> {
        self.get_sockopt_bytes(ZmqSocketOptions::CurveServerKey as i32)
    }

    pub fn events(&self) -> ZmqResult<ZmqPollEvents> {
        self.get_sockopt_int::<i16>(ZmqSocketOptions::Events as i32)
            .map(ZmqPollEvents::from_bits_truncate)
    }

    pub fn set_gssapi_plaintext(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::GssApiPlainText as i32, value)
    }

    pub fn gssapi_plaintext(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::GssApiPlainText as i32)
    }

    pub fn set_gssapi_principal<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::GssApiPrincipal as i32, value.as_ref())
    }

    pub fn gssapi_principal(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::GssApiPrincipal as i32)
    }

    pub fn set_gssapi_server(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::GssApiServer as i32, value)
    }

    pub fn gssapi_server(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::GssApiServer as i32)
    }

    pub fn set_gssapi_service_principal<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_string(
            ZmqSocketOptions::GssApiServicePrincipal as i32,
            value.as_ref(),
        )
    }

    pub fn gssapi_service_principal(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::GssApiServicePrincipal as i32)
    }

    pub fn set_handshake_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::HandshakeInterval as i32, value)
    }

    pub fn handshake_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::HandshakeInterval as i32)
    }

    pub fn set_heartbeat_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::HeartbeatInterval as i32, value)
    }

    pub fn heartbeat_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::HeartbeatInterval as i32)
    }

    pub fn set_heartbeat_timeout(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::HeartbeatTimeout as i32, value)
    }

    pub fn heartbeat_timeout(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::HeartbeatTimeout as i32)
    }

    pub fn set_heartbeat_ttl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::HeartbeatTimeToLive as i32, value)
    }

    pub fn heartbeat_ttl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::HeartbeatTimeToLive as i32)
    }

    pub fn set_identity<V: AsRef<[u8]>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(ZmqSocketOptions::Identity as i32, value.as_ref())
    }

    pub fn identity(&self) -> ZmqResult<Vec<u8>> {
        self.get_sockopt_bytes(ZmqSocketOptions::Identity as i32)
    }

    pub fn set_immediate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::Immediate as i32, value)
    }

    pub fn immediate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::Immediate as i32)
    }

    pub fn set_ipv6(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::IPv6 as i32, value)
    }

    pub fn ipv6(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::IPv6 as i32)
    }

    pub fn set_linger(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::Linger as i32, value)
    }

    pub fn linger(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::Linger as i32)
    }

    pub fn last_endpoint(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::LastEndpoint as i32)
    }

    pub fn set_maxmsgsize(&self, value: i64) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::MaxMessageSize as i32, value)
    }

    pub fn maxmsgsize(&self) -> ZmqResult<i64> {
        self.get_sockopt_int(ZmqSocketOptions::MaxMessageSize as i32)
    }

    pub fn mechanism(&self) -> ZmqResult<ZmqMechanism> {
        self.get_sockopt_int::<i32>(ZmqSocketOptions::Mechanism as i32)
            .map(ZmqMechanism::from)
    }

    pub fn set_multicast_hops(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::MulticastHops as i32, value)
    }

    pub fn multicast_hops(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::MulticastHops as i32)
    }

    pub fn set_plain_password<V: AsRef<str>>(&self, value: Option<V>) -> ZmqResult<()> {
        match value {
            None => self.set_sockopt_bytes(ZmqSocketOptions::PlainPassword as i32, vec![]),
            Some(ref_value) => {
                self.set_sockopt_string(ZmqSocketOptions::PlainPassword as i32, ref_value)
            }
        }
    }

    pub fn plain_password(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::PlainPassword as i32)
    }

    pub fn set_plain_server(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::PlainServer as i32, value)
    }

    pub fn plain_server(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::PlainServer as i32)
    }

    pub fn set_plain_username<V: AsRef<str>>(&self, value: Option<V>) -> ZmqResult<()> {
        match value {
            None => self.set_sockopt_bytes(ZmqSocketOptions::PlainUsername as i32, vec![]),
            Some(ref_value) => {
                self.set_sockopt_string(ZmqSocketOptions::PlainUsername as i32, ref_value)
            }
        }
    }

    pub fn plain_username(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::PlainUsername as i32)
    }

    pub fn set_probe_router(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::ProbeRouter as i32, value)
    }

    pub fn set_rate(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::Rate as i32, value)
    }

    pub fn rate(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::Rate as i32)
    }

    pub fn set_rcvbuf(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::ReceiveBuffer as i32, value)
    }

    pub fn rcvbuf(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::ReceiveBuffer as i32)
    }

    pub fn set_rcvhwm(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::ReceiveHighWatermark as i32, value)
    }

    pub fn rcvhwm(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::ReceiveHighWatermark as i32)
    }

    pub fn set_rcvtimeo(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::ReceiveTimeout as i32, value)
    }

    pub fn rcvtimeo(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::ReceiveTimeout as i32)
    }

    pub fn set_reconnect_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::ReconnectInterval as i32, value)
    }

    pub fn reconnect_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::ReconnectInterval as i32)
    }

    pub fn set_reconnect_ivl_max(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::ReconnectIntervalMax as i32, value)
    }

    pub fn reconnect_ivl_max(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::ReconnectIntervalMax as i32)
    }

    pub fn set_recovery_ivl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::RecoveryInterval as i32, value)
    }

    pub fn recovery_ivl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::RecoveryInterval as i32)
    }

    pub fn set_req_correlate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::RequestCorrelate as i32, value)
    }

    pub fn set_req_relaxed(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::RequestRelaxed as i32, value)
    }

    pub fn set_router_handover(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::RouterHandover as i32, value)
    }

    pub fn set_router_mandatory(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::RouterMandatory as i32, value)
    }

    pub fn set_sndbuf(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::SendBuffer as i32, value)
    }

    pub fn sndbuf(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::SendBuffer as i32)
    }

    pub fn set_sndhwm(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::SendHighWatermark as i32, value)
    }

    pub fn sndhwm(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::SendHighWatermark as i32)
    }

    pub fn set_sndtimeo(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::SendTimeout as i32, value)
    }

    pub fn sndtimeo(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::SendTimeout as i32)
    }

    pub fn set_socks_proxy<V: AsRef<str>>(&self, value: Option<V>) -> ZmqResult<()> {
        match value {
            None => self.set_sockopt_bytes(ZmqSocketOptions::SocksProxy as i32, vec![]),
            Some(ref_value) => {
                self.set_sockopt_string(ZmqSocketOptions::SocksProxy as i32, ref_value)
            }
        }
    }

    pub fn socks_proxy(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::SocksProxy as i32)
    }

    pub fn set_tcp_keepalive(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::TcpKeepalive as i32, value)
    }

    pub fn tcp_keepalive(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::TcpKeepalive as i32)
    }

    pub fn set_tcp_keepalive_cnt(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::TcpKeepaliveCount as i32, value)
    }

    pub fn tcp_keepalive_cnt(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::TcpKeepaliveCount as i32)
    }

    pub fn set_tcp_keepalive_idle(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::TcpKeepaliveIdle as i32, value)
    }

    pub fn tcp_keepalive_idle(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::TcpKeepaliveIdle as i32)
    }

    pub fn set_tcp_keepalive_intvl(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::TcpKeepaliveInterval as i32, value)
    }

    pub fn tcp_keepalive_intvl(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::TcpKeepaliveInterval as i32)
    }

    pub fn set_type_of_service(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::TypeOfService as i32, value)
    }

    pub fn type_of_service(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::TypeOfService as i32)
    }

    pub fn set_zap_domain<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::ZapDomain as i32, value)
    }

    pub fn zap_domain(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::ZapDomain as i32)
    }

    pub fn monitor<F: Into<MonitorFlags>>(&self, events: F) -> ZmqResult<ZmqSocket<Monitor>> {
        let fd = self
            .socket
            .get_sockopt_int::<usize>(ZmqSocketOptions::FileDescriptor as i32)?;
        let monitor_endpoint = format!("inproc://monitor.s-{fd}");

        self.socket
            .monitor(&monitor_endpoint, events.into().bits() as i32)?;

        let monitor = RawSocket::<Monitor>::from_ctx(self.context.as_raw())?;

        monitor.connect(&monitor_endpoint)?;

        Ok(ZmqSocket {
            context: self.context.clone(),
            socket: monitor.into(),
        })
    }

    pub fn poll<E: Into<ZmqPollEvents>>(&self, events: E, timeout_ms: i64) -> ZmqResult<i32> {
        let poll_events = ZmqPollEvents::from_bits_truncate(events.into().bits());
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
pub struct ZmqRecvFlags(i32);

bitflags! {
    impl ZmqRecvFlags: i32 {
        const DONT_WAIT = 0b00000001;
    }
}

pub trait ZmqReceiver<F>
where
    F: Into<ZmqRecvFlags> + Copy,
{
    fn recv_msg(&self, flags: F) -> ZmqResult<ZmqMessage>;
    fn recv_multipart(&self, flags: F) -> ZmqResult<Vec<Vec<u8>>> {
        iter::repeat_with(|| self.recv_msg(flags))
            .try_fold(vec![], |mut parts, zmq_result| match zmq_result {
                Err(e) => ControlFlow::Break(Err(e)),
                Ok(zmq_msg) => {
                    parts.push(zmq_msg.to_vec());
                    if !zmq_msg.get_more() {
                        ControlFlow::Break(Ok(parts))
                    } else {
                        ControlFlow::Continue(parts)
                    }
                }
            })
            .break_value()
            .unwrap()
    }
}

impl<T: ZmqSocketType + ZmqReceiverFlag + Unpin, F: Into<ZmqRecvFlags> + Copy> ZmqReceiver<F>
    for ZmqSocket<T>
{
    fn recv_msg(&self, flags: F) -> ZmqResult<ZmqMessage> {
        self.socket
            .recv(flags.into().bits())
            .map(ZmqMessage::from_raw_msg)
    }
}

#[async_trait]
pub trait AsyncZmqReceiver<'a> {
    async fn recv_msg_async(&'a self) -> Option<ZmqMessage>;
    async fn recv_multipart_async(&'a self) -> Option<Vec<Vec<u8>>> {
        futures::stream::repeat_with(|| async move { self.recv_msg_async().await })
            .then(|item| async move { item.await.ok_or("filtered value") })
            .try_fold(vec![], |mut parts, zmq_msg| async move {
                parts.push(zmq_msg.to_vec());

                if zmq_msg.get_more() {
                    return Err("End reached");
                }
                Ok(parts)
            })
            .await
            .ok()
    }
}

#[async_trait]
impl<'a, T: ZmqSocketType + ZmqReceiverFlag + Unpin> AsyncZmqReceiver<'a> for ZmqSocket<T>
where
    ZmqSocket<T>: Sync,
{
    async fn recv_msg_async(&'a self) -> Option<ZmqMessage> {
        MessageReceivingFuture { receiver: self }.now_or_never()
    }
}

struct MessageReceivingFuture<'a, T: ZmqSocketType + ZmqReceiverFlag + Unpin> {
    receiver: &'a ZmqSocket<T>,
}

impl<T: ZmqSocketType + ZmqReceiverFlag + Unpin> Future for MessageReceivingFuture<'_, T> {
    type Output = ZmqMessage;

    fn poll(self: Pin<&mut Self>, _ctx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        self.receiver
            .socket
            .recv(ZmqRecvFlags::DONT_WAIT.bits())
            .map(ZmqMessage::from_raw_msg)
            .map_or(Poll::Pending, Poll::Ready)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZmqSendFlags(i32);

bitflags! {
    impl ZmqSendFlags: i32 {
        const DONT_WAIT = 0b00000001;
        const SEND_MORE = 0b00000010;
    }
}

pub trait ZmqSender<V, S: ZmqSocketType + ZmqSenderFlag + Unpin>
where
    V: ZmqSendable<S>,
{
    fn send_msg(&self, msg: V, flags: ZmqSendFlags) -> ZmqResult<()>;

    fn send_multipart<I>(&self, iter: I, flags: ZmqSendFlags) -> ZmqResult<()>
    where
        I: IntoIterator<Item = V>,
    {
        let mut last_part: Option<V> = None;
        for part in iter {
            let maybe_last = last_part.take();
            if let Some(last) = maybe_last {
                self.send_msg(last, flags | ZmqSendFlags::SEND_MORE)?;
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

impl<T: ZmqSocketType + ZmqSenderFlag + Unpin, V: ZmqSendable<T>> ZmqSender<V, T> for ZmqSocket<T> {
    fn send_msg(&self, msg: V, flags: ZmqSendFlags) -> ZmqResult<()> {
        msg.send(self, flags.bits())
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZmqPollEvents(i16);

bitflags! {
    impl ZmqPollEvents: i16 {
        const ZMQ_POLLIN = 0b0000_0001;
        const ZMQ_POLLOUT = 0b0000_0010;
        const ZMQ_POLLERR = 0b0000_0100;
        const ZMQ_POLLPRI = 0b0000_1000;
    }
}
