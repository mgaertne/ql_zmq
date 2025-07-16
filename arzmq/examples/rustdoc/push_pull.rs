use core::{
    sync::atomic::{AtomicBool, Ordering},
};
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{PullSocket, PushSocket, Receiver, RecvFlags, SendFlags, Sender},
};

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn run_push_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let push = PushSocket::from_context(context)?;
    push.bind(endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            push.send_msg("Important update".into(), SendFlags::empty())
                .unwrap();
        }
    });

    Ok(())
}

fn run_pull_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let pull = PullSocket::from_context(context)?;
    pull.connect(endpoint)?;

    for i in 1..=iterations {
        let msg = pull.recv_msg(RecvFlags::empty())?;
        println!("Received message {i:2}: {msg}");
    }

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let push_endpoint = format!("tcp://*:{port}");
    run_push_socket(&context, &push_endpoint)?;

    let pull_endpoint = format!("tcp://localhost:{port}");
    run_pull_socket(&context, &pull_endpoint, iterations)?;

    Ok(())
}
