use super::{Socket, SocketOptions, SocketType};
use crate::{ZmqResult, sealed};

/// # A XSubscribe socket `ZMQ_XSUB`
///
/// Same as [`Subscribe`] except that you subscribe by sending subscription messages to the socket.
/// Subscription message is a byte 1 (for subscriptions) or byte 0 (for unsubscriptions) followed
/// by the subscription body. Messages without a sub/unsub prefix may also be sent, but have no
/// effect on subscription status.
///
/// A socket of type [`Subscribe`] is used by a subscriber to subscribe to data distributed by a
/// [`Publish`]. Initially a [`Subscribe`] socket is not subscribed to any messages, use the
/// [`subscribe()`] function specify which messages to subscribe to.
///
/// [`Subscribe`]: super::SubscribeSocket
/// [`Publish`]: super::PublishSocket
/// [`subscribe()`]: #method.subscribe
pub type XSubscribeSocket = Socket<XSubscribe>;

pub struct XSubscribe {}

impl sealed::SenderFlag for XSubscribe {}
impl sealed::ReceiverFlag for XSubscribe {}

unsafe impl Sync for Socket<XSubscribe> {}
unsafe impl Send for Socket<XSubscribe> {}

impl sealed::SocketType for XSubscribe {
    fn raw_socket_type() -> SocketType {
        SocketType::Subscribe
    }
}

impl Socket<XSubscribe> {
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
