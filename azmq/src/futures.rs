//! Convenience traits and implementations for using 0MQ asynchronously
use core::{
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures::FutureExt;

use crate::{
    message::{ZmqMessage, ZmqSendable},
    sealed,
    socket::{Monitor, MonitorSocketEvent, ZmqReceiver, ZmqRecvFlags, ZmqSendFlags, ZmqSocket},
};

#[async_trait]
pub trait AsyncZmqReceiver<'a> {
    async fn recv_msg_async(&'a self) -> Option<ZmqMessage>;
    async fn recv_multipart_async(&'a self) -> Vec<Vec<u8>> {
        let mut result = vec![];

        loop {
            if let Some(item) = self.recv_msg_async().await {
                result.push(item.to_vec());

                if !item.get_more() {
                    return result;
                }
            }
        }
    }
}

#[async_trait]
impl<'a, T: sealed::ZmqSocketType + sealed::ZmqReceiverFlag + Unpin> AsyncZmqReceiver<'a>
    for ZmqSocket<T>
where
    ZmqSocket<T>: Sync,
{
    async fn recv_msg_async(&'a self) -> Option<ZmqMessage> {
        MessageReceivingFuture { receiver: self }.now_or_never()
    }
}

struct MessageReceivingFuture<'a, T: sealed::ZmqSocketType + sealed::ZmqReceiverFlag + Unpin> {
    receiver: &'a ZmqSocket<T>,
}

impl<T: sealed::ZmqSocketType + sealed::ZmqReceiverFlag + Unpin> Future
    for MessageReceivingFuture<'_, T>
{
    type Output = ZmqMessage;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.receiver
            .socket
            .recv(ZmqRecvFlags::DONT_WAIT.bits())
            .map(ZmqMessage::from_raw_msg)
            .map_or(Poll::Pending, Poll::Ready)
    }
}

#[async_trait]
pub trait AsyncZmqSender<'a, M, S: sealed::ZmqSocketType + sealed::ZmqSenderFlag + Unpin>
where
    M: Into<ZmqMessage> + Clone + Send,
{
    async fn send_msg_async(&'a self, msg: M, flags: ZmqSendFlags) -> Option<()>;

    async fn send_multipart_async<I>(&'a self, items: I, flags: ZmqSendFlags) -> Option<()>
    where
        I: Iterator + Send + 'a,
        I::Item: Into<M> + Send + 'a,
    {
        let mut last_part = None;
        for part in items {
            let maybe_last = last_part.take();
            if let Some(last) = maybe_last {
                self.send_msg_async(last, flags | ZmqSendFlags::SEND_MORE)
                    .await?;
            }
            last_part = Some(part.into());
        }
        if let Some(last) = last_part {
            self.send_msg_async(last, flags).await
        } else {
            None
        }
    }
}

#[async_trait]
impl<'a, S, M> AsyncZmqSender<'a, M, S> for ZmqSocket<S>
where
    for<'async_trait> M: Into<ZmqMessage> + Clone + Send + 'async_trait,
    S: sealed::ZmqSocketType + sealed::ZmqSenderFlag + Unpin,
    ZmqSocket<S>: Sync,
{
    async fn send_msg_async(&'a self, msg: M, flags: ZmqSendFlags) -> Option<()> {
        MessageSendingFuture {
            receiver: self,
            message: msg,
            flags,
        }
        .now_or_never()
    }
}

struct MessageSendingFuture<'a, S: sealed::ZmqSocketType + sealed::ZmqSenderFlag + Unpin, M>
where
    M: Into<ZmqMessage> + Clone + Send,
{
    receiver: &'a ZmqSocket<S>,
    message: M,
    flags: ZmqSendFlags,
}

impl<'a, S: sealed::ZmqSocketType + sealed::ZmqSenderFlag + Unpin, M> Future
    for MessageSendingFuture<'a, S, M>
where
    M: Into<ZmqMessage> + Clone + Send,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let message = self.message.clone().into();

        message
            .send(self.receiver, self.flags.bits())
            .map_or(Poll::Pending, Poll::Ready)
    }
}

#[async_trait]
pub trait AsyncMonitorReceiver<'a> {
    async fn recv_monitor_event_async(&'a self) -> Option<MonitorSocketEvent>;
}

#[async_trait]
impl<'a> AsyncMonitorReceiver<'a> for ZmqSocket<Monitor> {
    async fn recv_monitor_event_async(&'a self) -> Option<MonitorSocketEvent> {
        MonitorSocketEventFuture { receiver: self }.now_or_never()
    }
}

struct MonitorSocketEventFuture<'a, T: sealed::ZmqSocketType + sealed::ZmqReceiverFlag + Unpin> {
    receiver: &'a ZmqSocket<T>,
}

impl Future for MonitorSocketEventFuture<'_, Monitor> {
    type Output = MonitorSocketEvent;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self
            .receiver
            .recv_multipart(ZmqRecvFlags::DONT_WAIT)
            .map(MonitorSocketEvent::try_from)
        {
            Ok(Ok(event)) => Poll::Ready(event),
            _ => Poll::Pending,
        }
    }
}
