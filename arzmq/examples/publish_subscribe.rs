use core::sync::atomic::Ordering;
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{PublishSocket, SendFlags, Sender, SubscribeSocket},
};

mod common;

use common::KEEP_RUNNING;

const SUBSCRIBED_TOPIC: &str = "arzmq-example";

fn run_publish_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let publish = PublishSocket::from_context(context)?;
    publish.bind(endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            let published_msg = format!("{SUBSCRIBED_TOPIC} important update");
            publish
                .send_msg(&published_msg, SendFlags::empty())
                .unwrap();
        }
    });

    Ok(())
}

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let publish_endpoint = format!("tcp://*:{port}");
    run_publish_socket(&context, &publish_endpoint)?;

    let subscribe = SubscribeSocket::from_context(&context)?;
    let subscribe_endpoint = format!("tcp://localhost:{port}");
    subscribe.connect(subscribe_endpoint)?;

    subscribe.subscribe(SUBSCRIBED_TOPIC)?;

    common::run_subscribe_client(&subscribe, SUBSCRIBED_TOPIC, iterations)?;

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}
