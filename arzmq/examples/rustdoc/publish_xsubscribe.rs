use core::{
    sync::atomic::{AtomicBool, Ordering},
};
use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    socket::{PublishSocket, Receiver, RecvFlags, SendFlags, Sender, XSubscribeSocket},
};

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
const SUBSCRIBED_TOPIC: &str = "arzmq-example";

fn run_publish_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let publish = PublishSocket::from_context(context)?;
    publish.bind(endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            let published_msg = format!("{SUBSCRIBED_TOPIC} important update");
            publish
                .send_msg(published_msg.as_str().into(), SendFlags::empty())
                .unwrap();
        }
    });

    Ok(())
}

fn run_xsubscribe_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let xsubscribe = XSubscribeSocket::from_context(context)?;
    xsubscribe.connect(endpoint)?;

    xsubscribe.subscribe(SUBSCRIBED_TOPIC)?;

    for number in 0..iterations {
        let zmq_msg = xsubscribe.recv_msg(RecvFlags::empty())?;
        let zmq_str = zmq_msg.to_string();
        let pubsub_item = zmq_str.split_once(" ");
        assert_eq!(Some((SUBSCRIBED_TOPIC, "important update")), pubsub_item);

        let (topic, item) = pubsub_item.unwrap();
        println!("Received msg for topic {topic:?}: {item}",);

        xsubscribe.subscribe(format!("topic-{number}"))?;
    }

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}

fn main() -> ZmqResult<()> {
    let port = 5556;
    let iterations = 10;

    let context = Context::new()?;

    let publish_endpoint = format!("tcp://*:{port}");
    run_publish_socket(&context, &publish_endpoint)?;

    let xsubscribe_endpoint = format!("tcp://localhost:{port}");
    run_xsubscribe_socket(&context, &xsubscribe_endpoint, iterations)?;

    Ok(())
}
