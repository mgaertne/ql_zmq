use std::thread;

use azmq::{
    ZmqResult,
    context::ZmqContext,
    socket::{Reply, Request, ZmqReceiver, ZmqRecvFlags, ZmqSendFlags, ZmqSender, ZmqSocket},
};

fn main() -> ZmqResult<()> {
    let port = 5556;

    let context = ZmqContext::new()?;

    let reply = ZmqSocket::<Reply>::from_context(&context)?;
    reply.bind(format!("tcp://*:{port}"))?;

    thread::spawn(move || {
        for _ in 1..=10 {
            let message = reply.recv_msg(ZmqRecvFlags::empty()).unwrap();
            println!("Received request: {message}");
            reply.send_msg("World", ZmqSendFlags::empty()).unwrap();
        }
    });

    let request = ZmqSocket::<Request>::from_context(&context)?;
    request.connect(format!("tcp://localhost:{port}"))?;

    for request_no in 1..=10 {
        println!("Sending request {request_no}");
        request.send_msg("Hello", ZmqSendFlags::empty())?;

        let message = request.recv_msg(ZmqRecvFlags::empty())?;
        println!("Received reply {request_no:2} [{message}]");
    }

    request.disconnect(format!("tcp://localhost:{port}"))?;

    Ok(())
}
