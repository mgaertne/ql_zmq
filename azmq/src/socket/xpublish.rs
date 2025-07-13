use super::{Socket, SocketOptions, SocketType};
use crate::{ZmqResult, sealed};

/// # A XSubscriber socket `ZMQ_XPUB`
///
/// Same as [`Publish`] except that you can receive subscriptions from the peers in form of
/// incoming messages. Subscription message is a byte 1 (for subscriptions) or byte 0 (for
/// unsubscriptions) followed by the subscription body. Messages without a sub/unsub prefix are
/// also received, but have no effect on subscription status.
///
/// A socket of type [`XPublish`] is used by a publisher to distribute data. Messages sent are
/// distributed in a fan out fashion to all connected peers.
///
/// When a [`XPublish`] sxocket enters the `mute` state due to having reached the high water mark
/// for a subscriber, then any messages that would be sent to the subscriber in question shall
/// instead be dropped until the mute state ends. The [`send_msg()`] function shall never block for
/// this socket type.
///
/// [`XPublish`]: XPublishSocket
/// [`Publish`]: super::PublishSocket
/// [`send_msg()`]: #impl-Sender<T>-for-Socket<T>
pub type XPublishSocket = Socket<XPublish>;

pub struct XPublish {}

impl sealed::SenderFlag for XPublish {}
impl sealed::ReceiverFlag for XPublish {}
impl sealed::SocketType for XPublish {
    fn raw_socket_type() -> SocketType {
        SocketType::XPublish
    }
}

unsafe impl Sync for Socket<XPublish> {}
unsafe impl Send for Socket<XPublish> {}

impl Socket<XPublish> {
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::Conflate as i32, value)
    }

    pub fn conflate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::Conflate as i32)
    }

    pub fn set_invert_matching(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::InvertMatching as i32, value)
    }

    pub fn invert_matching(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::InvertMatching as i32)
    }

    pub fn set_nodrop(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::XpubNoDrop as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn topic_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::TopicsCount as i32)
    }
}
