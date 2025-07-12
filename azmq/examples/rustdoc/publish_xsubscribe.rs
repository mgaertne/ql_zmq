use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::thread;

use azmq::{
    ZmqResult,
    context::Context,
    socket::{Publish, Receiver, RecvFlags, SendFlags, Sender, Socket, XSubscribe},
};

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
const SUBSCRIBED_TOPIC: &str = "azmq-example";

fn run_publish_socket(context: &Context, endpoint: &str) -> ZmqResult<()> {
    let publish = Socket::<Publish>::from_context(context)?;
    publish.bind(endpoint)?;

    thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Acquire) {
            thread::sleep(Duration::from_millis(100));

            let published_msg = format!("{SUBSCRIBED_TOPIC} important update");
            publish
                .send_msg(published_msg.as_str().into(), SendFlags::empty())
                .unwrap();
        }
    });

    Ok(())
}

fn run_xsubscribe_socket(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let xsubscribe = Socket::<XSubscribe>::from_context(context)?;
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
