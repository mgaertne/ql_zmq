use alloc::sync::Arc;

#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use builder::{ContextConfig, ContextConfigBuilder};
use derive_more::{Debug as DebugDeriveMore, Display as DisplayDeriveMore};

use crate::{ZmqResult, ffi::RawContext, zmq_sys_crate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SetContextOption {
    IoThreads = zmq_sys_crate::ZMQ_IO_THREADS as i32,
    MaxSockets = zmq_sys_crate::ZMQ_MAX_SOCKETS as i32,
    ThreadPriority = zmq_sys_crate::ZMQ_THREAD_PRIORITY as i32,
    ThreadSchedulingPolicy = zmq_sys_crate::ZMQ_THREAD_SCHED_POLICY as i32,
    MaxMessageSize = zmq_sys_crate::ZMQ_MAX_MSGSZ as i32,
    ThreadAffinityCPUAdd = zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_ADD as i32,
    ThreadAffinityCPURemove = zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_REMOVE as i32,
    ThreadNamePrefix = zmq_sys_crate::ZMQ_THREAD_NAME_PREFIX as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    ZeroCopyReceiving = zmq_sys_crate::ZMQ_ZERO_COPY_RECV as i32,
    IPv6 = zmq_sys_crate::ZMQ_IPV6 as i32,
    Blocky = zmq_sys_crate::ZMQ_BLOCKY as i32,
}

impl From<SetContextOption> for i32 {
    fn from(value: SetContextOption) -> Self {
        match value {
            SetContextOption::Blocky => zmq_sys_crate::ZMQ_BLOCKY as i32,
            SetContextOption::IoThreads => zmq_sys_crate::ZMQ_IO_THREADS as i32,
            SetContextOption::ThreadSchedulingPolicy => {
                zmq_sys_crate::ZMQ_THREAD_SCHED_POLICY as i32
            }
            SetContextOption::ThreadPriority => zmq_sys_crate::ZMQ_THREAD_PRIORITY as i32,
            SetContextOption::ThreadAffinityCPUAdd => {
                zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_ADD as i32
            }
            SetContextOption::ThreadAffinityCPURemove => {
                zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_REMOVE as i32
            }
            SetContextOption::ThreadNamePrefix => zmq_sys_crate::ZMQ_THREAD_NAME_PREFIX as i32,
            SetContextOption::MaxMessageSize => zmq_sys_crate::ZMQ_MAX_MSGSZ as i32,
            SetContextOption::MaxSockets => zmq_sys_crate::ZMQ_MAX_SOCKETS as i32,
            SetContextOption::IPv6 => zmq_sys_crate::ZMQ_IPV6 as i32,
            #[cfg(feature = "draft-api")]
            SetContextOption::ZeroCopyReceiving => zmq_sys_crate::ZMQ_ZERO_COPY_RECV as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum GetContextOption {
    IoThreads = zmq_sys_crate::ZMQ_IO_THREADS as i32,
    MaxSockets = zmq_sys_crate::ZMQ_MAX_SOCKETS as i32,
    SocketLimit = zmq_sys_crate::ZMQ_SOCKET_LIMIT as i32,
    ThreadSchedulingPolicy = zmq_sys_crate::ZMQ_THREAD_SCHED_POLICY as i32,
    MaxMessageSize = zmq_sys_crate::ZMQ_MAX_MSGSZ as i32,
    ThreadNamePrefix = zmq_sys_crate::ZMQ_THREAD_NAME_PREFIX as i32,
    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    ZeroCopyReceiving = zmq_sys_crate::ZMQ_ZERO_COPY_RECV as i32,
    IPv6 = zmq_sys_crate::ZMQ_IPV6 as i32,
    Blocky = zmq_sys_crate::ZMQ_BLOCKY as i32,
}

impl From<GetContextOption> for i32 {
    fn from(value: GetContextOption) -> Self {
        match value {
            GetContextOption::Blocky => zmq_sys_crate::ZMQ_BLOCKY as i32,
            GetContextOption::IoThreads => zmq_sys_crate::ZMQ_IO_THREADS as i32,
            GetContextOption::ThreadSchedulingPolicy => {
                zmq_sys_crate::ZMQ_THREAD_SCHED_POLICY as i32
            }
            GetContextOption::ThreadNamePrefix => zmq_sys_crate::ZMQ_THREAD_NAME_PREFIX as i32,
            GetContextOption::MaxMessageSize => zmq_sys_crate::ZMQ_MAX_MSGSZ as i32,
            GetContextOption::MaxSockets => zmq_sys_crate::ZMQ_MAX_SOCKETS as i32,
            GetContextOption::IPv6 => zmq_sys_crate::ZMQ_IPV6 as i32,
            GetContextOption::SocketLimit => zmq_sys_crate::ZMQ_SOCKET_LIMIT as i32,
            #[cfg(feature = "draft-api")]
            GetContextOption::ZeroCopyReceiving => zmq_sys_crate::ZMQ_ZERO_COPY_RECV as i32,
        }
    }
}

#[derive(DebugDeriveMore, DisplayDeriveMore)]
#[debug("ZmqContext {{ ... }}")]
#[display("ZmqContext")]
pub struct Context {
    pub(crate) inner: Arc<RawContext>,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Context {
    pub fn new() -> ZmqResult<Self> {
        let inner = RawContext::new()?;
        Ok(Self::from_raw_context(inner))
    }

    pub(crate) fn from_raw_context(raw_context: RawContext) -> Self {
        Self {
            inner: raw_context.into(),
        }
    }

    pub(crate) fn as_raw(&self) -> &RawContext {
        &self.inner
    }

    pub fn set_option<O: Into<SetContextOption>>(&self, option: O, value: i32) -> ZmqResult<()> {
        self.inner.set(option.into().into(), value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_option_ext<O: Into<SetContextOption>, V: AsRef<str>>(
        &self,
        option: O,
        value: V,
    ) -> ZmqResult<()> {
        let zmq_option = option.into();
        self.inner.set_ext(zmq_option.into(), value.as_ref())
    }

    pub fn get_option<O: Into<GetContextOption>>(&self, option: O) -> ZmqResult<i32> {
        self.inner.get(option.into().into())
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn get_option_ext<O: Into<GetContextOption>>(&self, option: O) -> ZmqResult<String> {
        let zmq_option = option.into();
        self.inner.get_ext(zmq_option.into())
    }

    pub fn set_blocky(&self, value: bool) -> ZmqResult<()> {
        let option_value = match value {
            false => 0,
            _ => 1,
        };
        self.set_option(SetContextOption::Blocky, option_value)
    }

    pub fn get_blocky(&self) -> ZmqResult<bool> {
        let option_value = self.get_option(GetContextOption::Blocky)?;
        Ok(option_value != 0)
    }

    pub fn set_io_threads(&self, value: i32) -> ZmqResult<()> {
        self.set_option(SetContextOption::IoThreads, value)
    }

    pub fn get_io_threads(&self) -> ZmqResult<i32> {
        self.get_option(GetContextOption::IoThreads)
    }

    pub fn set_max_message_size(&self, value: i32) -> ZmqResult<()> {
        self.set_option(SetContextOption::MaxMessageSize, value)
    }

    pub fn get_max_message_size(&self) -> ZmqResult<i32> {
        self.get_option(GetContextOption::MaxMessageSize)
    }

    pub fn set_max_sockets(&self, value: i32) -> ZmqResult<()> {
        self.set_option(SetContextOption::MaxSockets, value)
    }

    pub fn get_max_sockets(&self) -> ZmqResult<i32> {
        self.get_option(GetContextOption::MaxSockets)
    }

    pub fn get_socket_limit(&self) -> ZmqResult<i32> {
        self.get_option(GetContextOption::SocketLimit)
    }

    pub fn set_ipv6(&self, value: bool) -> ZmqResult<()> {
        let option_value = match value {
            false => 0,
            _ => 1,
        };
        self.set_option(SetContextOption::IPv6, option_value)
    }

    pub fn get_ipv6(&self) -> ZmqResult<bool> {
        let option_value = self.get_option(GetContextOption::IPv6)?;
        Ok(option_value != 0)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn set_zero_copy_receiving(&self, value: bool) -> ZmqResult<()> {
        let option_value = match value {
            false => 0,
            _ => 1,
        };
        self.set_option(SetContextOption::ZeroCopyReceiving, option_value)
    }

    #[cfg(feature = "draft-api")]
    #[doc(cfg(feature = "draft-api"))]
    pub fn zero_copy_receiving(&self) -> ZmqResult<bool> {
        let option_value = self.get_option(GetContextOption::ZeroCopyReceiving)?;
        Ok(option_value != 0)
    }

    pub fn shutdown(&self) -> ZmqResult<()> {
        self.inner.shutdown()
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(feature = "builder")]
mod builder {
    use derive_builder::Builder;
    use serde::{Deserialize, Serialize};

    use crate::{ZmqResult, context::Context};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Builder)]
    #[builder(derive(PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize))]
    pub struct ContextConfig {
        #[builder(default = true)]
        blocky: bool,
        #[builder(setter(into), default = 1)]
        io_threads: i32,
        #[builder(setter(into), default = "i32::MAX")]
        max_message_size: i32,
        #[cfg(feature = "draft-api")]
        #[doc(cfg(feature = "draft-api"))]
        #[builder(default = true)]
        zero_copy_receiving: bool,
        #[builder(setter(into), default = 1023)]
        max_sockets: i32,
        #[builder(default = false)]
        ipv6: bool,
    }

    impl ContextConfig {
        pub fn apply(&self, context: &Context) -> ZmqResult<()> {
            context.set_blocky(self.blocky)?;
            context.set_io_threads(self.io_threads)?;
            context.set_max_message_size(self.max_message_size)?;
            #[cfg(feature = "draft-api")]
            context.set_zero_copy_receiving(self.zero_copy_receiving)?;
            context.set_max_sockets(self.max_sockets)?;
            context.set_ipv6(self.ipv6)?;

            Ok(())
        }
    }
}
