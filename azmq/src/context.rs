use alloc::sync::Arc;

use derive_more::{Debug, Display};

use crate::{ZmqResult, ffi::RawContext, zmq_sys_crate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ZmqSetContextOption {
    Blocky = zmq_sys_crate::ZMQ_BLOCKY as i32,
    IoThreads = zmq_sys_crate::ZMQ_IO_THREADS as i32,
    ThreadSchedulingPolicy = zmq_sys_crate::ZMQ_THREAD_SCHED_POLICY as i32,
    ThreadPriority = zmq_sys_crate::ZMQ_THREAD_PRIORITY as i32,
    ThreadAffinityCPUAdd = zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_ADD as i32,
    ThreadAffinityCPURemove = zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_REMOVE as i32,
    ThreadNamePrefix = zmq_sys_crate::ZMQ_THREAD_NAME_PREFIX as i32,
    MaxMessageSize = zmq_sys_crate::ZMQ_MAX_MSGSZ as i32,
    ZeroCopyReceiving = 10i32,
    MaxSockets = zmq_sys_crate::ZMQ_MAX_SOCKETS as i32,
    IPv6 = zmq_sys_crate::ZMQ_IPV6 as i32,
}

impl From<ZmqSetContextOption> for i32 {
    fn from(value: ZmqSetContextOption) -> Self {
        match value {
            ZmqSetContextOption::Blocky => zmq_sys_crate::ZMQ_BLOCKY as i32,
            ZmqSetContextOption::IoThreads => zmq_sys_crate::ZMQ_IO_THREADS as i32,
            ZmqSetContextOption::ThreadSchedulingPolicy => {
                zmq_sys_crate::ZMQ_THREAD_SCHED_POLICY as i32
            }
            ZmqSetContextOption::ThreadPriority => zmq_sys_crate::ZMQ_THREAD_PRIORITY as i32,
            ZmqSetContextOption::ThreadAffinityCPUAdd => {
                zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_ADD as i32
            }
            ZmqSetContextOption::ThreadAffinityCPURemove => {
                zmq_sys_crate::ZMQ_THREAD_AFFINITY_CPU_REMOVE as i32
            }
            ZmqSetContextOption::ThreadNamePrefix => zmq_sys_crate::ZMQ_THREAD_NAME_PREFIX as i32,
            ZmqSetContextOption::MaxMessageSize => zmq_sys_crate::ZMQ_MAX_MSGSZ as i32,
            ZmqSetContextOption::ZeroCopyReceiving => 10i32,
            ZmqSetContextOption::MaxSockets => zmq_sys_crate::ZMQ_MAX_SOCKETS as i32,
            ZmqSetContextOption::IPv6 => zmq_sys_crate::ZMQ_IPV6 as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ZmqGetContextOption {
    IoThreads = zmq_sys_crate::ZMQ_IO_THREADS as i32,
    MaxSockets = zmq_sys_crate::ZMQ_MAX_SOCKETS as i32,
    MaxMessageSize = zmq_sys_crate::ZMQ_MAX_MSGSZ as i32,
    ZeroCopyReceiving = 10i32,
    SocketLimit = zmq_sys_crate::ZMQ_SOCKET_LIMIT as i32,
    IPv6 = zmq_sys_crate::ZMQ_IPV6 as i32,
    Blocky = zmq_sys_crate::ZMQ_BLOCKY as i32,
    ThreadSchedulingPolicy = zmq_sys_crate::ZMQ_THREAD_SCHED_POLICY as i32,
    ThreadNamePrefix = zmq_sys_crate::ZMQ_THREAD_NAME_PREFIX as i32,
}

impl From<ZmqGetContextOption> for i32 {
    fn from(value: ZmqGetContextOption) -> Self {
        match value {
            ZmqGetContextOption::Blocky => zmq_sys_crate::ZMQ_BLOCKY as i32,
            ZmqGetContextOption::IoThreads => zmq_sys_crate::ZMQ_IO_THREADS as i32,
            ZmqGetContextOption::ThreadSchedulingPolicy => {
                zmq_sys_crate::ZMQ_THREAD_SCHED_POLICY as i32
            }
            ZmqGetContextOption::ThreadNamePrefix => zmq_sys_crate::ZMQ_THREAD_NAME_PREFIX as i32,
            ZmqGetContextOption::MaxMessageSize => zmq_sys_crate::ZMQ_MAX_MSGSZ as i32,
            ZmqGetContextOption::ZeroCopyReceiving => 10i32,
            ZmqGetContextOption::MaxSockets => zmq_sys_crate::ZMQ_MAX_SOCKETS as i32,
            ZmqGetContextOption::IPv6 => zmq_sys_crate::ZMQ_IPV6 as i32,
            ZmqGetContextOption::SocketLimit => zmq_sys_crate::ZMQ_SOCKET_LIMIT as i32,
        }
    }
}

#[derive(Debug, Display)]
#[debug("ZmqContext {{ ... }}")]
#[display("ZmqContext")]
pub struct ZmqContext {
    pub(crate) inner: Arc<RawContext>,
}

unsafe impl Send for ZmqContext {}
unsafe impl Sync for ZmqContext {}

impl ZmqContext {
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

    pub fn set_context_option<O: Into<ZmqSetContextOption>>(
        &self,
        option: O,
        value: i32,
    ) -> ZmqResult<()> {
        self.inner.set(option.into().into(), value)
    }

    pub fn get_context_option<O: Into<ZmqGetContextOption>>(&self, option: O) -> ZmqResult<i32> {
        self.inner.get(option.into().into())
    }

    pub fn set_blocky(&self, value: bool) -> ZmqResult<()> {
        let option_value = match value {
            false => 0,
            _ => 1,
        };
        self.set_context_option(ZmqSetContextOption::Blocky, option_value)
    }

    pub fn get_blocky(&self) -> ZmqResult<bool> {
        let option_value = self.get_context_option(ZmqGetContextOption::Blocky)?;
        Ok(option_value != 0)
    }

    pub fn set_io_threads(&self, value: i32) -> ZmqResult<()> {
        self.set_context_option(ZmqSetContextOption::IoThreads, value)
    }

    pub fn get_io_threads(&self) -> ZmqResult<i32> {
        self.get_context_option(ZmqGetContextOption::IoThreads)
    }

    pub fn set_max_message_size(&self, value: i32) -> ZmqResult<()> {
        self.set_context_option(ZmqSetContextOption::MaxMessageSize, value)
    }

    pub fn get_max_message_size(&self) -> ZmqResult<i32> {
        self.get_context_option(ZmqGetContextOption::MaxMessageSize)
    }

    pub fn set_max_sockets(&self, value: i32) -> ZmqResult<()> {
        self.set_context_option(ZmqSetContextOption::MaxSockets, value)
    }

    pub fn get_max_sockets(&self) -> ZmqResult<i32> {
        self.get_context_option(ZmqGetContextOption::MaxSockets)
    }

    pub fn get_socket_limit(&self) -> ZmqResult<i32> {
        self.get_context_option(ZmqGetContextOption::SocketLimit)
    }

    pub fn set_ipv6(&self, value: bool) -> ZmqResult<()> {
        let option_value = match value {
            false => 0,
            _ => 1,
        };
        self.set_context_option(ZmqSetContextOption::IPv6, option_value)
    }

    pub fn get_ipv6(&self) -> ZmqResult<bool> {
        let option_value = self.get_context_option(ZmqGetContextOption::IPv6)?;
        Ok(option_value != 0)
    }

    pub fn shutdown(&self) -> ZmqResult<()> {
        self.inner.shutdown()
    }
}

impl Clone for ZmqContext {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct ZmqContextConfig {
    blocky: Option<bool>,
    ipv6: Option<bool>,
    io_threads: Option<i32>,
    max_sockets: Option<i32>,
}

impl ZmqContextConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self) -> ZmqResult<ZmqContext> {
        let context = ZmqContext::new()?;
        self.apply(&context)?;

        Ok(context)
    }

    pub fn apply(&self, context: &ZmqContext) -> ZmqResult<()> {
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

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct ZmqContextBuilder {
    contig: ZmqContextConfig,
}

impl ZmqContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(&self) -> ZmqResult<ZmqContext> {
        self.contig.build()
    }

    pub fn apply(&self, handle: &ZmqContext) -> ZmqResult<()> {
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
