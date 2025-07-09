use alloc::{
    collections::{
        VecDeque,
        vec_deque::{Drain, IntoIter, Iter, IterMut},
    },
    sync::Arc,
};
use core::ops::{Deref, RangeBounds};

use derive_more::{Debug as DebugDeriveMore, Display as DisplayDeriveMore};

use crate::{ZmqResult, ffi::RawMessage, sealed, socket::ZmqSocket};

#[derive(DebugDeriveMore, DisplayDeriveMore)]
#[debug("ZmqMessage {{ inner: {inner} }}")]
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

pub trait ZmqSendable<S: sealed::ZmqSocketType + sealed::ZmqSenderFlag> {
    fn send(self, socket: &ZmqSocket<S>, flags: i32) -> ZmqResult<()>;
}

impl<M, S: sealed::ZmqSocketType + sealed::ZmqSenderFlag> ZmqSendable<S> for M
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

#[derive(Default, DebugDeriveMore, DisplayDeriveMore)]
#[debug("ZmqMultipartMessage {{ ... }}")]
#[display("ZmqMultipartMessage {{ ... }}")]
pub struct ZmqMultipartMessage {
    inner: VecDeque<ZmqMessage>,
}

unsafe impl Send for ZmqMultipartMessage {}
unsafe impl Sync for ZmqMultipartMessage {}

impl ZmqMultipartMessage {
    pub fn new() -> Self {
        ZmqMultipartMessage::default()
    }

    pub fn into_inner(self) -> VecDeque<ZmqMessage> {
        self.inner
    }

    pub fn get(&self, index: usize) -> Option<&ZmqMessage> {
        self.inner.get(index)
    }

    pub fn pop_front(&mut self) -> Option<ZmqMessage> {
        self.inner.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<ZmqMessage> {
        self.inner.pop_back()
    }

    pub fn push_front(&mut self, msg: ZmqMessage) {
        self.inner.push_front(msg)
    }

    pub fn push_back(&mut self, msg: ZmqMessage) {
        self.inner.push_back(msg)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, ZmqMessage> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, ZmqMessage> {
        self.inner.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn drain<R>(&mut self, range: R) -> Drain<'_, ZmqMessage>
    where
        R: RangeBounds<usize>,
    {
        self.inner.drain(range)
    }
}

impl From<ZmqMessage> for ZmqMultipartMessage {
    fn from(msg: ZmqMessage) -> Self {
        let mut multipart = ZmqMultipartMessage::new();
        multipart.push_back(msg);
        multipart
    }
}

impl From<Vec<ZmqMessage>> for ZmqMultipartMessage {
    fn from(v: Vec<ZmqMessage>) -> Self {
        ZmqMultipartMessage { inner: v.into() }
    }
}

impl<'a> IntoIterator for &'a ZmqMultipartMessage {
    type IntoIter = Iter<'a, ZmqMessage>;
    type Item = &'a ZmqMessage;

    fn into_iter(self) -> Iter<'a, ZmqMessage> {
        self.iter()
    }
}

impl IntoIterator for ZmqMultipartMessage {
    type IntoIter = IntoIter<ZmqMessage>;
    type Item = ZmqMessage;

    fn into_iter(self) -> IntoIter<ZmqMessage> {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut ZmqMultipartMessage {
    type IntoIter = IterMut<'a, ZmqMessage>;
    type Item = &'a mut ZmqMessage;

    fn into_iter(self) -> IterMut<'a, ZmqMessage> {
        self.iter_mut()
    }
}
