use crate::{
    ZmqResult, sealed,
    socket::{ZmqSocket, ZmqSocketOptions, ZmqSocketType},
};

pub struct Reply {}

impl sealed::ZmqSenderFlag for Reply {}
impl sealed::ZmqReceiverFlag for Reply {}

impl sealed::ZmqSocketType for Reply {
    fn raw_socket_type() -> ZmqSocketType {
        ZmqSocketType::Reply
    }
}

unsafe impl Sync for ZmqSocket<Reply> {}
unsafe impl Send for ZmqSocket<Reply> {}

/// A Reply socket `ZMQ_REP`
///
/// A socket of type [`Reply`] is used by a service to receive requests from and send replies to a
/// client. This socket type allows only an alternating sequence of
/// [`recv_msg()`](method@super::ZmqReceiver::recv_msg()) and subsequent
/// [`send_msg()`](method@super::ZmqSender::send_msg()) calls. Each request received is
/// fair-queued from among all clients, and each reply sent is routed to the client that issued
/// the last request. If the original requester does not exist any more the reply is silently
/// discarded.
impl ZmqSocket<Reply> {
    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::RoutingId as i32)
    }
}
