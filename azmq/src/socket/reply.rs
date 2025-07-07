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

    pub fn bind<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.bind(endpoint.as_ref())
    }

    pub fn unbind<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.unbind(endpoint.as_ref())
    }
}
