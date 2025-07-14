use super::{Socket, SocketOptions, SocketType};
use crate::{ZmqResult, sealed};

/// # A XSubscriber socket `ZMQ_XPUB`
///
/// Same as [`Publish`] except that you can receive subscriptions from the peers in form of
/// incoming messages. Subscription message is a byte 1 (for subscriptions) or byte 0 (for
/// unsubscriptions) followed by the subscription body. Messages without a sub/unsub prefix are
/// also received, but have no effect on subscription status.
///
/// A socket of type [`XPublish`] is used by a publisher to distribute data. Messages sent are
/// distributed in a fan out fashion to all connected peers.
///
/// When a [`XPublish`] sxocket enters the `mute` state due to having reached the high water mark
/// for a subscriber, then any messages that would be sent to the subscriber in question shall
/// instead be dropped until the mute state ends. The [`send_msg()`] function shall never block for
/// this socket type.
///
/// [`XPublish`]: XPublishSocket
/// [`Publish`]: super::PublishSocket
/// [`send_msg()`]: #impl-Sender<T>-for-Socket<T>
pub type XPublishSocket = Socket<XPublish>;

pub struct XPublish {}

impl sealed::SenderFlag for XPublish {}
impl sealed::ReceiverFlag for XPublish {}
impl sealed::SocketType for XPublish {
    fn raw_socket_type() -> SocketType {
        SocketType::XPublish
    }
}

unsafe impl Sync for Socket<XPublish> {}
unsafe impl Send for Socket<XPublish> {}

impl Socket<XPublish> {
    /// # Establish message filter `ZMQ_SUBSCRIBE`
    ///
    /// The [`subscribe()`] option shall establish a new message filter on a [`Subscriber`] socket.
    /// Newly created [`Subscriber`] sockets shall filter out all incoming messages, therefore you
    /// should call this option to establish an initial message filter.
    ///
    /// An empty `topic` of length zero shall subscribe to all incoming messages. A non-empty
    /// `topic` shall subscribe to all messages beginning with the specified prefix. Multiple
    /// filters may be attached to a single [`Subscriber`] socket, in which case a message shall
    /// be accepted if it matches at least one filter.
    ///
    /// [`Subscriber`]: super::SubscribeSocket
    /// [`subscribe()`]: #method.subscribe
    pub fn subscribe<V: AsRef<[u8]>>(&self, topic: V) -> ZmqResult<()> {
        self.set_sockopt_bytes(SocketOptions::Subscribe, topic.as_ref())
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
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: XPublishSocket
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
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: XPublishSocket
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
    /// [`XPublish`]: XPublishSocket
    /// [`sndhwm()`]: #method.sndhwm
    /// [`Again`]: crate::ZmqError::Again
    /// [`DONT_WAIT`]: super::SendFlags::DONT_WAIT
    pub fn set_nodrop(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::XpubNoDrop, value)
    }

    /// # pass duplicate subscribe messages on [`XPublish`] socket `ZMQ_XPUB_VERBOSE`
    ///
    /// Sets the [`XPublish`] socket behaviour on new duplicated subscriptions. If enabled, the
    /// socket passes all subscribe messages to the caller. If disabled, only the first
    /// subscription to each filter will be passed. The default is `false` (disabled).
    ///
    /// [`XPublish`]: XPublishSocket
    pub fn set_verbose(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::XpubVerbose, value)
    }

    /// # pass duplicate subscribe and unsubscribe messages on [`XPublish`] socket `ZMQ_XPUB_VERBOSER`
    ///
    /// Sets the [`XPublish`] socket behaviour on new duplicated subscriptions and unsubscriptions.
    /// If enabled, the socket passes all subscribe and unsubscribe messages to the caller. If
    /// disabled, only the first subscription to each filter and the last unsubscription from each
    /// filter will be passed. The default is `false` (disabled).
    ///
    /// [`XPublish`]: XPublishSocket
    pub fn set_verboser(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::XpubVerboser, value)
    }

    /// # change the subscription handling to manual `ZMQ_XPUB_MANUAL`
    ///
    /// Sets the [`XPublish`] socket subscription handling mode manual/automatic. A value of
    /// `false` is the default and subscription requests will be handled automatically. A value of
    /// `true` will change the subscription requests handling to manual, with manual mode
    /// subscription requests are not added to the subscription list. To add subscription the user
    /// need to call [`subscribe()`] on [`XPublish`] socket.
    ///
    /// [`XPublish`]: XPublishSocket
    /// [`subscribe()`]: #method.subscribe
    pub fn set_manual(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::XpubManual, value)
    }

    /// # change the subscription handling to manual `ZMQ_XPUB_MANUAL_LAST_VALUE`
    ///
    /// This option is similar to [`set_manual()`]. The difference is that
    /// [`set_manual_last_value()`] changes the [`XPublish`] socket behaviour to send the first
    /// message to the last subscriber after the socket receives a subscription and call setsockopt
    /// with [`subscribe()`] on [`XPublish`] socket. This prevents duplicated messages when using
    /// last value caching (LVC).
    ///
    /// [`XPublish`]: XPublishSocket
    /// [`set_manual_last_value()`]: #method.set_manual_last_value
    /// [`set_manual()`]: #method.set_manual
    /// [`subscribe()`]: #method.subscribe
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_manual_last_value(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::XpubManualLastValue, value)
    }

    /// # set welcome message that will be received by subscriber when connecting `ZMQ_XPUB_WELCOME_MSG`
    ///
    /// Sets a welcome message that will be received by subscriber when connecting. Subscriber must
    /// subscribe to the Welcome message before connecting. Welcome message will also be sent on
    /// reconnecting. For welcome message to work well the user must poll on incoming subscription
    /// messages on the [`XPublish`] socket and handle them.
    ///
    /// Use a length of zero to disable welcome message.
    ///
    /// [`XPublish`]: XPublishSocket
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_welcome_msg<V: AsRef<str>>(&self, value: V) -> ZmqResult<()> {
        self.set_sockopt_string(SocketOptions::XpubWelcomeMessage, value)
    }

    /// # Process only first subscribe/unsubscribe in a multipart message `ZMQ_ONLY_FIRST_SUBSCRIBE`
    ///
    /// If set, only the first part of the multipart message is processed as a
    /// subscribe/unsubscribe message. The rest are forwarded as user data regardless of message
    /// contents.
    ///
    /// It not set (default), subscribe/unsubscribe messages in a multipart message are processed
    /// as such regardless of their number and order.
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_only_first_subscribe(&self, value: bool) -> ZmqResult<()> {
        self.set_sockopt_bool(SocketOptions::OnlyFirstSubscribe, value)
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
    /// [`Publish`]: super::PublishSocket
    /// [`XPublish`]: XPublishSocket
    /// [`XSubscribe`]: super::XSubscribeSocket
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn topic_count(&self) -> ZmqResult<i32> {
        self.get_sockopt_int(SocketOptions::TopicsCount)
    }
}
