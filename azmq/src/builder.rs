//! Builders for 0MQ contexts and sockets

use serde::{Deserialize, Serialize};

use crate::{ZmqResult, context::ZmqContext};

#[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
