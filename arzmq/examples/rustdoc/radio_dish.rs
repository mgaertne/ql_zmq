use core::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    message::Message,
    socket::{DishSocket, RadioSocket, Receiver, RecvFlags, SendFlags, Sender},
};

static GROUP: &str = "radio-dish-ex";
static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn run_radio_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let radio = RadioSocket::from_context(context)?;
    radio.bind(endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            let message: Message = "radio msg".into();
            message.set_group(GROUP).unwrap();

            radio.send_msg(message, SendFlags::empty()).unwrap();
        }
    });

    Ok(())
}

fn run_dish_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let dish = DishSocket::from_context(context)?;
    dish.connect(endpoint)?;
    dish.join(GROUP)?;

    for i in 1..=iterations {
        let msg = dish.recv_msg(RecvFlags::empty())?;
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
    run_radio_socket(&context, &push_endpoint)?;

    let pull_endpoint = format!("tcp://localhost:{port}");
    run_dish_socket(&context, &pull_endpoint, iterations)?;

    Ok(())
}
