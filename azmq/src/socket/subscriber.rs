use crate::{
    ZmqResult,
    sealed::{ZmqReceiverFlag, ZmqSocketType},
    socket::{ZmqSocket, ZmqSocketOptions},
    zmq_sys_crate,
};

pub struct Subscriber {}

impl ZmqReceiverFlag for Subscriber {}

unsafe impl Sync for ZmqSocket<Subscriber> {}
unsafe impl Send for ZmqSocket<Subscriber> {}

impl ZmqSocketType for Subscriber {
    fn raw_socket_type() -> u32 {
        zmq_sys_crate::ZMQ_SUB
    }
}

impl ZmqSocket<Subscriber> {
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::Conflate as i32, value)
    }

    pub fn conflate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::Conflate as i32)
    }

    pub fn subscribe<V: AsRef<[u8]>>(&self, topic: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(ZmqSocketOptions::Subscribe as i32, topic.as_ref())
    }

    pub fn unsubscribe<V: AsRef<[u8]>>(&self, topic: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(ZmqSocketOptions::Unsubscribe as i32, topic.as_ref())
    }

    pub fn connect<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.connect(endpoint.as_ref())
    }

    pub fn disconnect<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.disconnect(endpoint.as_ref())
    }
}
