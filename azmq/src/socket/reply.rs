use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOptions, SocketType},
};

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

/// # A Reply socket `ZMQ_REP`
///
/// A socket of type [`Reply`] is used by a service to receive requests from and send replies to a
/// client. This socket type allows only an alternating sequence of
/// [`recv_msg()`](method@super::Receiver::recv_msg()) and subsequent
/// [`send_msg()`](method@super::Sender::send_msg()) calls. Each request received is
/// fair-queued from among all clients, and each reply sent is routed to the client that issued
/// the last request. If the original requester does not exist any more the reply is silently
/// discarded.
impl Socket<Reply> {
    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::RoutingId as i32)
    }
}
