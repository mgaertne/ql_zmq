#![cfg(feature = "examples-async-std")]
use core::sync::atomic::{AtomicI32, Ordering};

use async_std::task;
use azmq::{
    ZmqResult,
    context::ZmqContext,
    futures::{AsyncZmqReceiver, AsyncZmqSender},
    socket::{Dealer, Reply, ZmqSendFlags, ZmqSocket},
};
use futures::join;

static ITERATIONS: AtomicI32 = AtomicI32::new(0);

async fn run_replier(reply: ZmqSocket<Reply>) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        let mut multipart = reply.recv_multipart_async().await;
        let content = multipart.pop_back().unwrap();
        if !content.is_empty() {
            println!("Received request: {content}");
        }
        multipart.push_back("World".into());
        reply
            .send_multipart_async(multipart, ZmqSendFlags::empty())
            .await;
    }

    Ok(())
}

async fn run_dealer(dealer: ZmqSocket<Dealer>) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        let request_no = ITERATIONS.load(Ordering::Acquire);
        println!("Sending request {request_no}");
        let multipart = vec![vec![].into(), "Hello".into()];
        let _ = dealer
            .send_multipart_async(multipart.into(), ZmqSendFlags::empty())
            .await;

        let mut message = dealer.recv_multipart_async().await;
        let content = message.pop_back().unwrap();
        if !content.is_empty() {
            println!("Received reply {request_no}: {content}",);

            ITERATIONS.fetch_sub(1, Ordering::Release);
        }
    }

    Ok(())
}

#[async_std::main]
async fn main() -> ZmqResult<()> {
    ITERATIONS.store(10, Ordering::Release);

    let port = 5556;

    let context = ZmqContext::new()?;

    let reply = ZmqSocket::<Reply>::from_context(&context)?;
    reply.bind(format!("tcp://*:{port}"))?;

    let dealer = ZmqSocket::<Dealer>::from_context(&context)?;
    dealer.connect(format!("tcp://localhost:{port}"))?;

    let dealer_handle = task::spawn(run_dealer(dealer));
    let reply_handle = task::spawn(run_replier(reply));

    let _ = join!(reply_handle, dealer_handle);

    Ok(())
}
