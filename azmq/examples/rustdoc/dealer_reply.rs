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
            let message = reply.recv_multipart(ZmqRecvFlags::empty()).unwrap();
            let content = message.iter().last().unwrap();
            if !content.is_empty() {
                println!("Received request: {:?}", str::from_utf8(content).unwrap());
            }
            let up_to_last_idx = message.len() - 1;
            let mut multipart: Vec<Vec<u8>> = message.into_iter().take(up_to_last_idx).collect();
            multipart.push("World".as_bytes().to_vec());
            reply
                .send_multipart(multipart, ZmqSendFlags::empty())
                .unwrap();
        }
    });

    let request = ZmqSocket::<Dealer>::from_context(&context)?;
    request.connect(format!("tcp://localhost:{port}"))?;

    for request_no in 1..=10 {
        println!("Sending request {request_no}");
        let multipart = vec![vec![], "Hello".as_bytes().to_vec()];
        request.send_multipart(multipart, ZmqSendFlags::empty())?;

        let message = request.recv_multipart(ZmqRecvFlags::empty())?;
        let content = message.iter().last().unwrap();
        if !content.is_empty() {
            println!(
                "Received reply {request_no:2} [{}]",
                str::from_utf8(content).unwrap()
            );
        }
    }

    request.disconnect(format!("tcp://localhost:{port}"))?;

    Ok(())
}
