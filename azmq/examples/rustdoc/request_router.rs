use std::thread;

use azmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, RecvFlags, Request, Router, SendFlags, Sender, Socket},
};

fn main() -> ZmqResult<()> {
    let port = 5556;

    let context = Context::new()?;

    let router = Socket::<Router>::from_context(&context)?;
    router.bind(format!("tcp://*:{port}"))?;

    thread::spawn(move || {
        for _ in 1..=10 {
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

    let request = Socket::<Request>::from_context(&context)?;
    request.connect(format!("tcp://localhost:{port}"))?;
    request.set_routing_id("request-router")?;

    for request_no in 1..=10 {
        println!("Sending request {request_no}");
        request.send_msg("Hello".into(), SendFlags::empty())?;

        let message = request.recv_msg(RecvFlags::empty())?;
        println!("Received reply {request_no:2} {message}");
    }

    request.disconnect(format!("tcp://localhost:{port}"))?;

    Ok(())
}
