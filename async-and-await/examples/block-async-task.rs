use anyhow::Result;
use std::time::Duration;

// 强制 tokio 只使用一个工作线程，这样 task 2 不会跑到其它线程执行
#[tokio::main(worker_threads = 1)]
async fn main() -> Result<()> {
    // 先开始执行 task 1 的话会阻塞，让 task 2 没有机会运行
    tokio::spawn(async move {
        eprintln!("task 1");

        // 试试把这句注释掉看看会产生什么结果
        // tokio::time::sleep(Duration::from_millis(1)).await;
      
        // 死循环
        loop {}
    });

    tokio::spawn(async move {
        eprintln!("task 2");
    });

    tokio::time::sleep(Duration::from_millis(1)).await;
    Ok(())
}