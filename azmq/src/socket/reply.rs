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

impl ZmqSocket<Reply> {
    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::RoutingId as i32)
    }
}
