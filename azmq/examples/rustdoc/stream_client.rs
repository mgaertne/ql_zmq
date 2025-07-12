use core::error::Error;
use std::{io::prelude::*, net::TcpListener, thread};

use azmq::{
    ZmqResult,
    context::Context,
    message::MultipartMessage,
    socket::{Receiver, RecvFlags, SendFlags, Sender, Socket, Stream},
};

fn run_tcp_server(endpoint: &str) -> Result<(), Box<dyn Error>> {
    let tcp_listener = TcpListener::bind(endpoint)?;
    thread::spawn(move || {
        let (mut tcp_stream, _socket_addr) = tcp_listener.accept().unwrap();
        tcp_stream.write_all("".as_bytes()).unwrap();
        loop {
            let mut buffer = [0; 256];
            if let Ok(length) = tcp_stream.read(&mut buffer) {
                if length == 0 {
                    break;
                }
                let recevied_msg = &buffer[..length];
                println!(
                    "Received request: {}",
                    str::from_utf8(recevied_msg).unwrap()
                );
                tcp_stream.write_all("World".as_bytes()).unwrap();
            }
        }
    });

    Ok(())
}

fn run_stream_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let zmq_stream = Socket::<Stream>::from_context(context)?;
    zmq_stream.connect(endpoint)?;
    let mut connect_msg = zmq_stream.recv_multipart(RecvFlags::empty())?;
    let routing_id = connect_msg.pop_front().unwrap();

    for request_no in 1..=iterations {
        let mut multipart = MultipartMessage::new();
        multipart.push_back(routing_id.clone());
        multipart.push_back("Hello".into());
        println!("Sending request {request_no}");
        zmq_stream.send_multipart(multipart, SendFlags::empty())?;

        let mut message = zmq_stream.recv_multipart(RecvFlags::empty())?;
        println!(
            "Received reply {request_no:2} {}",
            message.pop_back().unwrap()
        );
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let port = 5556;
    let iterations = 10;

    let tcp_endpoint = format!("127.0.0.1:{port}");
    run_tcp_server(&tcp_endpoint)?;

    let context = Context::new()?;

    let stream_endpoint = format!("tcp://127.0.0.1:{port}");
    run_stream_socket(&context, &stream_endpoint, iterations)?;

    Ok(())
}
