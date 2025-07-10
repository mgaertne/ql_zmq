use std::thread;

use azmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, RecvFlags, Reply, Request, SendFlags, Sender, Socket},
};

fn main() -> ZmqResult<()> {
    let port = 5556;

    let context = Context::new()?;

    let reply = Socket::<Reply>::from_context(&context)?;
    reply.bind(format!("tcp://*:{port}"))?;

    thread::spawn(move || {
        for _ in 1..=10 {
            let message = reply.recv_msg(RecvFlags::empty()).unwrap();
            println!("Received request: {message}");
            reply.send_msg("World".into(), SendFlags::empty()).unwrap();
        }
    });

    let request = Socket::<Request>::from_context(&context)?;
    request.connect(format!("tcp://localhost:{port}"))?;

    for request_no in 1..=10 {
        println!("Sending request {request_no}");
        request.send_msg("Hello".into(), SendFlags::empty())?;

        let message = request.recv_msg(RecvFlags::empty())?;
        println!("Received reply {request_no:2} {message}");
    }

    request.disconnect(format!("tcp://localhost:{port}"))?;

    Ok(())
}
