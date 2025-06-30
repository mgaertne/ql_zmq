use core::{
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::{Error, Result};
use zmq::{self, Message, SocketType};

use crate::{ZmqSocket, sealed::ZmqSocketType};

pub struct Dealer {}

unsafe impl Sync for ZmqSocket<Dealer> {}
unsafe impl Send for ZmqSocket<Dealer> {}

impl ZmqSocketType for Dealer {
    fn raw_socket_type() -> SocketType {
        zmq::DEALER
    }
}

impl ZmqSocket<Dealer> {
    /// The `ZMQ_BACKLOG` option shall set the maximum length of the queue of outstanding peer
    /// connections for the specified `socket`; this only applies to connection-oriented
    /// transports. For details refer to your operating system documentation for the `listen`
    /// function.
    pub fn set_backlog(&self, value: i32) -> Result<()> {
        self.socket.set_backlog(value).map_err(Error::from)
    }

    /// The `ZMQ_BACKLOG` option shall retrieve the maximum length of the queue of outstanding
    /// peer connections for the specified `socket`; this only applies to connection-oriented
    /// transports. For details refer to your operating system documentation for the `listen`
    /// function.
    pub fn backlog(&self) -> Result<i32> {
        self.socket.get_backlog().map_err(Error::from)
    }

    pub fn set_conflate(&self, value: bool) -> Result<()> {
        self.socket.set_conflate(value).map_err(Error::from)
    }

    pub fn conflate(&self) -> Result<bool> {
        self.socket.is_conflate().map_err(Error::from)
    }

    pub fn bind<V: AsRef<str>>(&self, endpoint: V) -> Result<()> {
        self.socket.bind(endpoint.as_ref()).map_err(Error::from)
    }

    pub fn unbind<V: AsRef<str>>(&self, endpoint: V) -> Result<()> {
        self.socket.unbind(endpoint.as_ref()).map_err(Error::from)
    }

    pub fn connect<V: AsRef<str>>(&self, endpoint: V) -> Result<()> {
        self.socket.connect(endpoint.as_ref()).map_err(Error::from)
    }

    pub fn disconnect<V: AsRef<str>>(&self, endpoint: V) -> Result<()> {
        self.socket
            .disconnect(endpoint.as_ref())
            .map_err(Error::from)
    }

    pub fn send<T: Into<Message>>(&self, msg: T, flags: i32) -> Result<()> {
        self.socket.send(msg, flags).map_err(Error::from)
    }

    pub fn recv(&self, flags: i32) -> Result<Message> {
        self.socket.recv_msg(flags).map_err(Error::from)
    }
}

impl Future for ZmqSocket<Dealer> {
    type Output = Message;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.socket
            .recv_msg(zmq::DONTWAIT)
            .map_or(Poll::Pending, Poll::Ready)
    }
}
