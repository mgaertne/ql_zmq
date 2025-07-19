use std::thread;

use arzmq::{
    ZmqResult,
    context::Context,
    message::Message,
    socket::{PeerSocket, Receiver, RecvFlags, SendFlags, Sender},
};

fn run_peer_server(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let peer = PeerSocket::from_context(context)?;
    peer.bind(endpoint)?;

    thread::spawn(move || {
        for _ in 1..=iterations {
            let message = peer.recv_msg(RecvFlags::empty()).unwrap();
            println!("Received request: {message}");

            let response: Message = "World".into();
            response
                .set_routing_id(message.routing_id().unwrap())
                .unwrap();
            peer.send_msg(response, SendFlags::empty()).unwrap();
        }
    });

    Ok(())
}

fn run_peer_client(context: &Context, endpoint: &str, iterations: i32) -> ZmqResult<()> {
    let peer = PeerSocket::from_context(context)?;
    let routing_id = peer.connect_peer(endpoint)?;

    for request_no in 1..=iterations {
        let request: Message = "Hello".into();
        request.set_routing_id(routing_id)?;
        println!("Sending request {request_no}");
        peer.send_msg(request, SendFlags::empty())?;

        let message = peer.recv_msg(RecvFlags::empty())?;
        println!("Received reply {request_no:2} {message}");
    }

    Ok(())
}

fn main() -> ZmqResult<()> {
    let endpoint = "inproc://arzmq-example-pair";
    let iterations = 10;

    let context = Context::new()?;

    run_peer_server(&context, endpoint, iterations)?;

    run_peer_client(&context, endpoint, iterations)?;

    Ok(())
}
