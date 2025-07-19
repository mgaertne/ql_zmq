use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{MultipartReceiver, MultipartSender, RecvFlags, RouterSocket, SendFlags},
};

mod common;

fn run_router_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let router = RouterSocket::from_context(context)?;
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

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let router_endpoint = format!("tcp://*:{port}");
    run_router_socket(&context, &router_endpoint, iterations)?;

    let dealer_endpoint = format!("tcp://localhost:{port}");
    common::run_dealer_client(&context, &dealer_endpoint, 10)?;

    Ok(())
}
