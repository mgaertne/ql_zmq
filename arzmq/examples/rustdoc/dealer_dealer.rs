use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{DealerSocket, Receiver, RecvFlags, SendFlags, Sender},
};

fn run_router_server(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
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

fn run_dealer_client(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let dealer = DealerSocket::from_context(context)?;
    dealer.connect(endpoint)?;

    for request_no in 1..=iterations {
        println!("Sending request {request_no}");
        let multipart = vec![vec![].into(), "Hello".into()];
        dealer.send_multipart(multipart.into(), SendFlags::empty())?;

        let mut message = dealer.recv_multipart(RecvFlags::empty())?;
        let content = message.pop_back().unwrap();
        if !content.is_empty() {
            println!("Received reply {request_no:2} {content}");
        }
    }

    Ok(())
}

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let router_endpoint = format!("tcp://*:{port}");
    run_router_server(&context, &router_endpoint, iterations)?;

    let dealer_endpoint = format!("tcp://localhost:{port}");
    run_dealer_client(&context, &dealer_endpoint, 10)?;

    Ok(())
}
