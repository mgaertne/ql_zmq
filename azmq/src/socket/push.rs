use crate::{
    sealed,
    socket::{Socket, SocketType},
};

pub struct Push {}

impl sealed::SenderFlag for Push {}

unsafe impl Sync for Socket<Push> {}
unsafe impl Send for Socket<Push> {}

impl sealed::SocketType for Push {
    fn raw_socket_type() -> SocketType {
        SocketType::Push
    }
}

/// # A push socket `ZMQ_PUSH`
///
/// A socket of type [`Push`] is used by a pipeline node to send messages to downstream pipeline
/// nodes. Messages are round-robined to all connected downstream nodes. The
/// [`recv_msg()`](method@super::Receiver::recv_msg()) function is not implemented for this socket
/// type.
///
/// When a [`Push`] socket enters the 'mute' state due to having reached the high water mark for
/// all downstream nodes, or, for connection-oriented transports, if the
/// [`Immediate`](variant@super::SocketOptions::Immediate) option is set and there are no
/// downstream nodes at all, then any [`send_msg()`](method@super::Sender::send_msg()) operations
/// on the socket shall block until the mute state ends or at least one downstream node becomes
/// available for sending; messages are not discarded.
impl Socket<Push> {}
