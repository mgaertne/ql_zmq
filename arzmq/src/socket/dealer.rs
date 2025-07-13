use super::{Socket, SocketOptions, SocketType};
use crate::{ZmqResult, sealed};

/// # A dealer socket `ZMQ_DEALER`
///
/// A socket of type [`Dealer`] is an advanced pattern used for extending request/reply sockets.
/// Each message sent is round-robined among all connected peers, and each message received is
/// fair-queued from all connected peers.
///
/// When a [`Dealer`] socket enters the 'mute' state due to having reached the high water mark for
/// all peers, or, for connection-oriented transports, if the [`immediate()`] option is set and
/// there are no peers at /// all, then any [`send_msg()`] operations on the socket shall block
/// until the mute state ends or at least one peer becomes available for sending; messages are not
/// discarded.
///
/// When a [`Dealer`] socket is connected to a [`Reply`](type@super::ReplySocket) socket each
/// message sent must consist of an empty message part, the delimiter, followed by one or more body
/// parts.
///
/// [`Dealer`]: DealerSocket
/// [`immediate()`]: #method.immediate
/// [`send_msg()`]: #impl-Sender<T>-for-Socket<T>
pub type DealerSocket = Socket<Dealer>;

pub struct Dealer {}

impl sealed::SenderFlag for Dealer {}
impl sealed::ReceiverFlag for Dealer {}

impl sealed::SocketType for Dealer {
    fn raw_socket_type() -> SocketType {
        SocketType::Dealer
    }
}

unsafe impl Sync for Socket<Dealer> {}
unsafe impl Send for Socket<Dealer> {}

impl Socket<Dealer> {
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::Conflate as i32, value)
    }

    pub fn conflate(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::Conflate as i32)
    }

    pub fn set_routing_id<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::RoutingId as i32, value)
    }

    pub fn routing_id(&self) -> ZmqResult<String> {
        self.get_sockopt_string(SocketOptions::RoutingId as i32)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_hello_message<T: AsRef<str>>(&self, value: T) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::HelloMessage as i32, value)
    }
}
