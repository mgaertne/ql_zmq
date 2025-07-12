use core::time::Duration;
use std::thread;

use azmq::{
    ZmqResult,
    context::Context,
    socket::{Publish, Receiver, RecvFlags, SendFlags, Sender, Socket, Subscribe},
};

fn main() -> ZmqResult<()> {
    let port = 5556;
    let subscribed_topic = "azmq-example";

    let context = Context::new()?;

    let publish = Socket::<Publish>::from_context(&context)?;
    publish.bind(format!("tcp://*:{port}"))?;

    let subscribe = Socket::<Subscribe>::from_context(&context)?;
    subscribe.connect(format!("tcp://localhost:{port}"))?;

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        let published_msg = format!("{subscribed_topic} important update");
        publish
            .send_msg(published_msg.as_str().into(), SendFlags::empty())
            .unwrap();
    });

    subscribe.subscribe(subscribed_topic)?;

    let zmq_msg = subscribe.recv_msg(RecvFlags::empty())?;
    let zmq_str = zmq_msg.to_string();
    let pubsub_item = zmq_str.split_once(" ");
    assert_eq!(Some((subscribed_topic, "important update")), pubsub_item);

    let (topic, item) = pubsub_item.unwrap();
    println!("Received msg for topic {topic:?}: {item}",);

    subscribe.subscribe("topic2")?;

    subscribe.disconnect(format!("tcp://localhost:{port}"))?;

    Ok(())
}
