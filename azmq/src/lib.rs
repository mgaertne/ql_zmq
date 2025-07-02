use core::{iter, marker::PhantomData, pin::Pin, task::Poll};

use anyhow::{Error, Result, anyhow};
use async_trait::async_trait;
use bitflags::bitflags;
use derive_more::From;
use futures::{
    future::FutureExt,
    stream::{StreamExt, TryStreamExt},
};
use zmq::{Context, Mechanism, Message, PollEvents, Sendable, Socket};

use crate::sealed::{ZmqReceiverFlag, ZmqSenderFlag, ZmqSocketType};

mod dealer;
mod monitor;
mod subscriber;

pub use dealer::Dealer;
pub use monitor::{AsyncMonitorReceiver, Monitor, MonitorSocketEvent};
pub use subscriber::Subscriber;

mod sealed {
    pub trait ZmqReceiverFlag {}
    pub trait ZmqSenderFlag {}
    pub trait ZmqSocketType {
        fn raw_socket_type() -> zmq::SocketType;
    }
}

pub struct ZmqSocket<T: ZmqSocketType> {
    context: Context,
    socket: Socket,
    socket_type: PhantomData<T>,
}

impl<T: ZmqSocketType + Unpin> ZmqSocket<T> {
    pub fn try_new() -> Result<Self> {
        let context = Context::new();
        let socket = context.socket(T::raw_socket_type())?;
        Ok(Self {
            context,
            socket,
            socket_type: PhantomData,
        })
    }
}

impl<T: ZmqSocketType + Unpin> ZmqSocket<T> {
    /// The `ZMQ_AFFINITY` option shall set the I/O thread affinity for newly created connections
    /// on the specified `socket`.
    ///
    /// Affinity determines which threads from the 0MQ I/O thread pool associated with the
    /// socket’s context shall handle newly created connections. A value of zero specifies no
    /// affinity, meaning that work shall be distributed fairly among all 0MQ I/O threads in the
    /// thread pool. For non-zero values, the lowest bit corresponds to thread 1, second lowest bit
    /// to thread 2 and so on. For example, a value of 3 specifies that subsequent connections on
    /// `socket` shall be handled exclusively by I/O threads 1 and 2.
    ///
    /// # Bitmap, default: 0
    pub fn set_affinity(&self, value: u64) -> Result<()> {
        self.socket.set_affinity(value).map_err(Error::from)
    }

    /// The `ZMQ_AFFINITY` option shall retrieve the I/O thread affinity for newly created
    /// connections on the specified `socket`.
    ///
    /// Affinity determines which threads from the 0MQ I/O thread pool associated with the
    /// socket’s context shall handle newly created connections. A value of zero specifies no
    /// affinity, meaning that work shall be distributed fairly among all 0MQ I/O threads in the
    /// thread pool. For non-zero values, the lowest bit corresponds to thread 1, second lowest
    /// bit to thread 2 and so on. For example, a value of 3 specifies that subsequent connections
    /// on `socket` shall be handled exclusively by I/O threads 1 and 2.
    ///
    /// # Default: 100 connections
    pub fn affinity(&self) -> Result<u64> {
        self.socket.get_affinity().map_err(Error::from)
    }

    /// Sets the time in milliseconds to wait before timing-out a connect() system call. The
    /// connect() system call normally takes a long time before it returns a time out error.
    /// Setting this option allows the library to time out the call at an earlier interval.
    ///
    /// # Default: 0 milliseconds (disabled)
    pub fn set_connect_timeout(&self, value: i32) -> Result<()> {
        self.socket.set_connect_timeout(value).map_err(Error::from)
    }

    /// Retrieves how long to wait before timing-out a connect() system call. The connect() system
    /// call normally takes a long time before it returns a time out error. Setting this option
    /// allows the library to time out the call at an earlier interval.
    pub fn connect_timeout(&self) -> Result<i32> {
        self.socket.get_connect_timeout().map_err(Error::from)
    }

    /// Sets the socket’s long term public key. You must set this on CURVE client sockets, see
    /// zmq_curve You can provide the key as 32 binary bytes, or as a 40-character string encoded
    /// in the Z85 encoding format and terminated in a null byte. The public key must always be
    /// used with the matching secret key. To generate a public/secret key pair, use
    /// zmq_curve_keypair To derive the public key from a secret key, use zmq_curve_public
    pub fn set_curve_publickey<V: AsRef<[u8]>>(&self, value: V) -> Result<()> {
        self.socket
            .set_curve_publickey(value.as_ref())
            .map_err(Error::from)
    }

    /// Retrieves the current long term public key for the socket. You can provide either a 32 byte
    /// buffer, to retrieve the binary key value, or a 41 byte buffer, to retrieve the key in a
    /// printable Z85 format. NOTE: to fetch a printable key, the buffer must be 41 bytes large to
    /// hold the 40-char key value and one null byte.
    pub fn curve_publickey(&self) -> Result<Vec<u8>> {
        self.socket.get_curve_publickey().map_err(Error::from)
    }

    /// Sets the socket’s long term secret key. You must set this on both CURVE client and server
    /// sockets, see zmq_curve You can provide the key as 32 binary bytes, or as a 40-character
    /// string encoded in the Z85 encoding format and terminated in a null byte. To generate a
    /// public/secret key pair, use zmq_curve_keypair To derive the public key from a secret key,
    /// use zmq_curve_public
    pub fn set_curve_secretkey<V: AsRef<[u8]>>(&self, value: V) -> Result<()> {
        self.socket
            .set_curve_secretkey(value.as_ref())
            .map_err(Error::from)
    }

    /// Retrieves the current long term secret key for the socket. You can provide either a 32 byte
    /// buffer, to retrieve the binary key value, or a 41 byte buffer, to retrieve the key in a
    /// printable Z85 format. NOTE: to fetch a printable key, the buffer must be 41 bytes large to
    /// hold the 40-char key value and one null byte.
    pub fn curve_secretkey(&self) -> Result<Vec<u8>> {
        self.socket.get_curve_secretkey().map_err(Error::from)
    }

    /// Sets the socket’s long term server key. You must set this on CURVE client sockets, see
    /// zmq_curve You can provide the key as 32 binary bytes, or as a 40-character string encoded
    /// in the Z85 encoding format and terminated in a null byte. This key must have been generated
    /// together with the server’s secret key. To generate a public/secret key pair, use
    /// zmq_curve_keypair
    pub fn set_curve_serverkey<V: AsRef<[u8]>>(&self, value: V) -> Result<()> {
        self.socket
            .set_curve_serverkey(value.as_ref())
            .map_err(Error::from)
    }

    /// Retrieves the current server key for the client socket. You can provide either a 32 byte
    /// buffer, to retrieve the binary key value, or a 41-byte buffer, to retrieve the key in a p
    /// rintable Z85 format. NOTE: to fetch a printable key, the buffer must be 41 bytes large to
    /// hold the 40-char key value and one null byte.
    pub fn curve_serverkey(&self) -> Result<Vec<u8>> {
        self.socket.get_curve_serverkey().map_err(Error::from)
    }

    /// The `ZMQ_EVENTS` option shall retrieve the event state for the specified `socket`. The
    /// returned value is a bit mask constructed by OR’ing a combination of the following event
    /// flags:
    ///
    /// `PollEvents::ZMQ_POLLIN`
    /// Indicates that at least one message may be received from the specified socket without
    /// blocking.
    ///
    /// `PollEvents::ZMQ_POLLOUT`
    /// Indicates that at least one message may be sent to the specified socket without blocking.
    ///
    /// The combination of a file descriptor returned by the `ZMQ_FD` option being ready for
    /// reading but no actual events returned by a subsequent retrieval of the `ZMQ_EVENTS`
    /// option is valid; applications should simply ignore this case and restart their polling
    /// operation/event loop.
    pub fn events(&self) -> Result<PollEvents> {
        self.socket.get_events().map_err(Error::from)
    }

    /// Defines whether communications on the socket will be encrypted, see zmq_gssapi A value of
    /// `1` means that communications will be plaintext. A value of `0` means communications will
    /// be encrypted.
    pub fn set_gssapi_plaintext(&self, value: bool) -> Result<()> {
        self.socket.set_gssapi_plaintext(value).map_err(Error::from)
    }

    /// Returns the `ZMQ_GSSAPI_PLAINTEXT` option, if any, previously set on the socket. A value
    /// of `1` means that communications will be plaintext. A value of `0` means communications
    /// will be encrypted.
    pub fn gssapi_plaintext(&self) -> Result<bool> {
        self.socket.is_gssapi_plaintext().map_err(Error::from)
    }

    /// Sets the name of the principal for whom GSSAPI credentials should be acquired.
    pub fn set_gssapi_principal<V: AsRef<str>>(&self, value: V) -> Result<()> {
        self.socket
            .set_gssapi_principal(value.as_ref())
            .map_err(Error::from)
    }

    /// The `ZMQ_GSSAPI_PRINCIPAL` option shall retrieve the principal name set for the GSSAPI
    /// security mechanism. The returned value shall be a NULL-terminated string and MAY be empty.
    /// The returned size SHALL include the terminating null byte.
    pub fn gssapi_principal(&self) -> Result<Result<String, Vec<u8>>> {
        self.socket.get_gssapi_principal().map_err(Error::from)
    }

    /// Defines whether the socket will act as server for GSSAPI security, see zmq_gssapi A value
    /// of `1` means the socket will act as GSSAPI server. A value of `0` means the socket will
    /// act as GSSAPI client.
    pub fn set_gssapi_server(&self, value: bool) -> Result<()> {
        self.socket.set_gssapi_server(value).map_err(Error::from)
    }

    /// Returns the `ZMQ_GSSAPI_SERVER` option, if any, previously set on the socket.
    pub fn gssapi_server(&self) -> Result<bool> {
        self.socket.is_gssapi_server().map_err(Error::from)
    }

    /// Sets the name of the principal of the GSSAPI server to which a GSSAPI client intends to
    /// connect.
    pub fn set_gssapi_service_principal<V: AsRef<str>>(&self, value: V) -> Result<()> {
        self.socket
            .set_gssapi_service_principal(value.as_ref())
            .map_err(Error::from)
    }

    /// The `ZMQ_GSSAPI_SERVICE_PRINCIPAL` option shall retrieve the principal name of the GSSAPI
    /// server to which a GSSAPI client socket intends to connect. The returned value shall be a
    /// NULL-terminated string and MAY be empty. The returned size SHALL include the terminating
    /// null byte.
    pub fn gssapi_service_principal(&self) -> Result<Result<String, Vec<u8>>> {
        self.socket
            .get_gssapi_service_principal()
            .map_err(Error::from)
    }

    pub fn set_handshake_ivl(&self, value: i32) -> Result<()> {
        self.socket.set_handshake_ivl(value).map_err(Error::from)
    }

    pub fn handshake_ivl(&self) -> Result<i32> {
        self.socket.get_handshake_ivl().map_err(Error::from)
    }

    pub fn set_heartbeat_ivl(&self, value: i32) -> Result<()> {
        self.socket.set_heartbeat_ivl(value).map_err(Error::from)
    }

    pub fn heartbeat_ivl(&self) -> Result<i32> {
        self.socket.get_heartbeat_ivl().map_err(Error::from)
    }

    pub fn set_heartbeat_timeout(&self, value: i32) -> Result<()> {
        self.socket
            .set_heartbeat_timeout(value)
            .map_err(Error::from)
    }

    pub fn heartbeat_timeout(&self) -> Result<i32> {
        self.socket.get_heartbeat_timeout().map_err(Error::from)
    }

    pub fn set_heartbeat_ttl(&self, value: i32) -> Result<()> {
        self.socket.set_heartbeat_ttl(value).map_err(Error::from)
    }

    pub fn heartbeat_ttl(&self) -> Result<i32> {
        self.socket.get_heartbeat_ttl().map_err(Error::from)
    }

    pub fn set_identity<V: AsRef<[u8]>>(&self, value: V) -> Result<()> {
        self.socket
            .set_identity(value.as_ref())
            .map_err(Error::from)
    }

    pub fn identity(&self) -> Result<Vec<u8>> {
        self.socket.get_identity().map_err(Error::from)
    }

    pub fn set_immediate(&self, value: bool) -> Result<()> {
        self.socket.set_immediate(value).map_err(Error::from)
    }

    pub fn immediate(&self) -> Result<bool> {
        self.socket.is_immediate().map_err(Error::from)
    }

    pub fn set_ipv6(&self, value: bool) -> Result<()> {
        self.socket.set_ipv6(value).map_err(Error::from)
    }

    pub fn ipv6(&self) -> Result<bool> {
        self.socket.is_ipv6().map_err(Error::from)
    }

    pub fn set_linger(&self, value: i32) -> Result<()> {
        self.socket.set_linger(value).map_err(Error::from)
    }

    pub fn last_endpoint(&self) -> Result<Result<String, Vec<u8>>> {
        self.socket.get_last_endpoint().map_err(Error::from)
    }

    pub fn linger(&self) -> Result<i32> {
        self.socket.get_linger().map_err(Error::from)
    }

    pub fn set_maxmsgsize(&self, value: i64) -> Result<()> {
        self.socket.set_maxmsgsize(value).map_err(Error::from)
    }

    pub fn maxmsgsize(&self) -> Result<i64> {
        self.socket.get_maxmsgsize().map_err(Error::from)
    }

    pub fn mechanism(&self) -> Result<Mechanism> {
        self.socket.get_mechanism().map_err(Error::from)
    }

    pub fn set_multicast_hops(&self, value: i32) -> Result<()> {
        self.socket.set_multicast_hops(value).map_err(Error::from)
    }

    pub fn multicast_hops(&self) -> Result<i32> {
        self.socket.get_multicast_hops().map_err(Error::from)
    }

    pub fn set_plain_password<V: AsRef<str>>(&self, value: Option<V>) -> Result<()> {
        let ref_value = value.as_ref().map(|v| v.as_ref());
        self.socket
            .set_plain_password(ref_value)
            .map_err(Error::from)
    }

    pub fn plain_password(&self) -> Result<Result<String, Vec<u8>>> {
        self.socket.get_plain_password().map_err(Error::from)
    }

    pub fn set_plain_server(&self, value: bool) -> Result<()> {
        self.socket.set_plain_server(value).map_err(Error::from)
    }

    pub fn plain_server(&self) -> Result<bool> {
        self.socket.is_plain_server().map_err(Error::from)
    }

    pub fn set_plain_username<V: AsRef<str>>(&self, value: Option<V>) -> Result<()> {
        let ref_value = value.as_ref().map(|v| v.as_ref());
        self.socket
            .set_plain_username(ref_value)
            .map_err(Error::from)
    }

    pub fn plain_username(&self) -> Result<Result<String, Vec<u8>>> {
        self.socket.get_plain_username().map_err(Error::from)
    }

    pub fn set_probe_router(&self, value: bool) -> Result<()> {
        self.socket.set_probe_router(value).map_err(Error::from)
    }

    pub fn set_rate(&self, value: i32) -> Result<()> {
        self.socket.set_rate(value).map_err(Error::from)
    }

    pub fn rate(&self) -> Result<i32> {
        self.socket.get_rate().map_err(Error::from)
    }

    pub fn set_rcvbuf(&self, value: i32) -> Result<()> {
        self.socket.set_rcvbuf(value).map_err(Error::from)
    }

    pub fn rcvbuf(&self) -> Result<i32> {
        self.socket.get_rcvbuf().map_err(Error::from)
    }

    pub fn set_rcvhwm(&self, value: i32) -> Result<()> {
        self.socket.set_rcvhwm(value).map_err(Error::from)
    }

    pub fn rcvhwm(&self) -> Result<i32> {
        self.socket.get_rcvhwm().map_err(Error::from)
    }

    pub fn set_rcvtimeo(&self, value: i32) -> Result<()> {
        self.socket.set_rcvtimeo(value).map_err(Error::from)
    }

    pub fn rcvtimeo(&self) -> Result<i32> {
        self.socket.get_rcvtimeo().map_err(Error::from)
    }

    pub fn set_reconnect_ivl(&self, value: i32) -> Result<()> {
        self.socket.set_reconnect_ivl(value).map_err(Error::from)
    }

    pub fn reconnect_ivl(&self) -> Result<i32> {
        self.socket.get_reconnect_ivl().map_err(Error::from)
    }

    pub fn set_reconnect_ivl_max(&self, value: i32) -> Result<()> {
        self.socket
            .set_reconnect_ivl_max(value)
            .map_err(Error::from)
    }

    pub fn reconnect_ivl_max(&self) -> Result<i32> {
        self.socket.get_reconnect_ivl_max().map_err(Error::from)
    }

    pub fn set_recovery_ivl(&self, value: i32) -> Result<()> {
        self.socket.set_recovery_ivl(value).map_err(Error::from)
    }

    pub fn recovery_ivl(&self) -> Result<i32> {
        self.socket.get_recovery_ivl().map_err(Error::from)
    }

    pub fn set_req_correlate(&self, value: bool) -> Result<()> {
        self.socket.set_req_correlate(value).map_err(Error::from)
    }

    pub fn set_req_relaxed(&self, value: bool) -> Result<()> {
        self.socket.set_req_relaxed(value).map_err(Error::from)
    }

    pub fn set_router_handover(&self, value: bool) -> Result<()> {
        self.socket.set_router_handover(value).map_err(Error::from)
    }

    pub fn set_router_mandatory(&self, value: bool) -> Result<()> {
        self.socket.set_router_mandatory(value).map_err(Error::from)
    }

    pub fn set_sndbuf(&self, value: i32) -> Result<()> {
        self.socket.set_sndbuf(value).map_err(Error::from)
    }

    pub fn sndbuf(&self) -> Result<i32> {
        self.socket.get_sndbuf().map_err(Error::from)
    }

    pub fn set_sndhwm(&self, value: i32) -> Result<()> {
        self.socket.set_sndhwm(value).map_err(Error::from)
    }

    pub fn sndhwm(&self) -> Result<i32> {
        self.socket.get_sndhwm().map_err(Error::from)
    }

    pub fn set_sndtimeo(&self, value: i32) -> Result<()> {
        self.socket.set_sndtimeo(value).map_err(Error::from)
    }

    pub fn sndtimeo(&self) -> Result<i32> {
        self.socket.get_sndtimeo().map_err(Error::from)
    }

    pub fn set_socks_proxy<V: AsRef<str>>(&self, value: Option<V>) -> Result<()> {
        let ref_value = value.as_ref().map(|v| v.as_ref());
        self.socket.set_socks_proxy(ref_value).map_err(Error::from)
    }

    pub fn socks_proxy(&self) -> Result<Result<String, Vec<u8>>> {
        self.socket.get_socks_proxy().map_err(Error::from)
    }

    pub fn set_tcp_keepalive(&self, value: i32) -> Result<()> {
        self.socket.set_tcp_keepalive(value).map_err(Error::from)
    }

    pub fn tcp_keepalive(&self) -> Result<i32> {
        self.socket.get_tcp_keepalive().map_err(Error::from)
    }

    pub fn set_tcp_keepalive_cnt(&self, value: i32) -> Result<()> {
        self.socket
            .set_tcp_keepalive_cnt(value)
            .map_err(Error::from)
    }

    pub fn tcp_keepalive_cnt(&self) -> Result<i32> {
        self.socket.get_tcp_keepalive_cnt().map_err(Error::from)
    }

    pub fn set_tcp_keepalive_idle(&self, value: i32) -> Result<()> {
        self.socket
            .set_tcp_keepalive_idle(value)
            .map_err(Error::from)
    }

    pub fn tcp_keepalive_idle(&self) -> Result<i32> {
        self.socket.get_tcp_keepalive_idle().map_err(Error::from)
    }

    pub fn set_tcp_keepalive_intvl(&self, value: i32) -> Result<()> {
        self.socket
            .set_tcp_keepalive_intvl(value)
            .map_err(Error::from)
    }

    pub fn tcp_keepalive_intvl(&self) -> Result<i32> {
        self.socket.get_tcp_keepalive_intvl().map_err(Error::from)
    }

    pub fn set_tos(&self, value: i32) -> Result<()> {
        self.socket.set_tos(value).map_err(Error::from)
    }

    pub fn tos(&self) -> Result<i32> {
        self.socket.get_tos().map_err(Error::from)
    }

    pub fn set_zap_domain<V: AsRef<str>>(&self, value: V) -> Result<()> {
        self.socket
            .set_zap_domain(value.as_ref())
            .map_err(Error::from)
    }

    pub fn zap_domain(&self) -> Result<Result<String, Vec<u8>>> {
        self.socket.get_zap_domain().map_err(Error::from)
    }

    pub fn monitor<F: Into<MonitorFlags>>(&self, events: F) -> Result<ZmqSocket<Monitor>> {
        let fd = self.socket.get_fd()?;
        let monitor_endpoint = format!("inproc://monitor.s-{fd}");

        self.socket
            .monitor(&monitor_endpoint, events.into().bits() as i32)?;

        let monitor = self.context.socket(zmq::PAIR)?;

        monitor.connect(&monitor_endpoint)?;

        Ok(ZmqSocket {
            context: self.context.clone(),
            socket: monitor,
            socket_type: PhantomData,
        })
    }
}

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
    fn recv_msg(&self, flags: F) -> Result<Message>;
    fn recv_multipart(&self, flags: F) -> Result<Vec<Vec<u8>>> {
        iter::repeat_with(|| self.recv_msg(flags)).try_fold(vec![], |mut parts, zmq_result| {
            let Ok(zmq_msg) = zmq_result else {
                return Ok(parts);
            };

            parts.push(zmq_msg.to_vec());
            if zmq_msg.get_more() {
                return Err(anyhow!("end reached"));
            }
            Ok(parts)
        })
    }
}

impl<T: ZmqSocketType + ZmqReceiverFlag + Unpin, F: Into<ZmqRecvFlags> + Copy> ZmqReceiver<F>
    for ZmqSocket<T>
{
    fn recv_msg(&self, flags: F) -> Result<Message> {
        self.socket
            .recv_msg(flags.into().bits())
            .map_err(Error::from)
    }
}

#[async_trait]
pub trait AsyncZmqReceiver {
    async fn recv_msg_async(&self) -> Option<Message>;
    async fn recv_multipart_async(&self) -> Option<Vec<Vec<u8>>> {
        futures::stream::repeat_with(|| async move { self.recv_msg_async().await })
            .then(|item| async move { item.await.ok_or(anyhow!("filtered value")) })
            .try_fold(vec![], |mut parts, zmq_msg| async move {
                parts.push(zmq_msg.to_vec());

                if zmq_msg.get_more() {
                    return Err(anyhow!("End reached"));
                }
                Ok(parts)
            })
            .await
            .ok()
    }
}

#[async_trait]
impl<T: ZmqSocketType + ZmqReceiverFlag + Unpin> AsyncZmqReceiver for ZmqSocket<T>
where
    ZmqSocket<T>: Sync,
{
    async fn recv_msg_async(&self) -> Option<Message> {
        MessageReceivingFuture { receiver: self }.now_or_never()
    }
}

struct MessageReceivingFuture<'a, T: ZmqSocketType + ZmqReceiverFlag + Unpin> {
    receiver: &'a ZmqSocket<T>,
}

impl<T: ZmqSocketType + ZmqReceiverFlag + Unpin> Future for MessageReceivingFuture<'_, T> {
    type Output = Message;

    fn poll(self: Pin<&mut Self>, _ctx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        self.receiver
            .socket
            .recv_msg(ZmqRecvFlags::DONT_WAIT.bits())
            .map_or(Poll::Pending, Poll::Ready)
    }
}

#[derive(Debug, Clone, Copy, From, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZmqSendFlags(i32);

bitflags! {
    impl ZmqSendFlags: i32 {
        const DONT_WAIT = 0b00000001;
        const SEND_MORE = 0b00000010;
    }
}

pub trait ZmqSender<V>
where
    V: Sendable,
{
    fn send_msg(&self, msg: V, flags: ZmqSendFlags) -> Result<()>;

    fn send_multipart<I>(&self, iter: I, flags: ZmqSendFlags) -> Result<()>
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

impl<T: ZmqSocketType + ZmqSenderFlag + Unpin, V: Sendable> ZmqSender<V> for ZmqSocket<T> {
    fn send_msg(&self, msg: V, flags: ZmqSendFlags) -> Result<()> {
        msg.send(&self.socket, flags.bits()).map_err(Error::from)
    }
}
