use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOptions, SocketType},
};

pub struct Request {}

impl sealed::SenderFlag for Request {}
impl sealed::ReceiverFlag for Request {}

impl sealed::SocketType for Request {
    fn raw_socket_type() -> SocketType {
        SocketType::Request
    }
}

unsafe impl Sync for Socket<Request> {}
unsafe impl Send for Socket<Request> {}

/// # A Requester socket `ZMQ_REQ`
///
/// A socket of type [`Request`] is used by a client to send requests to and receive replies from
/// a service. This socket type allows only an alternating sequence of
/// [`send_msg()`](method@super::Sender::send_msg()) and subsequent
/// [`recv_msg()`](method@super::Receiver::recv_msg()) calls. Each request sent is round-robined
/// among all services, and each reply received is matched with the last issued request.
///
/// For connection-oriented transports, If the [`immediate()`](method@super::Socket::immediate())
/// option is set and there is no service available, then any send operation on the socket shall
/// block until at least one service becomes available. The [`Request`] socket shall not discard
/// messages.
impl Socket<Request> {
    pub fn set_correlate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::RequestCorrelate as i32, value)
    }

    pub fn correlate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::RequestCorrelate as i32)
    }

    pub fn set_relaxed(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::RequestRelaxed as i32, value)
    }

    pub fn relaxed(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::RequestRelaxed as i32)
    }

    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::RoutingId as i32)
    }
}
