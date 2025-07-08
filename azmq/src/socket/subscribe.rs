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

/// A Publisher socket `ZMQ_SUB`
///
/// A socket of type [`Subscribe`] is used by a subscriber to subscribe to data distributed by a
/// [`Publish`](struct@super::Publish). Initially a [`Subscribe`] socket is not subscribed
/// to any messages, use the
/// [`subscribe()`](method@super::ZmqSocket<Subscriber>::subscribe) function specify
/// which messages to subscribe to.
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
