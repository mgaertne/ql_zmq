#![feature(cold_path)]
extern crate alloc;

pub mod context;
pub mod error;
mod ffi;
pub mod message;
pub mod socket;

pub use error::{ZmqError, ZmqResult};
pub(crate) use zmq_sys as zmq_sys_crate;

mod sealed {
    pub trait ZmqReceiverFlag {}
    pub trait ZmqSenderFlag {}
    pub trait ZmqSocketType {
        fn raw_socket_type() -> u32;
    }
}
