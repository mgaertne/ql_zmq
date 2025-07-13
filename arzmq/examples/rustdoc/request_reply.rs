use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, RecvFlags, ReplySocket, RequestSocket, SendFlags, Sender},
};

fn run_reply_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let reply = ReplySocket::from_context(context)?;
    reply.bind(endpoint)?;

    thread::spawn(move || {
        for _ in 1..=iterations {
            let message = reply.recv_msg(RecvFlags::empty()).unwrap();
            println!("Received request: {message}");
            reply.send_msg("World".into(), SendFlags::empty()).unwrap();
        }
    });

    Ok(())
}

fn run_request_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let request = RequestSocket::from_context(context)?;
    request.connect(endpoint)?;

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

    let reply_endpoint = format!("tcp://*:{port}");
    run_reply_socket(&context, &reply_endpoint, iterations)?;

    let request_endpoint = format!("tcp://localhost:{port}");
    run_request_socket(&context, &request_endpoint, iterations)?;

    Ok(())
}
