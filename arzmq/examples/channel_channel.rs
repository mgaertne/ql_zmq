use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{ChannelSocket, Receiver, RecvFlags, SendFlags, Sender},
};

fn run_channel_server(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let channel = ChannelSocket::from_context(context)?;
    channel.bind(endpoint)?;

    thread::spawn(move || {
        for _ in 1..=iterations {
            let message = channel.recv_msg(RecvFlags::empty()).unwrap();
            println!("Received request: {message}");
            channel.send_msg("World", SendFlags::empty()).unwrap();
        }
    });

    Ok(())
}

fn run_channel_client(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let channel = ChannelSocket::from_context(context)?;
    channel.connect(endpoint)?;

    for request_no in 1..=iterations {
        println!("Sending request {request_no}");
        channel.send_msg("Hello", SendFlags::empty())?;

        let message = channel.recv_msg(RecvFlags::empty())?;
        println!("Received reply {request_no:2} {message}");
    }

    Ok(())
}

fn main() -> ZmqResult<()> {
    let endpoint = "inproc://arzmq-example-pair";
    let iterations = 10;

    let context = Context::new()?;

    run_channel_server(&context, endpoint, iterations)?;

    run_channel_client(&context, endpoint, iterations)?;

    Ok(())
}
