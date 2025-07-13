use std::thread;

use azmq::{
    ZmqResult,
    context::Context,
    socket::{PairSocket, Receiver, RecvFlags, SendFlags, Sender},
};

fn run_pair_server(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let pair = PairSocket::from_context(context)?;
    pair.bind(endpoint)?;

    thread::spawn(move || {
        for _ in 1..=iterations {
            let message = pair.recv_msg(RecvFlags::empty()).unwrap();
            println!("Received request: {message}");
            pair.send_msg("World".into(), SendFlags::empty()).unwrap();
        }
    });

    Ok(())
}

fn run_pair_client(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let pair = PairSocket::from_context(context)?;
    pair.connect(endpoint)?;

    for request_no in 1..=iterations {
        println!("Sending request {request_no}");
        pair.send_msg("Hello".into(), SendFlags::empty())?;

        let message = pair.recv_msg(RecvFlags::empty())?;
        println!("Received reply {request_no:2} {message}");
    }

    Ok(())
}

fn main() -> ZmqResult<()> {
    let endpoint = "inproc://azmq-example-pair";
    let iterations = 10;

    let context = Context::new()?;

    run_pair_server(&context, endpoint, iterations)?;

    run_pair_client(&context, endpoint, iterations)?;

    Ok(())
}
