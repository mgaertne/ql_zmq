use crate::{
    sealed,
    socket::{Socket, SocketType},
};

pub struct Pull {}

impl sealed::ReceiverFlag for Pull {}

unsafe impl Sync for Socket<Pull> {}
unsafe impl Send for Socket<Pull> {}

impl sealed::SocketType for Pull {
    fn raw_socket_type() -> SocketType {
        SocketType::Pull
    }
}

/// # A pull socket `ZMQ_PULL`
///
/// A socket of type [`Pull`] is used by a pipeline node to receive messages from upstream pipeline
/// nodes. Messages are fair-queued from among all connected upstream nodes. The
/// [`send_msg()`](method@super::Sender::send_msg()) function is not implemented for this socket
/// type.
impl Socket<Pull> {}
