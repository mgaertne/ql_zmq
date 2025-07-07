use crate::{
    ZmqResult, sealed,
    socket::{ZmqSendFlags, ZmqSender, ZmqSocket, ZmqSocketOptions, ZmqSocketType},
};

pub struct Stream {}

impl sealed::ZmqSenderFlag for Stream {}
impl sealed::ZmqReceiverFlag for Stream {}

impl sealed::ZmqSocketType for Stream {
    fn raw_socket_type() -> ZmqSocketType {
        ZmqSocketType::Stream
    }
}

unsafe impl Sync for ZmqSocket<Stream> {}
unsafe impl Send for ZmqSocket<Stream> {}

impl ZmqSocket<Stream> {
    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::RoutingId as i32)
    }

    pub fn set_connect_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::ConnectRoutingId as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_stream_notify(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::StreamNotify as i32, value)
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

    pub fn disconnect<V: AsRef<str>>(&self) -> ZmqResult<()> {
        let routing_id = self.routing_id()?;
        self.send_msg(&routing_id, ZmqSendFlags::SEND_MORE)?;
        self.send_msg(vec![], ZmqSendFlags::DONT_WAIT)
    }
}
