use core::time::Duration;
use std::thread;

use azmq::{
    ZmqResult,
    context::ZmqContext,
    socket::{Publish, Subscribe, ZmqReceiver, ZmqRecvFlags, ZmqSendFlags, ZmqSender, ZmqSocket},
};

fn main() -> ZmqResult<()> {
    let port = 5556;
    let subscribed_topic = "azmq-example";

    let context = ZmqContext::new()?;

    let publish = ZmqSocket::<Publish>::from_context(&context)?;
    publish.bind(format!("tcp://*:{port}"))?;

    let subscribe = ZmqSocket::<Subscribe>::from_context(&context)?;
    subscribe.subscribe(subscribed_topic)?;
    subscribe.connect(format!("tcp://localhost:{port}"))?;

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        let published_msg = format!("{subscribed_topic} important update");
        publish
            .send_msg(&published_msg, ZmqSendFlags::empty())
            .unwrap();
    });

    let zmq_msg = subscribe.recv_msg(ZmqRecvFlags::empty())?;
    let zmq_str = zmq_msg.to_string();
    let pubsub_item = zmq_str.split_once(" ");
    assert_eq!(Some((subscribed_topic, "important update")), pubsub_item);

    let (topic, item) = pubsub_item.unwrap();
    println!("Received msg for topic {topic:?}: {item:?}",);

    subscribe.disconnect(format!("tcp://localhost:{port}"))?;

    Ok(())
}
