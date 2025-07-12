use std::thread;

use azmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, RecvFlags, Request, Router, SendFlags, Sender, Socket},
};

fn run_router_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let router = Socket::<Router>::from_context(context)?;
    router.bind(endpoint)?;

    thread::spawn(move || {
        for _ in 1..=iterations {
            let mut multipart = router.recv_multipart(RecvFlags::empty()).unwrap();
            let content = multipart.pop_back().unwrap();
            if !content.is_empty() {
                println!("Received request: {content}");
            }
            multipart.push_back("World".into());
            router
                .send_multipart(multipart, SendFlags::empty())
                .unwrap();
        }
    });

    Ok(())
}

fn run_request_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let request = Socket::<Request>::from_context(context)?;
    request.connect(endpoint)?;
    request.set_routing_id("request-router")?;

    for request_no in 1..=iterations {
        println!("Sending request {request_no}");
        request.send_msg("Hello".into(), SendFlags::empty())?;

        let message = request.recv_msg(RecvFlags::empty())?;
        println!("Received reply {request_no:2} {message}");
    }

    Ok(())
}

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let router_endpoint = format!("tcp://*:{port}");
    run_router_socket(&context, &router_endpoint, iterations)?;

    let request_endpoint = format!("tcp://localhost:{port}");
    run_request_socket(&context, &request_endpoint, iterations)?;

    Ok(())
}
