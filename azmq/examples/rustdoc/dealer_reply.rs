use std::thread;

use azmq::{
    ZmqResult,
    context::ZmqContext,
    socket::{Dealer, Reply, ZmqReceiver, ZmqRecvFlags, ZmqSendFlags, ZmqSender, ZmqSocket},
};

fn main() -> ZmqResult<()> {
    let port = 5556;

    let context = ZmqContext::new()?;

    let reply = ZmqSocket::<Reply>::from_context(&context)?;
    reply.bind(format!("tcp://*:{port}"))?;

    thread::spawn(move || {
        for _ in 1..=10 {
            let mut multipart = reply.recv_multipart(ZmqRecvFlags::empty()).unwrap();
            let content = multipart.pop_back().unwrap();
            if !content.is_empty() {
                println!("Received request: {content}");
            }
            multipart.push_back("world".into());
            reply
                .send_multipart(multipart, ZmqSendFlags::empty())
                .unwrap();
        }
    });

    let request = ZmqSocket::<Dealer>::from_context(&context)?;
    request.connect(format!("tcp://localhost:{port}"))?;

    for request_no in 1..=10 {
        println!("Sending request {request_no}");
        let multipart = vec![vec![].into(), "Hello".into()];
        request.send_multipart(multipart.into(), ZmqSendFlags::empty())?;

        let mut message = request.recv_multipart(ZmqRecvFlags::empty())?;
        let content = message.pop_back().unwrap();
        if !content.is_empty() {
            println!("Received reply {request_no:2} {content}");
        }
    }

    request.disconnect(format!("tcp://localhost:{port}"))?;

    Ok(())
}
