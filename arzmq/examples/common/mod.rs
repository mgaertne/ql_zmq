use core::sync::atomic::{AtomicBool, Ordering};

use arzmq::{
    ZmqResult,
    context::Context,
    message::Message,
    socket::{
        DealerSocket, MultipartReceiver, MultipartSender, Receiver, RecvFlags, SendFlags,
        SubscribeSocket,
    },
};

#[allow(dead_code)]
pub static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

#[allow(dead_code)]
pub fn run_dealer_client(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let dealer = DealerSocket::from_context(context)?;
    dealer.connect(endpoint)?;

    for request_no in 1..=iterations {
        println!("Sending request {request_no}");
        let multipart: Vec<Message> = vec![vec![].into(), "Hello".into()];
        dealer.send_multipart(multipart, SendFlags::empty())?;

        let mut message = dealer.recv_multipart(RecvFlags::empty())?;
        let content = message.pop_back().unwrap();
        if !content.is_empty() {
            println!("Received reply {request_no:2} {content}");
        }
    }

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}

#[allow(dead_code)]
pub fn run_subscribe_client(
    socket: &SubscribeSocket,
    subscribed_topic: &str,
    iterations: i32,
) -> ZmqResult<()> {
    for number in 0..iterations {
        let zmq_msg = socket.recv_msg(RecvFlags::empty())?;
        let zmq_str = zmq_msg.to_string();
        let pubsub_item = zmq_str.split_once(" ");
        assert_eq!(Some((subscribed_topic, "important update")), pubsub_item);

        let (topic, item) = pubsub_item.unwrap();
        println!("Received msg for topic {topic:?}: {item}",);

        socket.subscribe(format!("topic-{number}"))?;
    }

    Ok(())
}
