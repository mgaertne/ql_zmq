#![feature(cold_path)]
extern crate alloc;

pub mod context;
pub mod error;
mod ffi;
pub mod message;
pub mod socket;

use alloc::ffi::CString;

pub use error::{ZmqError, ZmqResult};
pub(crate) use zmq_sys as zmq_sys_crate;

mod sealed {
    use crate::socket;

    pub trait ZmqReceiverFlag {}
    pub trait ZmqSenderFlag {}
    pub trait ZmqSocketType {
        fn raw_socket_type() -> socket::ZmqSocketType;
    }
}

/// Return true if the used 0MQ library has the given capability.
///
/// For a list of capabilities, please consult the `zmq_has` manual
/// page.
///
pub fn has_capability(capability: &str) -> bool {
    let c_str = CString::new(capability.to_lowercase()).unwrap();
    unsafe { zmq_sys_crate::zmq_has(c_str.as_ptr()) != 0 }
}

/// Return the current zeromq version, as `(major, minor, patch)`.
pub fn version() -> (i32, i32, i32) {
    let mut major = Default::default();
    let mut minor = Default::default();
    let mut patch = Default::default();

    unsafe { zmq_sys_crate::zmq_version(&mut major, &mut minor, &mut patch) };

    (major, minor, patch)
}

pub use z85::{DecodeError as Z85DecodeError, decode as z85_decode, encode as z85_encode};
