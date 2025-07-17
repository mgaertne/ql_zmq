use derive_more::Display;

use crate::{
    ZmqError, ZmqResult, sealed,
    socket::{Socket, SocketOption},
    zmq_sys_crate,
};

#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum SecurityMechanism {
    Null = zmq_sys_crate::ZMQ_NULL as i32,
    #[display("PlainClient(username = {username}, password = {password})")]
    PlainClient {
        username: String,
        password: String,
    },
    #[display("PlainServer(username = {username}, password = {password})")]
    PlainServer {
        username: String,
        password: String,
    },
    #[display("CurveClient(...)")]
    CurveClient {
        server_key: Vec<u8>,
        public_key: Vec<u8>,
        secret_key: Vec<u8>,
    },
    #[display("CurveServer(...)")]
    CurveServer {
        secret_key: Vec<u8>,
    },
    GssApiClient {
        service_principal: String,
    },
    GssApiServer,
}

impl SecurityMechanism {
    pub fn apply<T: sealed::SocketType>(&self, socket: &Socket<T>) -> ZmqResult<()> {
        match self {
            SecurityMechanism::Null => socket.set_sockopt_bool(SocketOption::PlainServer, false)?,
            SecurityMechanism::PlainServer { username, password } => {
                socket.set_sockopt_bool(SocketOption::PlainServer, true)?;
                socket.set_sockopt_string(SocketOption::PlainUsername, username)?;
                socket.set_sockopt_string(SocketOption::PlainPassword, password)?;
            }
            SecurityMechanism::PlainClient { username, password } => {
                socket.set_sockopt_bool(SocketOption::PlainServer, true)?;
                socket.set_sockopt_string(SocketOption::PlainUsername, username)?;
                socket.set_sockopt_string(SocketOption::PlainPassword, password)?;
            }
            SecurityMechanism::CurveServer { secret_key } => {
                socket.set_sockopt_bool(SocketOption::CurveServer, true)?;
                socket.set_sockopt_bytes(SocketOption::CurveSecretKey, secret_key)?;
            }
            SecurityMechanism::CurveClient {
                server_key,
                public_key,
                secret_key,
            } => {
                socket.set_sockopt_bytes(SocketOption::CurveServerKey, server_key)?;
                socket.set_sockopt_bytes(SocketOption::CurvePublicKey, public_key)?;
                socket.set_sockopt_bytes(SocketOption::CurveSecretKey, secret_key)?;
            }
            SecurityMechanism::GssApiClient { service_principal } => {
                socket
                    .set_sockopt_string(SocketOption::GssApiServicePrincipal, service_principal)?;
            }
            SecurityMechanism::GssApiServer => {
                socket.set_sockopt_bool(SocketOption::GssApiServer, true)?;
            }
        }
        Ok(())
    }
}

impl<T: sealed::SocketType> TryFrom<&Socket<T>> for SecurityMechanism {
    type Error = ZmqError;

    fn try_from(socket: &Socket<T>) -> Result<Self, Self::Error> {
        match socket.get_sockopt_int::<i32>(SocketOption::Mechanism)? {
            value if value == zmq_sys_crate::ZMQ_NULL as i32 => Ok(Self::Null),
            value if value == zmq_sys_crate::ZMQ_PLAIN as i32 => {
                let username = socket.get_sockopt_string(SocketOption::PlainUsername)?;
                let password = socket.get_sockopt_string(SocketOption::PlainPassword)?;
                if socket.get_sockopt_bool(SocketOption::PlainServer)? {
                    Ok(Self::PlainServer { username, password })
                } else {
                    Ok(Self::PlainClient { username, password })
                }
            }
            value if value == zmq_sys_crate::ZMQ_CURVE as i32 => {
                let secret_key = socket.get_sockopt_bytes(SocketOption::CurveSecretKey)?;
                if socket.get_sockopt_bool(SocketOption::CurveServer)? {
                    Ok(Self::CurveServer { secret_key })
                } else {
                    let server_key = socket.get_sockopt_bytes(SocketOption::CurveServerKey)?;
                    let public_key = socket.get_sockopt_bytes(SocketOption::CurvePublicKey)?;
                    Ok(Self::CurveClient {
                        server_key,
                        public_key,
                        secret_key,
                    })
                }
            }
            value if value == zmq_sys_crate::ZMQ_GSSAPI as i32 => {
                if socket.get_sockopt_bool(SocketOption::GssApiServer)? {
                    Ok(Self::GssApiServer)
                } else {
                    let service_principal =
                        socket.get_sockopt_string(SocketOption::GssApiServicePrincipal)?;
                    Ok(Self::GssApiClient { service_principal })
                }
            }
            _ => Err(ZmqError::Unsupported),
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
#[repr(i32)]
pub enum GssApiNametype {
    NtHostbased = zmq_sys_crate::ZMQ_GSSAPI_NT_HOSTBASED as i32,
    NtUsername = zmq_sys_crate::ZMQ_GSSAPI_NT_USER_NAME as i32,
    NtKrb6Principal = zmq_sys_crate::ZMQ_GSSAPI_NT_KRB5_PRINCIPAL as i32,
}

impl TryFrom<i32> for GssApiNametype {
    type Error = ZmqError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            _ if value == zmq_sys_crate::ZMQ_GSSAPI_NT_HOSTBASED as i32 => Ok(Self::NtHostbased),
            _ if value == zmq_sys_crate::ZMQ_GSSAPI_NT_USER_NAME as i32 => Ok(Self::NtUsername),
            _ if value == zmq_sys_crate::ZMQ_GSSAPI_NT_KRB5_PRINCIPAL as i32 => {
                Ok(Self::NtKrb6Principal)
            }
            _ => Err(ZmqError::Unsupported),
        }
    }
}
