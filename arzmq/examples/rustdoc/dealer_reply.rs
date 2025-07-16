use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{DealerSocket, MultipartReceiver, MultipartSender, RecvFlags, ReplySocket, SendFlags},
};

fn run_reply_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let reply = ReplySocket::from_context(context)?;
    reply.bind(endpoint)?;

    thread::spawn(move || {
        for _ in 1..=iterations {
            let mut multipart = reply.recv_multipart(RecvFlags::empty()).unwrap();
            let content = multipart.pop_back().unwrap();
            if !content.is_empty() {
                println!("Received request: {content}");
            }
            multipart.push_back("world".into());
            reply.send_multipart(multipart, SendFlags::empty()).unwrap();
        }
    });

    Ok(())
}

fn run_dealer_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
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

    let reply_endpoint = format!("tcp://*:{port}");
    run_reply_socket(&context, &reply_endpoint, iterations)?;

    let dealer_endpoint = format!("tcp://localhost:{port}");
    run_dealer_socket(&context, &dealer_endpoint, 10)?;

    Ok(())
}
