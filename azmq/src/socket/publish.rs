use super::{ZmqSocket, ZmqSocketOptions, ZmqSocketType};
use crate::{ZmqResult, sealed};

pub struct Publish {}

impl sealed::ZmqSenderFlag for Publish {}
impl sealed::ZmqSocketType for Publish {
    fn raw_socket_type() -> ZmqSocketType {
        ZmqSocketType::Publish
    }
}

unsafe impl Sync for ZmqSocket<Publish> {}
unsafe impl Send for ZmqSocket<Publish> {}

impl ZmqSocket<Publish> {
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::Conflate as i32, value)
    }

    pub fn conflate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::Conflate as i32)
    }

    pub fn set_invert_matching(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::InvertMatching as i32, value)
    }

    pub fn invert_matching(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(ZmqSocketOptions::InvertMatching as i32)
    }

    pub fn set_nodrop(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::XpubNoDrop as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn topic_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(ZmqSocketOptions::TopicsCount as i32)
    }

    pub fn bind<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.bind(endpoint.as_ref())
    }

    pub fn unbind<V: AsRef<str>>(&self, endpoint: V) -> ZmqResult<()> {
        self.socket.unbind(endpoint.as_ref())
    }
}
