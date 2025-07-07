use crate::{
    ZmqResult, sealed,
    socket::{ZmqSocket, ZmqSocketOptions, ZmqSocketType},
};

pub struct Request {}

impl sealed::ZmqSenderFlag for Request {}
impl sealed::ZmqReceiverFlag for Request {}

impl sealed::ZmqSocketType for Request {
    fn raw_socket_type() -> ZmqSocketType {
        ZmqSocketType::Request
    }
}

unsafe impl Sync for ZmqSocket<Request> {}
unsafe impl Send for ZmqSocket<Request> {}

impl ZmqSocket<Request> {
    pub fn set_correlate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::RequestCorrelate as i32, value)
    }

    pub fn correlate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::RequestCorrelate as i32)
    }

    pub fn set_relaxed(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::RequestRelaxed as i32, value)
    }

    pub fn relaxed(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::RequestRelaxed as i32)
    }

    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::RoutingId as i32)
    }
}
