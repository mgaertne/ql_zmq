use super::{ZmqSocket, ZmqSocketOptions};
use crate::{
    ZmqResult,
    sealed::{ZmqReceiverFlag, ZmqSenderFlag, ZmqSocketType},
    zmq_sys_crate,
};

pub struct Dealer {}

impl ZmqSenderFlag for Dealer {}
impl ZmqReceiverFlag for Dealer {}

impl ZmqSocketType for Dealer {
    fn raw_socket_type() -> u32 {
        zmq_sys_crate::ZMQ_DEALER
    }
}

unsafe impl Sync for ZmqSocket<Dealer> {}
unsafe impl Send for ZmqSocket<Dealer> {}

impl ZmqSocket<Dealer> {
    /// The `ZMQ_BACKLOG` option shall set the maximum length of the queue of outstanding peer
    /// connections for the specified `socket`; this only applies to connection-oriented
    /// transports. For details refer to your operating system documentation for the `listen`
    /// function.
    pub fn set_backlog(&self, value: i32) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::Backlog as i32, value)
    }

    /// The `ZMQ_BACKLOG` option shall retrieve the maximum length of the queue of outstanding
    /// peer connections for the specified `socket`; this only applies to connection-oriented
    /// transports. For details refer to your operating system documentation for the `listen`
    /// function.
    pub fn backlog(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::Backlog as i32)
    }

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
