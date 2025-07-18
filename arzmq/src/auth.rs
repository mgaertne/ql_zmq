use crate::{
    ZmqResult, sealed,
    socket::{Socket, SocketOption},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "builder", derive(serde::Serialize, serde::Deserialize))]
pub struct ZapDomain {
    domain: String,
}

impl ZapDomain {
    pub fn new(domain: String) -> Self {
        Self { domain }
    }

    pub fn apply<T: sealed::SocketType>(&self, socket: &Socket<T>) -> ZmqResult<()> {
        socket.set_sockopt_string(SocketOption::ZapDomain, &self.domain)?;

        #[cfg(feature = "draft-api")]
        socket.set_sockopt_bool(SocketOption::ZapEnforceDomain, true)?;

        Ok(())
    }
}

impl<T: Into<String>> From<T> for ZapDomain {
    fn from(value: T) -> Self {
        ZapDomain::new(value.into())
    }
}
