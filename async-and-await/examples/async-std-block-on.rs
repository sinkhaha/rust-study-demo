use crossbeam::sync::Parker;
use futures_lite::pin;
use std::future::Future;
use std::task::{Context, Poll};
use waker_fn::waker_fn;

// 实现 block_on
pub fn block_on<F: Future>(future: F) -> F::Output {
    let parker = Parker::new();
    let unparker = parker.unparker().clone();
    let waker = waker_fn(move || unparker.unpark());
    let mut context = Context::from_waker(&waker);

    // 这里用到了 pin 宏
    pin!(future);

    //
    loop {
        // as_mut方法会解引用指针
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => parker.park(), // parker.park()阻塞线程，知道其他人在相应的Unparker上调用.unpark()
        }
    }
}

// 运行 cargo run --example async-std-block-on
fn main() {
    assert_eq!(block_on(std::future::ready(42)), 42);

    use async_std::task::{sleep, spawn};
    use futures_lite::FutureExt;
    use std::time::Duration;

    assert_eq!(
        block_on({
            let one_sec = async {
                sleep(Duration::from_secs(1)).await;
                43
            };
            let half_sec = async {
                sleep(Duration::from_millis(500)).await;
                44
            };
            spawn(one_sec.race(half_sec))
        }),
        44
    );
}
