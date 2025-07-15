use alloc::collections::{
    VecDeque,
    vec_deque::{Drain, IntoIter, Iter, IterMut},
};
use core::{
    fmt::{Display, Formatter},
    ops::{Deref, RangeBounds},
};

use derive_more::{Debug as DebugDeriveMore, Display as DisplayDeriveMore};
use parking_lot::FairMutex;

use crate::{ZmqResult, ffi::RawMessage, sealed, socket::Socket};

#[derive(DebugDeriveMore)]
#[debug("ZmqMessage {{ inner: {inner:?} }}")]
pub struct Message {
    inner: FairMutex<RawMessage>,
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

    pub fn bytes(&self) -> Vec<u8> {
        let msg_guard = self.inner.lock();
        (*msg_guard).deref().to_vec()
    }

    pub fn len(&self) -> usize {
        let msg_guard = self.inner.lock();
        msg_guard.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_more(&self) -> bool {
        let msg_guard = self.inner.lock();
        msg_guard.get_more()
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_routing_id(&self, value: u32) -> ZmqResult<()> {
        let mut msg_guard = self.inner.lock();
        msg_guard.set_routing_id(value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn routing_id(&self) -> Option<u32> {
        let msg_guard = self.inner.lock();
        msg_guard.routing_id()
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_group<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        let mut msg_guard = self.inner.lock();
        msg_guard.set_group(value.as_ref())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn group(&self) -> Option<String> {
        let msg_guard = self.inner.lock();
        msg_guard.group()
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::from_raw_msg(RawMessage::default())
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        let msg_guard = self.inner.lock();
        Message {
            inner: (*msg_guard).clone().into(),
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let msg_guard = self.inner.lock();
        write!(f, "{}", *msg_guard)
    }
}

impl<T: Into<RawMessage>> From<T> for Message {
    fn from(value: T) -> Self {
        let raw_msg = value.into();
        Self {
            inner: raw_msg.into(),
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
        let raw_msg = zmq_msg.inner.lock();

        socket.socket.send(&*raw_msg, flags)?;
        Ok(())
    }
}

#[derive(Default, DebugDeriveMore, DisplayDeriveMore)]
#[debug("ZmqMultipartMessage {{ {inner:?} }}")]
#[display("ZmqMultipartMessage {{ {inner:?} }}")]
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
