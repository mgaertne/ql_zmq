use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{GatherSocket, Receiver, RecvFlags, ScatterSocket, SendFlags, Sender},
};

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn run_scatter_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let scatter = ScatterSocket::from_context(context)?;
    scatter.bind(endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(100));

            scatter
                .send_msg("Important update".into(), SendFlags::empty())
                .unwrap();
        }
    });

    Ok(())
}

fn run_gather_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let gather = GatherSocket::from_context(context)?;
    gather.connect(endpoint)?;

    for i in 1..=iterations {
        let msg = gather.recv_msg(RecvFlags::empty())?;
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
    run_scatter_socket(&context, &push_endpoint)?;

    let pull_endpoint = format!("tcp://localhost:{port}");
    run_gather_socket(&context, &pull_endpoint, iterations)?;

    Ok(())
}
