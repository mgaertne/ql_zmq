use std::{io::prelude::*, net::TcpStream, thread};

use azmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, RecvFlags, SendFlags, Sender, Socket, Stream},
};

fn main() -> ZmqResult<()> {
    let port = 5556;

    let context = Context::new()?;

    let zmq_stream = Socket::<Stream>::from_context(&context)?;
    zmq_stream.bind(format!("tcp://*:{port}"))?;

    thread::spawn(move || {
        let mut connect_msg = zmq_stream.recv_multipart(RecvFlags::empty()).unwrap();
        let _routing_id = connect_msg.pop_front().unwrap();

        loop {
            let mut message = zmq_stream.recv_multipart(RecvFlags::empty()).unwrap();
            println!("Received request: {}", message.pop_back().unwrap());

            message.push_back("World".into());
            zmq_stream
                .send_multipart(message, SendFlags::empty())
                .unwrap();
        }
    });

    let mut tcp_stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    for request_no in 1..=10 {
        println!("Sending requrst {request_no}");
        tcp_stream.write_all("Hello".as_bytes()).unwrap();

        let mut buffer = [0; 256];
        if let Ok(length) = tcp_stream.read(&mut buffer) {
            if length == 0 {
                continue;
            }
            let recevied_msg = &buffer[..length];
            println!(
                "Received reply {request_no:2} {}",
                str::from_utf8(recevied_msg).unwrap()
            );
        }
    }

    Ok(())
}
