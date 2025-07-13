use core::error::Error;
use std::{io::prelude::*, net::TcpStream, thread};

use azmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, RecvFlags, SendFlags, Sender, StreamSocket},
};

fn run_stream_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let zmq_stream = StreamSocket::from_context(context)?;
    zmq_stream.bind(endpoint)?;

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

    Ok(())
}

fn run_tcp_client(endpoint: &str, iterations: i32) -> Result<(), Box<dyn Error>> {
    let mut tcp_stream = TcpStream::connect(endpoint)?;
    for request_no in 1..=iterations {
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

fn main() -> Result<(), Box<dyn Error>> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let stream_endpoint = format!("tcp://*:{port}");
    run_stream_socket(&context, &stream_endpoint)?;

    let tcp_endpoint = format!("127.0.0.1:{port}");
    run_tcp_client(&tcp_endpoint, iterations)?;

    Ok(())
}
