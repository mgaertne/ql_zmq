use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{RequestSocket, RouterSocket},
};

mod common;

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let router = RouterSocket::from_context(&context)?;

    let router_endpoint = format!("tcp://*:{port}");
    router.bind(&router_endpoint)?;

    thread::spawn(move || {
        (1..=iterations)
            .try_for_each(|_| common::run_multipart_recv_reply(&router, "World"))
            .unwrap();
    });

    let request = RequestSocket::from_context(&context)?;

    let request_endpoint = format!("tcp://localhost:{port}");
    request.connect(&request_endpoint)?;

    (1..=iterations).try_for_each(|_| common::run_send_recv(&request, "Hello"))?;

    Ok(())
}
