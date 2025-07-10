use alloc::{
    collections::{
        VecDeque,
        vec_deque::{Drain, IntoIter, Iter, IterMut},
    },
    sync::Arc,
};
use core::ops::{Deref, RangeBounds};

use derive_more::{Debug as DebugDeriveMore, Display as DisplayDeriveMore};

use crate::{ZmqResult, ffi::RawMessage, sealed, socket::Socket};

#[derive(DebugDeriveMore, DisplayDeriveMore)]
#[debug("ZmqMessage {{ inner: {inner} }}")]
#[display("{inner}")]
pub struct Message {
    inner: Arc<RawMessage>,
}

unsafe impl Send for Message {}
unsafe impl Sync for Message {}

impl Message {
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

impl Deref for Message {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::from_raw_msg(RawMessage::default())
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Message {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Into<RawMessage>> From<T> for Message {
    fn from(value: T) -> Self {
        let raw_msg = value.into();
        Self {
            inner: Arc::new(raw_msg),
        }
    }
}

pub trait Sendable<S: sealed::SocketType + sealed::SenderFlag> {
    fn send(self, socket: &Socket<S>, flags: i32) -> ZmqResult<()>;
}

impl<M, S: sealed::SocketType + sealed::SenderFlag> Sendable<S> for M
where
    M: Into<Message>,
{
    fn send(self, socket: &Socket<S>, flags: i32) -> ZmqResult<()> {
        let zmq_msg = self.into();
        let raw_msg = zmq_msg.inner;

        socket.socket.send(&**raw_msg, flags)?;
        Ok(())
    }
}

#[derive(Default, DebugDeriveMore, DisplayDeriveMore)]
#[debug("ZmqMultipartMessage {{ ... }}")]
#[display("ZmqMultipartMessage {{ ... }}")]
pub struct MultipartMessage {
    inner: VecDeque<Message>,
}

unsafe impl Send for MultipartMessage {}
unsafe impl Sync for MultipartMessage {}

impl MultipartMessage {
    pub fn new() -> Self {
        MultipartMessage::default()
    }

    pub fn into_inner(self) -> VecDeque<Message> {
        self.inner
    }

    pub fn get(&self, index: usize) -> Option<&Message> {
        self.inner.get(index)
    }

    pub fn pop_front(&mut self) -> Option<Message> {
        self.inner.pop_front()
    }

    pub fn pop_back(&mut self) -> Option<Message> {
        self.inner.pop_back()
    }

    pub fn push_front(&mut self, msg: Message) {
        self.inner.push_front(msg)
    }

    pub fn push_back(&mut self, msg: Message) {
        self.inner.push_back(msg)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, Message> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Message> {
        self.inner.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn drain<R>(&mut self, range: R) -> Drain<'_, Message>
    where
        R: RangeBounds<usize>,
    {
        self.inner.drain(range)
    }
}

impl From<Message> for MultipartMessage {
    fn from(msg: Message) -> Self {
        let mut multipart = MultipartMessage::new();
        multipart.push_back(msg);
        multipart
    }
}

impl From<Vec<Message>> for MultipartMessage {
    fn from(v: Vec<Message>) -> Self {
        MultipartMessage { inner: v.into() }
    }
}

impl<'a> IntoIterator for &'a MultipartMessage {
    type IntoIter = Iter<'a, Message>;
    type Item = &'a Message;

    fn into_iter(self) -> Iter<'a, Message> {
        self.iter()
    }
}

impl IntoIterator for MultipartMessage {
    type IntoIter = IntoIter<Message>;
    type Item = Message;

    fn into_iter(self) -> IntoIter<Message> {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut MultipartMessage {
    type IntoIter = IterMut<'a, Message>;
    type Item = &'a mut Message;

    fn into_iter(self) -> IterMut<'a, Message> {
        self.iter_mut()
    }
}
