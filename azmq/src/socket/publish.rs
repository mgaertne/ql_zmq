use super::{Socket, SocketOptions, SocketType};
use crate::{ZmqResult, sealed};

pub struct Publish {}

impl sealed::SenderFlag for Publish {}
impl sealed::SocketType for Publish {
    fn raw_socket_type() -> SocketType {
        SocketType::Publish
    }
}

unsafe impl Sync for Socket<Publish> {}
unsafe impl Send for Socket<Publish> {}

/// # A Subscriber socket `ZMQ_PUB`
///
/// A socket of type [`Publish`] is used by a publisher to distribute data. Messages sent are
/// distributed in a fan out fashion to all connected peers.
///
/// When a [`Publish`] socket enters the `mute` state due to having reached the high water mark
/// for a subscriber, then any messages that would be sent to the subscriber in question shall
/// instead be dropped until the mute state ends. The
/// [`send_msg()`](method@super::Sender::send_msg()) function shall never block for
/// this socket type.
impl Socket<Publish> {
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
