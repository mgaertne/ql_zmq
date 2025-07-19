use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{MultipartReceiver, MultipartSender, RecvFlags, ReplySocket, SendFlags},
};

mod common;

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

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let reply_endpoint = format!("tcp://*:{port}");
    run_reply_socket(&context, &reply_endpoint, iterations)?;

    let dealer_endpoint = format!("tcp://localhost:{port}");
    common::run_dealer_client(&context, &dealer_endpoint, 10)?;

    Ok(())
}
