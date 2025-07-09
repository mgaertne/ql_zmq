use std::thread;

use azmq::{
    ZmqResult,
    context::ZmqContext,
    socket::{Request, Router, ZmqReceiver, ZmqRecvFlags, ZmqSendFlags, ZmqSender, ZmqSocket},
};

fn main() -> ZmqResult<()> {
    let port = 5556;

    let context = ZmqContext::new()?;

    let router = ZmqSocket::<Router>::from_context(&context)?;
    router.bind(format!("tcp://*:{port}"))?;

    thread::spawn(move || {
        for _ in 1..=10 {
            let mut multipart = router.recv_multipart(ZmqRecvFlags::empty()).unwrap();
            let content = multipart.pop_back().unwrap();
            if !content.is_empty() {
                println!("Received request: {content}");
            }
            multipart.push_back("World".into());
            router
                .send_multipart(multipart, ZmqSendFlags::empty())
                .unwrap();
        }
    });

    let request = ZmqSocket::<Request>::from_context(&context)?;
    request.connect(format!("tcp://localhost:{port}"))?;
    request.set_routing_id("request-router")?;

    for request_no in 1..=10 {
        println!("Sending request {request_no}");
        request.send_msg("Hello".into(), ZmqSendFlags::empty())?;

        let message = request.recv_msg(ZmqRecvFlags::empty())?;
        println!("Received reply {request_no:2} {message}");
    }

    request.disconnect(format!("tcp://localhost:{port}"))?;

    Ok(())
}
