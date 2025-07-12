use super::{Socket, SocketOptions, SocketType};
use crate::{ZmqResult, sealed};

pub struct Subscribe {}

impl sealed::ReceiverFlag for Subscribe {}

unsafe impl Sync for Socket<Subscribe> {}
unsafe impl Send for Socket<Subscribe> {}

impl sealed::SocketType for Subscribe {
    fn raw_socket_type() -> SocketType {
        SocketType::Subscribe
    }
}

/// # A Subscriber socket `ZMQ_SUB`
///
/// A socket of type [`Subscribe`] is used by a subscriber to subscribe to data distributed by a
/// [`Publish`](struct@super::Publish). Initially a [`Subscribe`] socket is not subscribed
/// to any messages, use the
/// [`subscribe()`](method@super::Socket<Subscriber>::subscribe) function specify
/// which messages to subscribe to.
impl Socket<Subscribe> {
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::Conflate as i32, value)
    }

    pub fn conflate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::Conflate as i32)
    }

    pub fn subscribe<V: AsRef<[u8]>>(&self, topic: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(SocketOptions::Subscribe as i32, topic.as_ref())
    }

    pub fn unsubscribe<V: AsRef<[u8]>>(&self, topic: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(SocketOptions::Unsubscribe as i32, topic.as_ref())
    }
}
