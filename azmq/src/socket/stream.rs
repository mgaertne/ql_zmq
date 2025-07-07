use crate::{
    ZmqResult, sealed,
    socket::{ZmqSocket, ZmqSocketOptions, ZmqSocketType},
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
}
