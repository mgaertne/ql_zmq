use super::{ZmqSocket, ZmqSocketOptions, ZmqSocketType};
use crate::{ZmqResult, sealed};

pub struct Subscribe {}

impl sealed::ZmqReceiverFlag for Subscribe {}

unsafe impl Sync for ZmqSocket<Subscribe> {}
unsafe impl Send for ZmqSocket<Subscribe> {}

impl sealed::ZmqSocketType for Subscribe {
    fn raw_socket_type() -> ZmqSocketType {
        ZmqSocketType::Subscribe
    }
}

impl ZmqSocket<Subscribe> {
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
}
