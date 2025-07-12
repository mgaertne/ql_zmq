use core::time::Duration;
use std::thread;

use azmq::{
    ZmqResult,
    context::Context,
    socket::{Receiver, RecvFlags, SendFlags, Sender, Socket, XPublish, XSubscribe},
};

fn main() -> ZmqResult<()> {
    let port = 5556;
    let subscribed_topic = "azmq-example";

    let context = Context::new()?;

    let xpublish = Socket::<XPublish>::from_context(&context)?;
    xpublish.bind(format!("tcp://*:{port}"))?;

    let xsubscribe = Socket::<XSubscribe>::from_context(&context)?;
    xsubscribe.connect(format!("tcp://localhost:{port}"))?;

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        let subscription = xpublish.recv_msg(RecvFlags::empty()).unwrap();
        let (first_byte, subscription_topic) =
            (subscription[0], str::from_utf8(&subscription[1..]).unwrap());
        assert_eq!(first_byte, 1);
        assert_eq!(subscription_topic, subscribed_topic);
        println!("{first_byte} {subscription_topic}");

        let published_msg = format!("{subscribed_topic} important update");
        xpublish
            .send_msg(published_msg.as_str().into(), SendFlags::empty())
            .unwrap();
    });

    xsubscribe.subscribe(subscribed_topic)?;

    let zmq_msg = xsubscribe.recv_msg(RecvFlags::empty())?;
    let zmq_str = zmq_msg.to_string();
    let pubsub_item = zmq_str.split_once(" ");
    assert_eq!(Some((subscribed_topic, "important update")), pubsub_item);

    let (topic, item) = pubsub_item.unwrap();
    println!("Received msg for topic {topic:?}: {item}",);

    xsubscribe.subscribe("topic2")?;

    xsubscribe.disconnect(format!("tcp://localhost:{port}"))?;

    Ok(())
}
