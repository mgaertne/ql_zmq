use crate::{
    ZmqResult, sealed,
    socket::{MultipartReceiver, Socket, SocketOption, SocketType},
};

/// # A pull socket `ZMQ_PULL`
///
/// A socket of type [`Pull`] is used by a pipeline node to receive messages from upstream pipeline
/// nodes. Messages are fair-queued from among all connected upstream nodes. The `send_msg()`
/// function is not implemented for this socket type.
///
/// [`Pull`]: PullSocket
pub type PullSocket = Socket<Pull>;

pub struct Pull {}

impl sealed::ReceiverFlag for Pull {}

unsafe impl Sync for Socket<Pull> {}
unsafe impl Send for Socket<Pull> {}

impl sealed::SocketType for Pull {
    fn raw_socket_type() -> SocketType {
        SocketType::Pull
    }
}

impl MultipartReceiver for Socket<Pull> {}

impl Socket<Pull> {
    /// # Keep only last message `ZMQ_CONFLATE`
    ///
    /// If set, a socket shall keep only one message in its inbound/outbound queue, this message
    /// being the last message received/the last message to be sent. Ignores
    /// [`receive_highwater_mark()`] and [`send_highwater_mark()`] options. Does not support
    /// multi-part messages, in particular, only one part of it is kept in the socket internal
    /// queue.
    ///
    /// # Note
    ///
    /// If [`recv_msg()`] is not called on the inbound socket, the queue and memory will grow with
    /// each message received. Use [`events()`] to trigger the conflation of the messages.
    ///
    /// [`receive_highwater_mark()`]: #method.receive_highwater_mark
    /// [`send_highwater_mark()`]: #method.send_highwater_mark
    /// [`recv_msg()`]: #method.recv_msg
    /// [`events()`]: #method.events
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOption::Conflate, value)
    }
}
