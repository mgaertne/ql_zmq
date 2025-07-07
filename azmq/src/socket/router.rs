#[cfg(feature = "draft-api")]
use bitflags::bitflags;

use crate::{
    ZmqResult, sealed,
    socket::{ZmqSocket, ZmqSocketOptions, ZmqSocketType},
};

pub struct Router {}

impl sealed::ZmqSenderFlag for Router {}
impl sealed::ZmqReceiverFlag for Router {}

impl sealed::ZmqSocketType for Router {
    fn raw_socket_type() -> ZmqSocketType {
        ZmqSocketType::Router
    }
}

unsafe impl Sync for ZmqSocket<Router> {}
unsafe impl Send for ZmqSocket<Router> {}

#[cfg(feature = "draft-api")]
#[doc(cfg(feature = "draft-api"))]
pub struct RouterNotify(i32);

#[cfg(feature = "draft-api")]
bitflags! {
    impl RouterNotify: i32 {
        const NotifyConnect    = 0b0000_0000_0000_0001;
        const NotifyDisconnect = 0b0000_0000_0000_0010;
    }
}

impl ZmqSocket<Router> {
    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(ZmqSocketOptions::RoutingId as i32)
    }

    pub fn set_connect_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::ConnectRoutingId as i32, value)
    }

    pub fn set_router_handover(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::RouterHandover as i32, value)
    }

    pub fn set_router_mandatory(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(ZmqSocketOptions::RouterMandatory as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_disconnect_message<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::DisconnectMessage as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_hello_message<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(ZmqSocketOptions::HelloMessage as i32, value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_router_notify(&self, value: RouterNotify) -> ZmqResult<()> {
        self.set_sockopt_int(ZmqSocketOptions::RouterNotify as i32, value.bits())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn router_notify(&self) -> ZmqResult<RouterNotify> {
        self.get_sockopt_int(ZmqSocketOptions::RouterNotify as i32)
            .map(RouterNotify::from_bits_truncate)
    }
}
