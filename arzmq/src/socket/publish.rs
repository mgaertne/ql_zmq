use super::{MultipartSender, Socket, SocketOptions, SocketType};
use crate::{ZmqResult, sealed};

/// # A Subscriber socket `ZMQ_PUB`
///
/// A socket of type [`Publish`] is used by a publisher to distribute data. Messages sent are
/// distributed in a fan out fashion to all connected peers.
///
/// When a [`Publish`] socket enters the `mute` state due to having reached the high water mark
/// for a subscriber, then any messages that would be sent to the subscriber in question shall
/// instead be dropped until the mute state ends. The [`send_msg()`] function shall never block for
/// this socket type.
///
/// [`Publish`]: PublishSocket
/// [`send_msg()`]: #impl-Sender<T>-for-Socket<T>
pub type PublishSocket = Socket<Publish>;

pub struct Publish {}

impl sealed::SenderFlag for Publish {}
impl sealed::SocketType for Publish {
    fn raw_socket_type() -> SocketType {
        SocketType::Publish
    }
}

unsafe impl Sync for Socket<Publish> {}
unsafe impl Send for Socket<Publish> {}

impl MultipartSender<Publish> for Socket<Publish> {}

impl Socket<Publish> {
    /// # Keep only last message `ZMQ_CONFLATE`
    ///
    /// If set, a socket shall keep only one message in its inbound/outbound queue, this message
    /// being the last message received/the last message to be sent. Ignores [`recvhwm()`] and
    /// [`sndhwm()`] options. Does not support multi-part messages, in particular, only one part of
    /// it is kept in the socket internal queue.
    ///
    /// # Note
    ///
    /// If [`recv_msg()`] is not called on the inbound socket, the queue and memory will grow with
    /// each message received. Use [`events()`] to trigger the conflation of the messages.
    ///
    /// [`recvhwm()`]: #method.recvhwm
    /// [`sndhwm()`]: #method.sndhwm
    /// [`recv_msg()`]: #method.recv_msg
    /// [`events()`]: #method.events
    pub fn set_conflate(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::Conflate, value)
    }

    /// Invert message filtering `ZMQ_INVERT_MATCHING`
    /// Reverses the filtering behavior of [`Publish`]-[`Subscribe`] sockets, when set to `true`.
    ///
    /// On [`Publish`] and [`XPublish`] sockets, this causes messages to be sent to all connected
    /// sockets *except* those subscribed to a prefix that matches the message. On [`Subscribe`]
    /// sockets, this causes only incoming messages that do *not* match any of the socket’s
    /// subscriptions to be received by the user.
    ///
    /// Whenever `ZMQ_INVERT_MATCHING` is set to `true` on a [`Publish`] socket, all [`Subscribe`]
    /// sockets connecting to it must also have the option set to `true`. Failure to do so will
    /// have the [`Subscribe`] sockets reject everything the [`Publish`] socket sends them.
    /// [`XSubscribe`] sockets do not need to do this because they do not filter incoming messages.
    ///
    /// [`Subscribe`]: super::SubscribeSocket
    /// [`Publish`]: PublishSocket
    /// [`XPublish`]: super::XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    pub fn set_invert_matching(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::InvertMatching, value)
    }

    /// Retrieve inverted filtering status `ZMQ_INVERT_MATCHING`
    ///
    /// Returns the value of the `ZMQ_INVERT_MATCHING` option. A value of `true` means the socket
    /// uses inverted prefix matching.
    ///
    /// On [`Publish`] and [`XPublish`] sockets, this causes messages to be sent to all connected
    /// sockets *except* those subscribed to a prefix that matches the message. On [`Subscribe`]
    /// sockets, this causes only incoming messages that do *not* match any of the socket’s
    /// subscriptions to be received by the user.
    ///
    /// Whenever `ZMQ_INVERT_MATCHING` is set to `true` on a [`Publish`] socket, all [`Publish`]
    /// sockets connecting to it must also have the option set to `true`. Failure to do so will
    /// have the [`Subscribe`] sockets reject everything the [`Publish`] socket sends them.
    /// [`XSubscribe`] sockets do not need to do this because they do not filter incoming messages.
    ///
    /// [`Subscribe`]: super::SubscribeSocket
    /// [`Publish`]: PublishSocket
    /// [`XPublish`]: super::XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    pub fn invert_matching(&self) -> ZmqResult<bool> {
        self.get_sockopt_bool(SocketOptions::InvertMatching)
    }

    /// # do not silently drop messages if [`sndhwm()`] is reached `ZMQ_XPUB_NODROP`
    ///
    /// Sets the [`XPublish`] socket behaviour to return error [`Again`] if [`sndhwm()`] is
    /// reached and the message could not be send.
    ///
    /// A value of `false` is the default and drops the message silently when the peers [`sndhwm()`]
    /// is reached. A value of `true` returns an [`Again`] error code if the [`sndhwm()`] is
    /// reached and [`DONT_WAIT`] was used.
    ///
    /// [`XPublish`]: super::XPublishSocket
    /// [`sndhwm()`]: #method.sndhwm
    /// [`Again`]: crate::ZmqError::Again
    /// [`DONT_WAIT`]: super::SendFlags::DONT_WAIT
    pub fn set_nodrop(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::XpubNoDrop, value)
    }

    /// # Number of topic subscriptions received `ZMQ_TOPICS_COUNT`
    ///
    /// Gets the number of topic (prefix) subscriptions either
    ///
    /// * received on a [`Publish`]/[`XPublish`] socket from all the connected
    ///   [`Subscribe`]/[`XSubscribe`] sockets or
    /// * acknowledged on an [`Publish`]/[`XPublish`] socket from all the connected
    ///   [`Subscribe`]/[`XSubscribe`] sockets
    ///
    /// [`Subscribe`]: super::SubscribeSocket
    /// [`Publish`]: PublishSocket
    /// [`XPublish`]: super::XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn topic_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::TopicsCount)
    }
}
