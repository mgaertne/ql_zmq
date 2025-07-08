#![cfg(feature = "examples-tokio")]
use core::sync::atomic::{AtomicI32, Ordering};

use azmq::{
    ZmqResult,
    context::ZmqContext,
    futures::{AsyncZmqReceiver, AsyncZmqSender},
    socket::{Dealer, Reply, ZmqSendFlags, ZmqSocket},
};
use tokio::{join, task};

static ITERATIONS: AtomicI32 = AtomicI32::new(0);

async fn run_replier(reply: ZmqSocket<Reply>) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        let message = reply.recv_multipart_async().await;
        let content = message.iter().last().unwrap();
        if !content.is_empty() {
            println!("Received request: {:?}", str::from_utf8(content).unwrap());
        }
        let up_to_last_idx = message.len() - 1;
        let mut multipart: Vec<Vec<u8>> = message.into_iter().take(up_to_last_idx).collect();
        multipart.push("World".as_bytes().to_vec());
        <ZmqSocket<Reply> as AsyncZmqSender<'_, Vec<u8>, Reply>>::send_multipart_async(
            &reply,
            multipart.into_iter(),
            ZmqSendFlags::empty(),
        )
        .await;
    }

    Ok(())
}

async fn run_dealer(dealer: ZmqSocket<Dealer>) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        let request_no = ITERATIONS.load(Ordering::Acquire);
        println!("Sending request {request_no}");
        let multipart = vec![vec![], "Hello".as_bytes().to_vec()];
        let _ = <ZmqSocket<Dealer> as AsyncZmqSender<'_, Vec<u8>, Dealer>>::send_multipart_async(
            &dealer,
            multipart.into_iter(),
            ZmqSendFlags::empty(),
        )
        .await;

        let message = dealer.recv_multipart_async().await;
        let content = message.iter().last().unwrap();
        if !content.is_empty() {
            println!(
                "Received reply {request_no:2} [{}]",
                str::from_utf8(content).unwrap()
            );

            ITERATIONS.fetch_sub(1, Ordering::Release);
        }
    }

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
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
