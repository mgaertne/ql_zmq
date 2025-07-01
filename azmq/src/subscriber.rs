use anyhow::{Error, Result};

use crate::{
    ZmqSocket,
    sealed::{ZmqReceiver, ZmqSocketType},
};

pub struct Subscriber {}

impl ZmqReceiver for Subscriber {}

unsafe impl Sync for ZmqSocket<Subscriber> {}
unsafe impl Send for ZmqSocket<Subscriber> {}

impl ZmqSocketType for Subscriber {
    fn raw_socket_type() -> zmq::SocketType {
        zmq::SocketType::SUB
    }
}

impl ZmqSocket<Subscriber> {
    pub fn set_conflate(&self, value: bool) -> Result<()> {
        self.socket.set_conflate(value).map_err(Error::from)
    }

    pub fn conflate(&self) -> Result<bool> {
        self.socket.is_conflate().map_err(Error::from)
    }

    pub fn subscribe<V: AsRef<[u8]>>(&self, topic: V) -> Result<()> {
        self.socket
            .set_subscribe(topic.as_ref())
            .map_err(Error::from)
    }

    pub fn unsubscribe<V: AsRef<[u8]>>(&self, topic: V) -> Result<()> {
        self.socket
            .set_unsubscribe(topic.as_ref())
            .map_err(Error::from)
    }

    pub fn connect<V: AsRef<str>>(&self, endpoint: V) -> Result<()> {
        self.socket.connect(endpoint.as_ref()).map_err(Error::from)
    }

    pub fn disconnect<V: AsRef<str>>(&self, endpoint: V) -> Result<()> {
        self.socket
            .disconnect(endpoint.as_ref())
            .map_err(Error::from)
    }
}
