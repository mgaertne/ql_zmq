use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    message::Message,
    socket::{ClientSocket, Receiver, RecvFlags, SendFlags, Sender, ServerSocket},
};

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn run_server_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let server = ServerSocket::from_context(context)?;
    server.bind(endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(100));

            let message = server.recv_msg(RecvFlags::empty()).unwrap();
            println!("Received message: \"{message}\"");

            let returned: Message = "World".into();
            returned
                .set_routing_id(message.routing_id().unwrap())
                .unwrap();
            server.send_msg(returned, SendFlags::empty()).unwrap();
        }
    });

    Ok(())
}

fn run_client_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let client = ClientSocket::from_context(context)?;
    client.connect(endpoint)?;

    for number in 0..iterations {
        client.send_msg("Hello".into(), SendFlags::empty())?;
        let zmq_msg = client.recv_msg(RecvFlags::empty())?;
        println!("Received msg {number}: {zmq_msg}",);
    }

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let publish_endpoint = format!("tcp://*:{port}");
    run_server_socket(&context, &publish_endpoint)?;

    let subscribe_endpoint = format!("tcp://localhost:{port}");
    run_client_socket(&context, &subscribe_endpoint, iterations)?;

    Ok(())
}
