use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{DealerSocket, MultipartReceiver, MultipartSender, RecvFlags, SendFlags},
};

mod common;

fn run_dealer_server(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let dealer = DealerSocket::from_context(context)?;
    dealer.bind(endpoint)?;

    thread::spawn(move || {
        for _ in 1..=iterations {
            let mut multipart = dealer.recv_multipart(RecvFlags::empty()).unwrap();
            let content = multipart.pop_back().unwrap();
            if !content.is_empty() {
                println!("Received request: {content}");
            }
            multipart.push_back("World".into());
            dealer
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
    run_dealer_server(&context, &router_endpoint, iterations)?;

    let dealer_endpoint = format!("tcp://localhost:{port}");
    common::run_dealer_client(&context, &dealer_endpoint, 10)?;

    Ok(())
}
