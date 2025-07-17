use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOptions},
};

pub struct ZapDomain {
    domain: String,
}

impl ZapDomain {
    pub fn new(domain: String) -> Self {
        Self { domain }
    }

    pub fn apply<T: sealed::SocketType>(&self, socket: &Socket<T>) -> ZmqResult<()> {
        socket.set_sockopt_string(SocketOptions::ZapDomain, &self.domain)?;

        #[cfg(feature = "draft-api")]
        socket.set_sockopt_bool(SocketOptions::ZapEnforceDomain, true)?;

        Ok(())
    }
}

impl<T: Into<String>> From<T> for ZapDomain {
    fn from(value: T) -> Self {
        ZapDomain::new(value.into())
    }
}
