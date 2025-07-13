use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::thread;

use azmq::{
    ZmqResult,
    context::Context,
    socket::{Pull, Push, Receiver, RecvFlags, SendFlags, Sender, Socket},
};

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn run_push_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let push = Socket::<Push>::from_context(context)?;
    push.bind(endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(100));

            push.send_msg("Important update".into(), SendFlags::empty())
                .unwrap();
        }
    });

    Ok(())
}

fn run_pull_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let pull = Socket::<Pull>::from_context(context)?;
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
