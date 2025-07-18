use alloc::sync::Arc;

#[cfg(feature = "builder")]
#[doc(cfg(feature = "builder"))]
pub use builder::{ContextBuilder, ContextConfig};
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
    use serde::{Deserialize, Serialize};

    use crate::{ZmqResult, context::Context};

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ContextConfig {
        blocky: Option<bool>,
        ipv6: Option<bool>,
        io_threads: Option<i32>,
        max_sockets: Option<i32>,
    }

    impl ContextConfig {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn build(&self) -> ZmqResult<Context> {
            let context = Context::new()?;
            self.apply(&context)?;

            Ok(context)
        }

        pub fn apply(&self, context: &Context) -> ZmqResult<()> {
            self.ipv6
                .iter()
                .try_for_each(|&ipv6| context.set_ipv6(ipv6))?;
            self.blocky
                .iter()
                .try_for_each(|&blocky| context.set_blocky(blocky))?;
            self.io_threads
                .iter()
                .try_for_each(|&io_threads| context.set_io_threads(io_threads))?;
            self.max_sockets
                .iter()
                .try_for_each(|&max_sockets| context.set_max_sockets(max_sockets))?;

            Ok(())
        }

        pub fn blocky(&self) -> Option<bool> {
            self.blocky
        }

        pub fn set_blocky(&mut self, value: Option<bool>) {
            self.blocky = value;
        }

        pub fn ipv6(&self) -> Option<bool> {
            self.ipv6
        }

        pub fn set_ipv6(&mut self, value: Option<bool>) {
            self.ipv6 = value;
        }

        pub fn io_threads(&self) -> Option<i32> {
            self.io_threads
        }

        pub fn set_io_threads(&mut self, value: Option<i32>) {
            self.io_threads = value;
        }

        pub fn max_sockets(&mut self) -> Option<i32> {
            self.max_sockets
        }

        pub fn set_max_sockets(&mut self, value: Option<i32>) {
            self.max_sockets = value;
        }
    }

    #[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ContextBuilder {
        contig: ContextConfig,
    }

    impl ContextBuilder {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn build(&self) -> ZmqResult<Context> {
            self.contig.build()
        }

        pub fn apply(&self, handle: &Context) -> ZmqResult<()> {
            self.contig.apply(handle)
        }

        pub fn blocky(&mut self, value: bool) -> &mut Self {
            self.contig.set_blocky(Some(value));
            self
        }

        pub fn ipv6(&mut self, value: bool) -> &mut Self {
            self.contig.set_ipv6(Some(value));
            self
        }

        pub fn io_threads(&mut self, value: i32) -> &mut Self {
            self.contig.set_io_threads(Some(value));
            self
        }

        pub fn max_sockets(&mut self, value: i32) -> &mut Self {
            self.contig.set_max_sockets(Some(value));
            self
        }
    }
}
