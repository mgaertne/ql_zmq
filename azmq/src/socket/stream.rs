use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOptions, SocketType},
};

pub struct Stream {}

impl sealed::SenderFlag for Stream {}
impl sealed::ReceiverFlag for Stream {}

impl sealed::SocketType for Stream {
    fn raw_socket_type() -> SocketType {
        SocketType::Stream
    }
}

unsafe impl Sync for Socket<Stream> {}
unsafe impl Send for Socket<Stream> {}

impl Socket<Stream> {
    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::RoutingId as i32)
    }

    pub fn set_connect_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::ConnectRoutingId as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_stream_notify(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::StreamNotify as i32, value)
    }
}
