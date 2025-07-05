use alloc::sync::Arc;
use core::ops::Deref;

use derive_more::{Debug, Display};

use crate::{
    ZmqResult,
    ffi::RawMessage,
    sealed::{ZmqSenderFlag, ZmqSocketType},
    socket::ZmqSocket,
};

#[derive(Debug, Display)]
#[debug("ZmqMessage {{ ... }}")]
#[display("{inner}")]
pub struct ZmqMessage {
    inner: Arc<RawMessage>,
}

unsafe impl Send for ZmqMessage {}
unsafe impl Sync for ZmqMessage {}

impl ZmqMessage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size(len: usize) -> ZmqResult<Self> {
        Ok(Self::from_raw_msg(RawMessage::with_size(len)))
    }

    pub(crate) fn from_raw_msg(raw_msg: RawMessage) -> Self {
        Self {
            inner: raw_msg.into(),
        }
    }

    pub fn get_more(&self) -> bool {
        self.inner.get_more()
    }
}

impl Deref for ZmqMessage {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl Default for ZmqMessage {
    fn default() -> Self {
        Self::from_raw_msg(RawMessage::default())
    }
}

impl Clone for ZmqMessage {
    fn clone(&self) -> Self {
        ZmqMessage {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Into<RawMessage>> From<T> for ZmqMessage {
    fn from(value: T) -> Self {
        let raw_msg = value.into();
        Self {
            inner: Arc::new(raw_msg),
        }
    }
}

pub trait ZmqSendable<S: ZmqSocketType + ZmqSenderFlag> {
    fn send(self, socket: &ZmqSocket<S>, flags: i32) -> ZmqResult<()>;
}

impl<M, S: ZmqSocketType + ZmqSenderFlag> ZmqSendable<S> for M
where
    M: Into<ZmqMessage>,
{
    fn send(self, socket: &ZmqSocket<S>, flags: i32) -> ZmqResult<()> {
        let zmq_msg = self.into();
        let raw_msg = zmq_msg.inner;

        socket.socket.send(&**raw_msg, flags)?;
        Ok(())
    }
}
