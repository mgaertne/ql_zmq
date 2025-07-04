use super::{ZmqSocket, ZmqSocketOptions, ZmqSocketType};
use crate::{ZmqResult, sealed};

pub struct Dealer {}

impl sealed::ZmqSenderFlag for Dealer {}
impl sealed::ZmqReceiverFlag for Dealer {}

impl sealed::ZmqSocketType for Dealer {
    fn raw_socket_type() -> ZmqSocketType {
        ZmqSocketType::Dealer
    }
}

unsafe impl Sync for ZmqSocket<Dealer> {}
unsafe impl Send for ZmqSocket<Dealer> {}

impl ZmqSocket<Dealer> {
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::Conflate as i32, value)
    }

    pub fn conflate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::Conflate as i32)
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

    pub fn disconnect<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.disconnect(endpoint.as_ref())
    }
}
