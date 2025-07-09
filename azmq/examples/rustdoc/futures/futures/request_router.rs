#![cfg(feature = "examples-futures")]
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use azmq::{
    ZmqResult,
    context::ZmqContext,
    futures::{AsyncZmqReceiver, AsyncZmqSender},
    socket::{Request, Router, ZmqSendFlags, ZmqSocket},
};
use futures::{executor::ThreadPool, join, task::SpawnExt};

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
static ITERATIONS: AtomicI32 = AtomicI32::new(0);

async fn run_router(router: ZmqSocket<Router>) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 1 {
        let mut multipart = router.recv_multipart_async().await;
        let content = multipart.pop_back().unwrap();
        if !content.is_empty() {
            println!("Received request: {content}");
        }
        multipart.push_back("World".into());
        router
            .send_multipart_async(multipart, ZmqSendFlags::empty())
            .await;
    }

    Ok(())
}

async fn run_requester(request: ZmqSocket<Request>) -> ZmqResult<()> {
    while ITERATIONS.load(Ordering::Acquire) > 0 {
        let request_no = ITERATIONS.load(Ordering::Acquire);
        println!("Sending request {request_no}");
        let _ = request
            .send_msg_async("Hello".into(), ZmqSendFlags::empty())
            .await;

        loop {
            if let Some(message) = request.recv_msg_async().await {
                println!("Received reply {request_no}: {message}");

                ITERATIONS.fetch_sub(1, Ordering::Release);

                break;
            }
        }
    }

    KEEP_RUNNING.store(false, Ordering::Release);

    Ok(())
}

#[cfg(feature = "examples-futures")]
fn main() -> ZmqResult<()> {
    let executor = ThreadPool::new().unwrap();
    futures::executor::block_on(async {
        ITERATIONS.store(10, Ordering::Release);

        let port = 5556;

        let context = ZmqContext::new()?;

        let router = ZmqSocket::<Router>::from_context(&context)?;
        router.bind(format!("tcp://*:{port}"))?;

        let request = ZmqSocket::<Request>::from_context(&context)?;
        request.connect(format!("tcp://localhost:{port}"))?;

        let request_handle = executor.spawn_with_handle(run_requester(request)).unwrap();
        let router_handle = executor.spawn_with_handle(run_router(router)).unwrap();

        let _ = join!(router_handle, request_handle);

        Ok(())
    })
}
