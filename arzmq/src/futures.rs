//! Convenience traits and implementations for using 0MQ asynchronously
use core::{
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures::FutureExt;

use crate::{
    message::{Message, MultipartMessage, Sendable},
    sealed, socket,
    socket::{
        MonitorSocket, MonitorSocketEvent, MultipartReceiver, MultipartSender, RecvFlags,
        SendFlags, Socket,
    },
};

#[async_trait]
pub trait AsyncReceiver<'a> {
    async fn recv_msg_async(&'a self) -> Option<Message>;
}

#[async_trait]
pub trait AsyncMultipartReceiver<'a>: AsyncReceiver<'a> {
    async fn recv_multipart_async(&'a self) -> MultipartMessage {
        let mut result = MultipartMessage::new();

        loop {
            if let Some(item) = self.recv_msg_async().await {
                let got_more = item.get_more();
                result.push_back(item);

                if !got_more {
                    return result;
                }
            }
        }
    }
}

#[async_trait]
impl<'a, T: sealed::SocketType + sealed::ReceiverFlag + Unpin> AsyncReceiver<'a> for Socket<T>
where
    Socket<T>: Sync,
{
    async fn recv_msg_async(&'a self) -> Option<Message> {
        MessageReceivingFuture { receiver: self }.now_or_never()
    }
}

struct MessageReceivingFuture<'a, T: sealed::SocketType + sealed::ReceiverFlag + Unpin> {
    receiver: &'a Socket<T>,
}

impl<T: sealed::SocketType + sealed::ReceiverFlag + Unpin> Future
    for MessageReceivingFuture<'_, T>
{
    type Output = Message;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Self::Output> {
        self.receiver
            .socket
            .recv(RecvFlags::DONT_WAIT.bits())
            .map(Message::from_raw_msg)
            .map_or(Poll::Pending, Poll::Ready)
    }
}

#[async_trait]
impl<'a, T: MultipartReceiver<RecvFlags> + AsyncReceiver<'a>> AsyncMultipartReceiver<'a> for T {}

#[async_trait]
pub trait AsyncSender<'a, S: sealed::SocketType + sealed::SenderFlag + Unpin> {
    async fn send_msg_async(&'a self, msg: Message, flags: SendFlags) -> Option<()>;
}

#[async_trait]
pub trait AsyncMultipartSender<'a, S: sealed::SocketType + sealed::SenderFlag + Unpin>:
    AsyncSender<'a, S>
{
    async fn send_multipart_async(
        &'a self,
        multipart: MultipartMessage,
        flags: SendFlags,
    ) -> Option<()> {
        let mut last_part = None;
        for part in multipart {
            let maybe_last = last_part.take();
            if let Some(last) = maybe_last {
                self.send_msg_async(last, flags | SendFlags::SEND_MORE)
                    .await?;
            }
            last_part = Some(part);
        }
        if let Some(last) = last_part {
            self.send_msg_async(last, flags).await
        } else {
            None
        }
    }
}

#[async_trait]
impl<'a, S> AsyncSender<'a, S> for Socket<S>
where
    S: sealed::SocketType + sealed::SenderFlag + Unpin,
    Socket<S>: Sync,
{
    async fn send_msg_async(&'a self, msg: Message, flags: SendFlags) -> Option<()> {
        MessageSendingFuture {
            receiver: self,
            message: msg,
            flags,
        }
        .now_or_never()
    }
}

struct MessageSendingFuture<'a, S: sealed::SocketType + sealed::SenderFlag + Unpin, M>
where
    M: Into<Message> + Send,
{
    receiver: &'a Socket<S>,
    message: M,
    flags: SendFlags,
}

impl<'a, S: sealed::SocketType + sealed::SenderFlag + Unpin, M> Future
    for MessageSendingFuture<'a, S, M>
where
    M: Into<Message> + Clone + Send,
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
impl<
    'a,
    S: sealed::SenderFlag + sealed::SocketType + Unpin,
    T: MultipartSender<S> + AsyncSender<'a, S>,
> AsyncMultipartSender<'a, S> for T
{
}

#[async_trait]
pub trait AsyncMonitorReceiver<'a> {
    async fn recv_monitor_event_async(&'a self) -> Option<MonitorSocketEvent>;
}

#[async_trait]
impl<'a> AsyncMonitorReceiver<'a> for MonitorSocket {
    async fn recv_monitor_event_async(&'a self) -> Option<MonitorSocketEvent> {
        MonitorSocketEventFuture { receiver: self }.now_or_never()
    }
}

struct MonitorSocketEventFuture<'a, T: sealed::SocketType + sealed::ReceiverFlag + Unpin> {
    receiver: &'a Socket<T>,
}

impl Future for MonitorSocketEventFuture<'_, socket::monitor::Monitor> {
    type Output = MonitorSocketEvent;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self
            .receiver
            .recv_multipart(RecvFlags::DONT_WAIT)
            .map(MonitorSocketEvent::try_from)
        {
            Ok(Ok(event)) => Poll::Ready(event),
            _ => Poll::Pending,
        }
    }
}
