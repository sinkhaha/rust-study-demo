use std::sync::{Arc, Mutex};
use std::task::Waker;

// 是携带闭包返回值的 Future
pub struct SpawnBlocking<T>(Arc<Mutex<Shared<T>>>);

// Shared结构体必须充当 Future 和 运行闭包的线程之间的结合点，因此它由 Arc 拥有并受 Mutex 保护（同步互斥锁）
struct Shared<T> {
    value: Option<T>,
    waker: Option<Waker>,
}

// 这里是同步函数，不是异步，返回一个SpawnBlocking的结构体，我们利用该结果体实现自己的Future
pub fn spawn_blocking<T, F>(closure: F) -> SpawnBlocking<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    // 创建Shared值
    let inner = Arc::new(Mutex::new(Shared {
        value: None,
        waker: None,
    }));

    // 启动一个线程来运行闭包，将结果存在在Shared的value字段中，并调用唤醒器（如果有的话）
    std::thread::spawn({
        let inner = inner.clone();
        move || {
            let value = closure(); // 运行闭包的线程会将其返回值保存在value中，然后调用waker

            let maybe_waker = {
                let mut guard = inner.lock().unwrap();
                guard.value = Some(value);
                guard.waker.take()
            };

            if let Some(waker) = maybe_waker {
                waker.wake(); // 调用唤醒器
            }
        }
    });

    SpawnBlocking(inner)
}

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

impl<T: Send> Future for SpawnBlocking<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut guard = self.0.lock().unwrap();

        // 轮询此Future会检查 value 是否存在，如果不存在则将唤醒器保存在waker中
        if let Some(value) = guard.value.take() {
            return Poll::Ready(value);
        }

        guard.waker = Some(cx.waker().clone()); // 保存唤醒器以备后用
        Poll::Pending
    }
}

// 运行 cargo run --example async-std-spawn-blocking
fn main() {
    async_std::task::block_on(async {
        for i in 0..1000 {
            assert_eq!(spawn_blocking(move || i).await, i);
        }
    });

    async_std::task::block_on(async {
        let futures: Vec<_> = (0..100).map(|i| (i, spawn_blocking(move || i))).collect();

        for (i, f) in futures {
            assert_eq!(f.await, i);
        }
    });
}
