# 1 异步

异步任务类似于线程，但其创建速度更快，在它们之间可以更有效地传递控制权，并且其内存开销比线程少一个数量级。利用 `Rust` 异步任务可以在单个线程 或 线程池中交替执行许多批次独立的活动。



**异步任务的优点**

* 异步任务可以使用更少的内存

  > 在 `Linux`，一个线程内存使用量至少 `20KiB`，包括用空间和内核空间的内存使用量

* 异步任务创建速度更快

  > `Linux` 上创建一个线程大约要 15 微秒，而启动一个异步任务大约 300 纳秒，快了 50 倍

* 异步任务之间的上下文切换比操作系统线程之间的上下文切换更快

  > `Linux` 上这两个操作所需时间分别是 0.2 微秒和 1.7 微秒



## 1.1 Future 定义

`Future` 是 `Rust` 异步编程的核心， [Future](https://doc.rust-lang.org/std/future/trait.Future.html) 特型的定义如下：

```rust
pub trait Future {
    type Output;
    // 下面会讲到Pin类型，这里可以先把Pin<&mut Self> 当作&mut Self理解
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

由代码可知

* `Future` 有一个关联类型 `Output`

* 还有一个 `poll()` 轮询方法，它返回 `Poll<Self::Output>`。`Poll` 是个枚举，有 `Ready` 和 `Pending` 两个状态

通过调用 `poll()` 方法可以推进 `Future` 的进一步执行。`Future` 的 `poll()` 方法不会等待操作完成，它总是立即返回，如果 `Future` 完成了，则 `poll()` 会返回 `Poll::Ready(output)`，即得到 `Future` 的值并返回，其中 `output` 就是 `Future` 的最终结果；若 `Future` 还没完成，则返回 `Poll::Pending()`，此时 `Future` 会被挂起，需要等某个事件将其唤醒（可以通过调用 `Context` 中提供的回调函数 `waker` 来唤醒 `Future`）

> 即使 `Future` 被过度轮询，它也只会永远返回 `Poll::Pending`



## 1.2 async/await

`async/await` 是 `Rust` 的异步编程模型，是产生和运行并发任务的手段。一般而言，`async` 定义了一个可以并发执行的任务，而 `await` 则触发这个任务并发执行。

* `async` 的作用：用来创建 `Future`，可以修饰函数，即异步函数；也可以修饰代码块，即异步块
* `await` 的作用：`await` 来触发 `Future` 的执行，并等待 `Future` 执行完毕。在 `async` 异步函数中使用 `await` 可以等待另一个异步调用（`Future`）的完成，这样相当于使用同步的方式实现了异步的执行效果。注意 `await` 不会阻塞当前的线程，而是异步的等待当前的 `Future` 的完成，在等待的过程中，该线程还可以继续执行其他的 `Future`，最终实现了并发处理的效果。`await` 要在 `async` 函数或 `async` 块中才能使用，同步函数中不能使用 `await`。



`JavaScript` 也是通过 `async` 的方式提供了异步编程，`Rust` 的 `Future` 跟 `JavaScript` 的 `Promise` 非常类似。它们的区别是：`JavaScript` 的 `Promise` 一旦创建就开始执行，对 `Promise` 的 `await` 只是等待这个`Promise` 执行完成并得到结果。而`Rust` 的 `Future`，只有在主动 `await` 进行调度后才开始执行。



`async/await` 只是一个语法糖，它使用状态机将 `Future` 包装起来进行处理。

> `Rust 1.75.0` 版本才支持特型里有异步方法，也可以用 `async-trait crate`，它提供了基于宏的解决方案



## 1.3 同步/多线程/异步例子

假如有一个需求：读取 `Cargo.toml` 和 `Cargo.lock` 并将它们转换成 `yaml` 写入 `/tmp` 文件夹下。下面分别用同步、多线程和异步的方式，来实现这个功能



### 1.3.1 同步的实现方式

> 完整代码可看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/sync-io.rs)

```rust
use anyhow::Result;
use serde_yaml::Value;
use std::fs;

fn main() -> Result<()> {
    // 读取 Cargo.toml，IO 操作 1，这里等待系统调用，会阻塞
    let content1 = fs::read_to_string("./Cargo.toml")?;
    // 读取 Cargo.lock，IO 操作 2
    let content2 = fs::read_to_string("./Cargo.lock")?;

    // 计算
    let yaml1 = toml2yaml(&content1)?;
    let yaml2 = toml2yaml(&content2)?;

    // 写入 /tmp/Cargo.yml，IO 操作 3
    fs::write("/tmp/Cargo.yml", &yaml1)?;
    // 写入 /tmp/Cargo.lock，IO 操作 4
    fs::write("/tmp/Cargo.lock", &yaml2)?;

    println!("{}", yaml1);
    println!("{}", yaml2);

    Ok(())
}

fn toml2yaml(content: &str) -> Result<String> {
    let value: Value = toml::from_str(&content)?;
    Ok(serde_yaml::to_string(&value)?)
}
```

**分析：**

* 因为 `std::fs::read_to_string` 和 `std::fs::write` 是同步方法，所以代码中有 4 处地方都要同步等待系统 `I/O` 操作，每一处都会阻塞主线程，也就是在系统调用完之前，此时是单线程，它就阻塞了，不能做任何其他事情。比如在读 `Cargo.toml` 时，整个主线程被阻塞，直到 `Cargo.toml` 读完，才能继续读 `Cargo.lock` 文件

* 整个主线程，只有在运行 `toml2yaml` 的时间片内，才真正在执行计算任务，读取文件以及写入文件等这些 `IO` 操作，`CPU` 都在闲置

上面同步方式的两个问题可以用多线程来解决



### 1.3.2 多线程的实现方式

此方式利用多线程，把文件的读取和文件的写入放在单独的线程中执行

> 完整代码可看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/thread-io.rs)

```rust
use anyhow::{anyhow, Result};
use serde_yaml::Value;

use std::{
    fs,
    thread::{self, JoinHandle},
};

/// 包装一下 JoinHandle，这样可以提供额外的方法
struct MyJoinHandle<T>(JoinHandle<Result<T>>);

impl<T> MyJoinHandle<T> {
    /// 等待 thread 执行完（类似 await）
    pub fn thread_await(self) -> Result<T> {
        self.0.join().map_err(|_| anyhow!("failed"))?
    }
}

fn main() -> Result<()> {
    let t1 = thread_read("./Cargo.toml");
    let t2 = thread_read("./Cargo.lock");

    let content1 = t1.thread_await()?;
    let content2 = t2.thread_await()?;

    // 计算
    let yaml1 = toml2yaml(&content1)?;
    let yaml2 = toml2yaml(&content2)?;

    let t3 = thread_write("/tmp/Cargo.yml", yaml1);
    let t4 = thread_write("/tmp/Cargo.lock", yaml2);

    let yaml1 = t3.thread_await()?;
    let yaml2 = t4.thread_await()?;

    fs::write("/tmp/Cargo.yml", &yaml1)?;
    fs::write("/tmp/Cargo.lock", &yaml2)?;

    println!("{}", yaml1);
    println!("{}", yaml2);

    Ok(())
}

// 针对读文件单独开一个线程
fn thread_read(filename: &'static str) -> MyJoinHandle<String> {
    let handle = thread::spawn(move || {
        let s = fs::read_to_string(filename)?;
        Ok::<_, anyhow::Error>(s)
    });
    MyJoinHandle(handle)
}

// 针对写文件单独开一个线程
fn thread_write(filename: &'static str, content: String) -> MyJoinHandle<String> {
    let handle = thread::spawn(move || {
        fs::write(filename, &content)?;
        Ok::<_, anyhow::Error>(content)
    });
    MyJoinHandle(handle)
}

fn toml2yaml(content: &str) -> Result<String> {
    let value: Value = toml::from_str(&content)?;
    Ok(serde_yaml::to_string(&value)?)
}
```

**分析**

* 分别用线程读取每个文件，且读取两个文件是并发执行（写入也类似），缩短了读取文件等待文件 `I/O` 操作的时间，读取的总共等待的时间是 `max(time_for_file1, time_for_file2)`

* 该方式也有一个缺点：不适用于同时读写太多文件的场景；因为每读一个文件会创建一个线程，在操作系统中，线程的数量是有限的，创建过多的线程会大大增加系统的开销

  > 一个线程的栈大小也可能有数十或数百 `KB`，一个大的系统可能有很多线程，如果同时开了很多线程在同时运行，可能会让开销变得很高

大多数操作系统对 `I/O` 操作提供了非阻塞接口，更理想的方式是利用 `Rust` 的异步处理，进而最大程度的利用 `CPU` 资源



### 1.3.3 异步的实现方式

下面使用异步的方式来实现需求，这里使用 `async-std` 异步运行时库，一般任何函数的异步版本都会接受与其同步版本完全相同的参数，但返回值类型包裹在 `Future` 中。`async-std` 库提供了大量和标准库 `std` 同名方法的异步版本

> 也可以使用 `tokio` 异步运行时库



**异步实现代码如下**

> 完整代码可看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/async-std-io.rs)

 ```rust
use anyhow::Result;
use async_std::{fs, task};
use futures::try_join;
use serde_yaml::Value;

fn toml2yaml(content: &str) -> Result<String> {
    let value: Value = toml::from_str(&content)?;
    Ok(serde_yaml::to_string(&value)?)
}

// 异步函数
async fn read_toml2yaml() -> Result<()> {
    // 这里使用异步的read_to_string，注意后面没有加 await
    let f1 = fs::read_to_string("./Cargo.toml");
    let f2 = fs::read_to_string("./Cargo.lock");

    // 这里用try_join宏等待两个异步任务完成
    let (content1, content2) = try_join!(f1, f2)?;

    // 计算
    let yaml1 = toml2yaml(&content1)?;
    let yaml2 = toml2yaml(&content2)?;

    let f3 = fs::write("/tmp/Cargo.yml", &yaml1);
    let f4 = fs::write("/tmp/Cargo.lock", &yaml2);

    try_join!(f3, f4)?;

    println!("{}", yaml1);
    println!("{}", yaml2);

    Ok(())
}

// main还是一个同步方法
fn main() -> Result<()> {
    // main是同步方法，不能使用await，所以用了block_on
    // 这里用了 block_on 去调度和执行 read_toml2yaml 返回的 Future，这里会阻塞等待 Future 的完成
    task::block_on(read_toml2yaml())?;
    Ok(())
}
 ```

**分析**

* 这里使用了 `async_std::fs` 提供的异步读写文件方法，而不是 `std::fs`，`read_toml2yaml` 使用了 `async`，是异步函数
*  `async_std::fs` 的文件操作会返回一个 `Future`，所以调用这个异步的 `read_to_string` 实际并没有立即读取文件内容，它唯一的职责是构建并返回一个 `Future`，该 `Future` 会在被轮询到时完成其真正的读取文件工作（这个 `Future` 必须包含执行调用请求所需的全部信息）。接着用 `try_ join!` 宏轮询这些 `Future`，得到它们运行后的结果。此时文件读取的总时间是 `max(time_for_file1, time_for_file2)`，此异步版本的性能和使用线程的版本几乎一致，但是消耗的线程资源要少很多
* `try_join!` 宏的作用：用来轮询多个 `Future` 是否完成，它会依次处理每个 `Future`，遇到阻塞就处理下一个，直到所有 `Future` 产生结果

* 这里`main` 函数依然是同步方法，因为同步方法不能使用 `await` 表达式，所以要调用异步函数并等待它执行，可以使用 `async_std::task::block_on` 方法， `block_on` 会调度和执行 `Future`，它会阻塞等待 `Future` 的完成，效果类似于 `await`
* 异步函数本身会返回 `Future` ，`Rust` 会自动把 `async fn f(...) -> T` 函数的返回值视为承载 `T` 的 `Future`，而非直接的 `T` 值



**注意：**

* 这里不能写成在异步方法后面面添加 `await` 表达式的方式，因为添加 `await ` 会轮询 `Future` ，一直到该 `Future` 执行结束，所以在这里依旧是先读取 `Cargo.toml`，再读取 `Cargo.lock`，并没有达到并发的效果，此时就和同步的版本没有区别，例如

```rust
let content1 = fs::read_to_string("./Cargo.toml").await?;
```

* 因为异步函数被调用时，它会在函数体开始执行之前立即返回，得到的只是承载它最终值的 `Future`，而不是结果值，所以在 `main()` 中如果没有使用 `block_on()` 等待 `read_toml2yaml()` 执行完成，则此时 `response` 是 `Result<()>` 型的 `Future`，该 `Future` 也不会被调度执行，最终导致 `read_toml2yaml` 函数也不会被执行，如

```rust
fn main() -> Result<()> {
    let response = read_toml2yaml();
    Ok(())
}
```

异步函数返回的 `Future` 中包含函数体运行时所需的一切信息：函数的参数、局部变量的内存空间等。所以这里`response` 必须保存传给 `read_toml2yaml` 的值（如果有参数的话），因为 `read_toml2yaml` 的函数体将需要这些值来运行。



**如果使用 `await` 表达式实现 `Future` 的调度，则代码如下**

```rust
use anyhow::Result;
use async_std::{fs, task};
use serde_yaml::Value;

fn toml2yaml(content: &str) -> Result<String> {
    let value: Value = toml::from_str(&content)?;
    Ok(serde_yaml::to_string(&value)?)
}

// read_toml2yaml是异步函数
async fn read_toml2yaml() -> Result<()> {
    // 这里加了 await 等待读取 Cargo.toml 完成，Cargo.toml读取完成了才会继续执行读取 Cargo.lock的代码
    let content1 = fs::read_to_string("./Cargo.toml").await?;
    let content2 = fs::read_to_string("./Cargo.lock").await?;

    // 计算
    let yaml1 = toml2yaml(&content1)?;
    let yaml2 = toml2yaml(&content2)?;

    // 这里也加了 await
    fs::write("/tmp/Cargo.yml", &yaml1).await?;
    fs::write("/tmp/Cargo.lock", &yaml2).await?;

    println!("{}", yaml1);
    println!("{}", yaml2);

    Ok(())
}

fn main() -> Result<()> {
    task::block_on(read_toml2yaml())?;
    Ok(())
}
```

**分析**

* 当首次轮询 `read_toml2yaml` 返回的 `Future` 时，会从函数体的顶部开始执行

* 当运行到 `fs::read_to_string("./Cargo.toml")` 后面的 `await`，`await` 表达式会轮询 `read_to_string` 返回的 `Future`，等待 `Future` 完成，如果还没完成，则向调用者返回 `Poll::Pending`，程序不能从这个 `await` 继续向下面运行了，直到后面对这个 `Future` 的再次轮询，且 `Future` 完成了返回了 `Poll::Ready` 才会继续执行下面读取 `Cargo.lock` 的代码 

  

`fs::read_to_string("./Cargo.toml").await` 的执行流程大概可以理解为

```rust
{
  // 伪代码
  let conntct_future = fs::read_to_string("./Cargo.toml");
  
  'retry_point:
   match connect_future.poll(cx) {
     Poll::Ready(value) => value,
     Poll::Pending => {
        // 安排对read_toml2yaml返回的Future进行下一次Poll，以便在'retry_point处恢复执行
        ...
        return Poll::Pending;
     }
  }
}
```

* `await` 表达式会获取 `Future` 的所有权，然后轮询它，如果已经完成，那么 `Future` 的最终值就是 `await` 表达式的值，然后继续执行，否则，此 `Future` 返回 `Poll::Pending`

* 注意下一次对 `read_toml2yaml` 返回的 `Future` 进行轮询时不会再从函数的顶部开始，而是会在即将轮询`connect_future` 的中途时间点恢复执行函数。`read_toml2yaml` 返回的 `Future` 会跟踪下一次 `poll` 应该恢复的点，以及恢复该点所需的所有本地状态，比如变量、参数和临时变量。



# 2 Rust 的异步模式

## 2.1 Reactor Pattern 模式

`Rust` 使用 `Future` 做异步处理是一个典型的 `Reactor Pattern` 模式。



`Reactor Pattern` 是构建高性能事件驱动系统的一个很典型模式，`executor` 和 `reactor` 是 `Reactor Pattern `的组成部分。`Reactor pattern` 包含三部分：

* `task`：待处理的任务。任务可以被打断，并且把控制权交给 `executor`，等待之后的调度
* `executor`：一个调度器。维护等待运行的任务（`ready queue`），以及被阻塞的任务（`wait queue`）
* `reactor`：维护事件队列。当事件来临时，通知 `executor` 唤醒某个任务等待运行

`executor` 会调度执行待处理的任务，当任务无法继续进行却又没有完成时，它会挂起任务，并设置好合适的唤醒条件。之后，如果 `reactor` 得到了满足条件的事件，它会唤醒之前挂起的任务，然后 `executor` 就有机会继续执行这个任务。这样一直循环下去，直到任务执行完毕。



## 2.2 执行器(executor)

任何使用了协程来处理并发的程序，都需要有一个 `executor` 来负责协程（`Future`）的调度。因为操作系统只负责调度线程，它不会去调度用户态的协程（比如 `Future`）。



`Rust` 只提供 `Future` 协程，它在语言层面并不提供执行器 `executor`，当不需要使用协程时，不需要引入任何运行时；而需要使用协程时，可以在生态系统中选择最合适的 `executor`。`Golang` 确相反，它也支持协程，但在语言层面自带了一个用户态的调度器。



`Rust` 的 `Future` 是惰性的。在 `Rust` 中，调用异步函数后其实什么都不会做，调用完只是立即返回一个 `Future`，只有 `Future`  被 `poll` 轮询时才会被执行

* 一个推动 `Future` 的方式就是通过 `async_std::task::block_on` 等函数，这些函数将轮询并驱动它直到完成，这些函数成为执行器，它们承担着与其他语言中全局事件循环类似的职责
* 另一个推动它的方式就是调用异步函数后使用 `.await` ，此时也会轮询 `Future` 直到完成；注意 `await` 只能在 `async` 中使用，那些最外层的 `async` 函数，需要靠执行器（`executor`） 来推动 



 `Rust` 有如下几种常见的 `executor` ：

* [futures](https://github.com/rust-lang/futures-rs)：这个库自带了很简单的 `executor`
* [tokio](https://github.com/tokio-rs/tokio)：提供 `executor`，当使用 `#[tokio::main]` 时，就隐含引入了 `tokio` 的 `executor`
* [async-std](https://github.com/async-rs/async-std) ：提供 `executor`，和 `tokio` 类似
* [smol](https://github.com/smol-rs/smol) ：提供 `async-executor`，主要提供了 `block_on`



## 2.3 通知机制(wake)

`executor` 执行器会管理一批 `Future` 

1. 最开始，执行器会先 `poll` 轮询一次 `Future` （后面就不会主动去 `poll` 了）
2. 如果 `Future` 的值完成了，就返回它
3. 如果 `Future` 的值未完成，则返回 `Poll::Pending`（此时会将 `Context` 中唤醒器 `Waker` 的克隆体存储在某处），并挂起 `Future`，执行器会进入休眠状态
4. 当 `Future` 需要再次被轮询时（比如收到系统某个通知事件后），`Future` 会通过调用其唤醒器 `Waker` 的 `wake()` 方法去主动通知执行器，执行器就会醒来再次轮询 `Future`，这种 `wake` 通知然后再 `poll` 的方式会不断重复，一直到 `Future` 完成为止

> 注意：如果没有执行器，单独调用异步函数和异步块后返回的 `Future` 是不会被执行的



`Waker` 的定义和相关的代码非常抽象，内部使用了一个 `vtable` 来允许各种各样的 `waker` 的行为：

```rust
pub struct RawWakerVTable {
    clone: unsafe fn(*const ()) -> RawWaker,
    wake: unsafe fn(*const ()),
    wake_by_ref: unsafe fn(*const ()),
    drop: unsafe fn(*const ()),
}
```

* `Waker` 中 `wake()` 方法的作用是告诉执行器，相关的任务可以被唤醒了，此时执行器就可以对相应的 `Future` 再次进行 `poll` 操作。

* `Waker` 实现了 `Clone` 和 `Send` 特型，因此 `Future` 总是可以克隆自己的唤醒器副本并根据需要将其发送到其他线程，`Waker::wake` 方法会消耗此唤醒器。



`Context` 是 `Waker` 的一个封装，`Future` 的 `poll` 方法里用到 `Context` ，其定义如下 ：

```rust
pub struct Context<'a> {
    waker: &'a Waker,
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}
```



`Rust` 自身不提供异步运行时，它只在标准库里规定了一些基本的接口，可以由各个运行时自行决定怎么实现。所以在标准库中，只能看到这些接口的定义，以及“高层”接口的实现，比如 `Waker` 下的 `wake` 方法，只是调用了 `vtable` 里的 `wake()` 而已。`vtable` 具体的实现并不在标准库中，而是在第三方的异步运行时里，比如 `futures`库的 `waker vtable` [定义](https://github.com/rust-lang/futures-rs/blob/master/futures-task/src/waker.rs)

```rust
impl Waker {
    /// Wake up the task associated with this `Waker`.
    #[inline]
    pub fn wake(self) {
        // The actual wakeup call is delegated through a virtual function call
        // to the implementation which is defined by the executor.
        let wake = self.waker.vtable.wake;
        let data = self.waker.data;

        // Don't call `drop` -- the waker will be consumed by `wake`.
        crate::mem::forget(self);

        // SAFETY: This is safe because `Waker::from_raw` is the only way
        // to initialize `wake` and `data` requiring the user to acknowledge
        // that the contract of `RawWaker` is upheld.
        unsafe { (wake)(data) };
    }
    ...
}
```



## 2.4 运行时(tokio)的异步处理流程

`Rust` 使用 `Future` 做异步处理就是一个典型的 `Reactor Pattern` 模式。以 `tokio` 为例：`async/await` 提供语法层面的支持，`Future` 是异步任务的数据结构，当 `.await` 时，`executor` 就会调度并执行它。



`tokio` 的调度器会运行在**多个线程**上，运行线程上自己的 `ready queue` 上的任务（`Future`），如果没有，就去别的线程的调度器上偷一些过来运行（`work-stealing` 调度机制）。当某个任务无法再继续取得进展，此时 `Future` 运行的结果是 `Poll::Pending`，那么调度器会挂起任务，并设置好合适的唤醒条件（`Waker`），等待被 `reactor` 唤醒。而 `reactor` 会利用操作系统提供的异步 `I/O`（如 `epoll / kqueue / IOCP`），来监听操作系统提供的 `IO` 事件，当遇到满足条件的事件时，就会调用 `Waker.wake()` 唤醒被挂起的 `Future`，这个 `Future` 会回到 `ready queue` 等待执行。



整个流程如下：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-05%2019.15.06.png)



以具体的代码示例来理解这个过程:

> 完整代码看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/tcp-listener.rs)

```rust
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LinesCodec};

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
  
    println!("listen to: {}", addr);
  
    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted: {:?}", addr);
      
        // 创建异步任务
        tokio::spawn(async move {
            // 使用 LinesCodec 把 TCP 数据切成一行行字符串处理
            let framed = Framed::new(stream, LinesCodec::new());
          
            // split 成 writer 和 reader
            let (mut w, mut r) = framed.split();
          
            for line in r.next().await {
                // 每读到一行就加个前缀发回
                w.send(format!("I got: {}", line?)).await?;
            }
          
            Ok::<_, anyhow::Error>(())
        });
    }
}
```

这是一个简单的 `TCP` 服务器，服务器每收到一个客户端的请求，就会用 `tokio::spawn` 创建一个异步任务，放入 `executor` 中执行。这个异步任务接受客户端发来的按行分隔（分隔符是` “\r\n”`）的数据帧，服务器每收到一行，就加个前缀把内容也按行发回给客户端。



假设客户端输入了很大的一行数据，服务器在执行` r.next().await ` 时，如果接收不完一行的数据时，这个 `Future` 返回 `Poll::Pending`，此时它被挂起。当后续客户端的数据到达时，`reactor` 会知道这个 `socket` 上又有数据了，于是找到 `socket` 对应的 `Future`，将其唤醒，继续接收数据。这样反复下去，最终 ` r.next().await` 得到 `Poll::Ready(Ok(line))`，于是它返回 `Ok(line)`，程序继续往下走，进入到 `w.send()` 的阶段。



可以用 `telnet` 来测试：

```rust
❯ telnet localhost 8080
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
hello
I got: hello
Connection closed by foreign host.
```



## 2.5 运行时(async_std)的一些方法

下面介绍一些 `async_std` 运行时库的一些常用的方法

### 2.5.1 同步阻塞轮询Future：block_on

因为异步函数本身会返回 `Future`，所以调用者要以某种方式对它进行轮询，且还得等待一个它的结果，由于同步函数不能用 `await` 等待异步任务。如果要从同步代码调用异步函数，可以使用 `async_std::task::block_on`。`block_on` 是一个会生成异步函数最终值的同步函数，可以将其视为从异步到同步的适配器。前面异步的例子就使用了 `block_on`，如

```rust
use anyhow::Result;
use async_std::task;

fn main() -> Result<()> {
    
    // 使用task::block_on
    let response = task::block_on(read_toml2yaml())?;
  
    println!("{}", response);

    Ok(())
}
```

`block_on` 的作用在于知道如何进入休眠（比如 `read_toml2yaml` 里所有 `Future` 都没完成时会休眠），直到 `Future` 真正值得再次轮询时再启动轮询（比如 `read_to_string` 返回的 `Future` 完成了才再次轮询），而不是浪费处理器时间和电池寿命疯狂的不断的 `poll` 轮询。



注意： `block_on` 是阻塞式的，意味着不应该在异步函数中使用它，因为在值被准备好之前它会一直阻塞整个线程。在异步函数中还是推荐用使用 `await`。



### 2.5.2 单线程中启动异步任务：spawn_local

因为在 `Future` 的值完成前，`async_std::task::block_on` 会一直阻塞，这样的效果不一定比同步要好。此时就需要让线程在等待的同时可以做其他工作，要实现这个目的，可以让 `block_on` 结合 `async_std::task::spawn_local` 一起使用。



`async_std::task::spawn_local` 函数用于在**本线程**启动一个新的异步任务，它会接受一个 `Future` 并将其添加到任务池中，这个任务池是当前线程在调用 `block_on` 时要轮询的任务池，它会返回自己的 `async_std::task::JoinHandle` 类型，它本身就是一个 `Future`，可以等待（`await`）它以获取 `Future` 的最终值。



如果使用 `spawn_local` 将一堆 `Future` 添加到任务池中，那么 `block_on` 就会在可以向前推进时轮询每个 `Future`，并行执行整个任务池，直到每个 `Future` 的结果都完成。只要当前正在阻塞着 `block_on` 的 `Future` 还未完成，` block_on` 就会转移目标，继续轮询任务池里的下一个异步任务，直到所有任务都执行完成。但是当所有可轮询的 `Future` 都未完成，`block_on` 也会进入休眠状态，直到再次被唤醒。



**使用 block_on 结合 spawn_local 的例子**

假如现在要实现同时发出一批 `http` 请求。要使用 `spawn_local`，还需要启动 `async-std` 的 `unstable` 特性，如

```toml
// Cargo.toml
[dependencies]
async-std = { version = "1.12.0", features = ["unstable"] }
```

先看最初版本的代码

```rust
use async_std::io::prelude::*;
use async_std::net;

// 打开到Web服务器的TCP连接，并发送一个请求
async fn cheapo_request(host: &str, port: u16, path: &str) -> std::io::Result<String> {
    // 这里加了 await
    let mut socket = net::TcpStream::connect((host, port)).await?;

    let request = format!("GET {} HTTP/1.1 \r\nHost: {}\r\n\r\n", path, host);

    // 这里加了 await
    socket.write_all(request.as_bytes()).await?;

    socket.shutdown(net::Shutdown::Write)?;

    let mut response = String::new();

    // 这里加了 await
    socket.read_to_string(&mut response).await?;

    Ok(response)
}

// 发起多个请求
pub async fn many_requests(requests: Vec<(String, u16, String)>) -> Vec<std::io::Result<String>> {
    use async_std::task;

    let mut handles = vec![];
    for (host, port, path) in requests {
        // 将每个调用返回的Future传给 spawn_local，然后将返回的JoinHandle放入向量中
        handles.push(task::spawn_local(cheapo_request(&host, port, &path)));
    }

    let mut results = vec![];
    for handle in handles {
        // 等待每个 JoinHandle
        results.push(handle.await);
    }

    results
}
```

这个例子可以任意等待这些 `JoinHandle`，因为请求已经发出，因此只要此线程调用了 `block_on` 并且没有更有价值的事情可做，请求的 `Future` 就会根据需要进行轮询。所以请求都将并行执行，一旦完成操作，`many_requests` 会把所有结果返回给它的调用者。



但是此代码会报错，因为这里把引 `&host` 和 `&path` 传给了 `cheapo_request` ，生命周期有问题。问题是 `spawn_local` 无法确定我们会在 `host` 和 `path` 被丢弃之前等待任何完成，事实上，`spawn_local` 只会接受生命周期为 `'static` 的 `Future`，因为也可以简单地忽略它返回的 `JoinHandle`，并在程序执行其他部分时让此任务继续运行。



**这里可以得出一个结论**：

如果将引用传给一个异步函数，那么它返回的 `Future` 就必须持有这些引用，因此安全起见，`Future` 的生命周期不能超出它们借来的值。



解决方式是创建一个接受这些参数的拥有型版本的异步函数 `cheapo_owning_request` 来包裹一下 `cheapo_request`，`many_requests` 改成调用 `cheapo_owning_request` 且用 `await` 等待其执行即可

```rust
async fn cheapo_owning_request(host: String, port: u16, path: String) -> std::io::Result<String> {
   // 在这里调用cheapo_request函数，且使用 await 等待
    cheapo_request(&host, port, &path).await 
}

pub async fn many_requests(requests: Vec<(String, u16, String)>) -> Vec<std::io::Result<String>> {
    use async_std::task;

    let mut handles = vec![];
    for (host, port, path) in requests {
        // 改成调用 cheapo_owning_request
        handles.push(task::spawn_local(cheapo_owning_request(host, port, path)));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}

// 使用 block_on 进行调用
fn main() {
    let requests = vec![
        ("a.com".to_string(), 80, "/".to_string()),
        ("b.com".to_string(), 80, "/".to_string()),
        ("c.com".to_string(), 80, "/".to_string()),
    ];

    let results = async_std::task::block_on(many_requests(requests));

    for result in results {
        match result {
            Ok(response) => println!("{}", response),
            Err(err) => eprintln!("error: {}", err),
        }
    }
}
```

* `cheapo_owning_request` 函数会接受 `String` 而不是 `&str` 引用，因此它的 `Future` 拥有 `host` 字符串和 `path` 字符串本身，并且其生命周期为 `'static`。`Rust` 的借用检查器可以发现它立即开始等待 `cheapo_request` 返回的 `Future`，因此，如果该 `Future` 被轮询，那么它借来的 `host` 变量和 `path` 变量必然依旧存在，所以没问题

* 代码通过 `block_on` 对 `many_requests` 的调用，启动了 3 个异步任务，假设它们分别是 `A、B、C`。`block_on` 首先轮询 `A`，如果 `A` 中的 `Future` 还没完成，则 `block_on` 会继续轮询下一个任务，比如轮询 `B`，最后轮询 `C`。当所有可轮询的 `Future` 都返回了 `Poll::Pending` 时，`block_on` 也会进入休眠状态，直到 `TcpStream::connect` 返回的 `Future` 表明它的任务需要再次轮询时才唤醒



**异步任务和线程的区别**：

* `spawn_local` 不会创建一个新的线程，所以它可以减少线程创建和同步的开销，从而提高性能，它本质还是异步任务的方式
* 从一个异步任务到另一个异步任务到切换只会出现在 `await` 表达式处，且只有当等待的 `Future` 返回了 `Poll::Pending` 时才会发生。这意味着如果在 `cheapo_request` 中放置了一个长时间运行的计算，那么传给 `spawn_local` 的其他任务在它完成之前全都没有机会运行，使用线程则不会出现这个问题，因为操作系统可以在任何时候挂起任何线程，并设置定时器以确保没有哪个线程会独占处理器。所以必要时需要使用多线程的方式去实现，而不是异步



### 2.5.3 线程池中启动异步任务：spawn

`async_std` 运行时还有一个 `async_std::task::spawn`，它跟 `spawn_local`类似，使用 `async_std::task::spawn` 可以在工作线程池中启动 `Future`，线程池专门用于轮询那些已准备好向前推进的 `Future`。



`async_std::task::spawn` 和 `async_std::task::spawn_local` 主要区别是在于任务的运行上下文

* `async_std::task::spawn_local` 用于在当前任务的本地运行上下文中执行子任务，它要求子任务必须是 `'static` 生命周期的。这意味着子任务的所有权不会被移交给异步运行时，而是在当前任务的上下文中保持活动
* `async_std::task::spawn`：该函数用于在异步运行时的全局上下文中执行子任务。它可以接受非 `'static` 生命周期的子任务。**这意味着子任务的所有权会被移交给异步运行时，由异步运行时负责管理子任务的生命周期**。使用`spawn`函数，子任务可以在异步运行时的任务队列中独立运行，与父任务的生命周期无关。这种方式更加通用，但需要注意潜在的线程安全问题，因为子任务可能同时访问共享的资源。



例子

```rust
pub async fn many_requests_async_spawn(
    requests: Vec<(String, u16, String)>,
) -> Vec<std::io::Result<String>> {
    use async_std::task;

    let mut handles = vec![];
    for (host, port, path) in requests {
       // 这里使用 async_std的spawn方法
        handles.push(task::spawn(async move {
            cheapo_request(&host, port, &path).await
        }));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}
```

`spawn` 和 `spawn_local` 一样，返回一个 `JoinHandle` 值，可以等待它以获得 `Future` 的最终值。但与 `spawn_local` 不同的是，`Future` 不用等到调用 `block_on` 才进行轮询，一旦线程池中的某个线程空闲了，该线程就会试着轮询它。实践上，`spawn` 会比 `spawn_local` 用得多。



使用 `spawn` 时，线程池倾向于保持忙碌。因此无论哪个线程率先得到轮询的机会，都会轮询到你的`Future`。异步调用可能在一个线程上开始执行，阻塞在 `await` 表达式，然后在另一个线程中恢复，即实际上可能会有多个不同的线程来承载此次代码的执行，所以要注意线程安全的问题。



### 2.5.4 独立线程运行任务：spawn_blocking

在异步编程中，通常要避免 `CPU` 密集型任务阻塞当前线程的操作，因为它们会阻塞整个异步执行器，导致其他异步任务无法进展。此时可以使用 `async_std::task::spawn_blocking` ，它允许你将这些阻塞性操作转移到一个新的独立线程中执行，从而保持异步执行器的响应性。



`async_std::task::spawn_blocking` 函数用于在异步环境中启动一个阻塞性任务，该函数会接受一个闭包，它会创建一个新的线程，并在这个新线程中执行你提供的闭包。这个闭包中的代码可以是同步的，因为它不会运行在异步执行器的线程上。与`async_std::task::spawn` 函数不同，`spawn_blocking` 函数不会立即返回一个 `JoinHandle`，而是在阻塞任务直到完成时才返回。



例子：在异步函数中使用 `argonautica` 库来根据哈希值检查用户的密码

```toml
argonautica = "0.2.0"
```

```rust
// 如果 password 和 hash 匹配，则返回 Ok(true)
async fn verify_password(
    password: &str,
    hash: &str,
    key: &str,
) -> Result<bool, argonautica::Error> {
    // 生成副本，以使闭包的生命周期是 'static
    let password = password.to_string();
    let hash = hash.to_string();
    let key = key.to_string();

    // 这样闭包里的计算会在各自的线程执行，不会影响其他用户请求的响应
    async_std::task::spawn_blocking(move || {
        argonautica::Verifier::dafault()
            .with_hash(hash)
            .with_password(password)
            .with_secret_key(key)
            .verify()
    })
    .await
}
```

注意：过度使用  `spawn_blocking` 可能会导致性能问题，因为它涉及到线程创建和同步状态的开销



## 2.6 手动实现 block_on

下面编写自己的 `block_on` 版本，完整	代码可看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/async-std-block-on.rs)

> `Cargo.toml` 先添加一下包

```toml
[dependencies]
waker-fn = "1.1.1"
futures-lite = "2.3.0"
crossbeam = "0.8.4"
async-std = "1.12.0"
```

```rust
use waker_fn::waker_fn;      
use futures_lite::pin;      
use crossbeam::sync::Parker; 
use std::future::Future;
use std::task::{Context, Poll};

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


fn main() {
    assert_eq!(block_on(std::future::ready(42)), 42);

    use async_std::task::{spawn, sleep};
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
        44);
}
```



## 2.7 手动实现 spawn_blocking

`spawn_blocking` 上面有讲过了，这里简单实现以下，我们的版本为每个闭包创建一个新线程，而不是像`async_std` 的版本使用线程池，完整代码可见[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/async-std-spawn-blocking.rs)

```rust
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
where F: FnOnce() -> T,
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
            let value = closure();     // 运行闭包的线程会将其返回值保存在value中，然后调用waker

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
```



## 2.8 使用异步注意事项

### 2.8.1 处理计算密集型任务

要避免在异步任务中处理大量计算密集型的任务，因为效率不高，且还容易饿死其它任务，`CPU` 密集型任务更适合使用线程，而非 `Future`。因为 `Future` 的调度是协作式多任务，除非 `Future` 主动放弃 `CPU`，不然它就会一直被执行，直到运行结束。如下例子：

> 完整代码看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/block-async-task.rs)

```rust
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
```

这段代码的 `task 2` 会没有机会执行到，因为` task 1` 有一个死循环，`task 1` 不执行结束（不让出 `CPU`），`task 2` 就没有机会被调度。

> 如果真的需要在 `tokio`（或者其它异步运行时）下运行计算密集型的代码，那么最好使用 `yield_now` 来主动让出` CPU`，将线程交还给调度器，自己则进入就绪队列等待下一轮的调度，比如 [tokio::task::yield_now()](https://docs.rs/tokio/1.13.0/tokio/task/fn.yield_now.html)，这样可以避免某个计算密集型的任务饿死其它任务



为了解决饿死其它任务的问题，可以同时使用线程和异步任务，把计算密集型任务放在线程中执行，`IO` 密集型任务放在异步运行时中执行，例如（ `tokio`），并使用 ` channel` 在线程和 `future` 之间做消息的同步。如下例子：

> 完整代码可看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/pow.rs)

```rust
use std::thread;

use anyhow::Result;
use blake3::Hasher;
use futures::{SinkExt, StreamExt};
use rayon::prelude::*;
use tokio::{
    net::TcpListener,
    sync::{mpsc, oneshot},
};
use tokio_util::codec::{Framed, LinesCodec};

pub const PREFIX_ZERO: &[u8] = &[0, 0, 0];

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    println!("listen to: {}", addr);

    // 创建 tokio task 和 thread 之间的 channel
    let (sender, mut receiver) = mpsc::unbounded_channel::<(String, oneshot::Sender<String>)>();

    // 使用 thread 处理计算密集型任务
    thread::spawn(move || {
        // 读取从 tokio task 过来的 msg，注意这里用的是 blocking_recv，而非 await
        while let Some((line, reply)) = receiver.blocking_recv() {
            // 计算 pow
            let result = match pow(&line) {
                Some((hash, nonce)) => format!("hash: {}, once: {}", hash, nonce),
                None => "Not found".to_string(),
            };
            // 把计算结果从 oneshot channel 里发回
            if let Err(e) = reply.send(result) {
                println!("Failed to send: {}", e);
            }
        }
    });

    // 使用 tokio task 处理 IO 密集型任务
    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted: {:?}", addr);
        let sender1 = sender.clone();
        tokio::spawn(async move {
            // 使用 LinesCodec 把 TCP 数据切成一行行字符串处理
            let framed = Framed::new(stream, LinesCodec::new());
            // split 成 writer 和 reader
            let (mut w, mut r) = framed.split();
            for line in r.next().await {
                // 为每个消息创建一个 oneshot channel，用于发送回复
                let (reply, reply_receiver) = oneshot::channel();
                sender1.send((line?, reply))?;

                // 接收 pow 计算完成后的 hash 和 nonce
                if let Ok(v) = reply_receiver.await {
                    w.send(format!("Pow calculated: {}", v)).await?;
                }
            }
            Ok::<_, anyhow::Error>(())
        });
    }
}

// 使用 rayon 并发计算 u32 空间下所有 nonce，直到找到有头 N 个 0 的哈希
pub fn pow(s: &str) -> Option<(String, u32)> {
    let hasher = blake3_base_hash(s.as_bytes());
    let nonce = (0..u32::MAX).into_par_iter().find_any(|n| {
        let hash = blake3_hash(hasher.clone(), n).as_bytes().to_vec();
        &hash[..PREFIX_ZERO.len()] == PREFIX_ZERO
    });
    nonce.map(|n| {
        let hash = blake3_hash(hasher, &n).to_hex().to_string();
        (hash, n)
    })
}

// 计算携带 nonce 后的哈希
fn blake3_hash(mut hasher: blake3::Hasher, nonce: &u32) -> blake3::Hash {
    hasher.update(&nonce.to_be_bytes()[..]);
    hasher.finalize()
}

// 计算数据的哈希
fn blake3_base_hash(data: &[u8]) -> Hasher {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher
}
```

这是一个 `TCP server` 例子，客户端输入过来的一行文字，会被计算出一个 `POW（Proof of Work`）的哈希：调整 `nonce`，不断计算哈希，直到哈希的头三个字节全是零为止。服务器要返回计算好的哈希和获得该哈希的 `nonce`。这是一个典型的`计算密集型任务`，所以使用线程来处理。



而在 `tokio task` 和 `thread` 间使用 `channel` 进行同步。这里使用了一个 `ubounded MPSC channel` 从 `tokio task` 侧往  `thread` 侧发送消息，每条消息都附带一个 `oneshot channel` 用于  `thread` 侧往 `tokio task` 侧发送数据。

> `MPSC`：`Multi-Producer Single-Consumer`，多生产者，单消费者



用 `telnet` 连接进行测试，发送 `“hello world!”`，会得到不同的哈希和 `nonce`

```rust
❯ telnet localhost 8080
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
hello world!
Pow calculated: hash: 0000006e6e9370d0f60f06bdc288efafa203fd99b9af0480d040b2cc89c44df0, once: 403407307
Connection closed by foreign host.

❯ telnet localhost 8080
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
hello world!
Pow calculated: hash: 000000e23f0e9b7aeba9060a17ac676f3341284800a2db843e2f0e85f77f52dd, once: 36169623
Connection closed by foreign host.
```



### 2.8.2 异步代码中要使用 Mutex

在使用 `Mutex` 等同步原语时，要注意标准库的 `MutexGuard` 无法跨越 `.await`，所以，此时要使用对异步友好的 `Mutex`，如 `tokio::sync::Mutex`。如下例子

> 完整代码看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/tokio-mutex.rs)

```rust
use anyhow::Result;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

struct DB;

impl DB {
    // 假装在 commit 数据
    async fn commit(&mut self) -> Result<usize> {
        Ok(42)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let db1 = Arc::new(Mutex::new(DB));
    let db2 = Arc::clone(&db1);

    tokio::spawn(async move {
        let mut db = db1.lock().await; // 拿到锁
      
        // 因为拿到的 MutexGuard 要跨越 await，所以不能用 std::sync::Mutex
        // 只能用 tokio::sync::Mutex
        let affected = db.commit().await?;
        println!("db1: Total affected rows: {}", affected);
      
        Ok::<_, anyhow::Error>(())
    });

    tokio::spawn(async move {
        let mut db = db2.lock().await;
        let affected = db.commit().await?;
        println!("db2: Total affected rows: {}", affected);

        Ok::<_, anyhow::Error>(())
    });

    // 让两个 task 有机会执行完
    tokio::time::sleep(Duration::from_millis(1)).await;

    Ok(())
}
```

这个例子模拟了一个数据库的异步 `commit()` 操作。如果我们需要在多个` tokio task` 中使用这个 `DB`，需要使用 `Arc<Mutext<DB>>`。然而，`db1.lock() `拿到锁后，需要运行 `db.commit().await`，这是一个异步操作

> 因为 `tokio` 实现了 `work-stealing` 调度，`Future` 有可能在不同的线程中执行，普通的 `MutexGuard` 编译直接就会出错，所以需要使用 [tokio 的 Mutex](https://docs.rs/tokio/1.13.0/tokio/sync/struct.Mutex.html)



# 3 Future 进阶

## 3.1 async 的生命周期

`async fn` 异步函数如果拥有引用类型的参数，那它返回的 `Future` 的生命周期就会被这些参数的生命周期所限制，如

```rust
async fn foo(x: &u8) -> u8 { *x }

// 上面的异步函数跟下面的这个用异步块实现的函数是等价的
fn foo<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
    async move { *x }
}
```

**此函数表明  `async fn` 异步函数返回的 `Future` 必须满足以下条件:**  当 `x` 依然有效时， 该 `Future` 就必须继续等待( `await` )，也就是说 `x` 必须比 `Future` 活得更久。



在一般情况下，在函数调用后就立即 `await` 不会存在任何问题，例如 `foo(&x).await`。但是，若 `Future` 被先存起来或发送到另一个任务或者线程，就可能存在问题。如下例子会报错，因为 `x`  的生命周期只到 `bad` 函数的结尾，而 `borrow_x` 返回的 `Future` 显然会活得更久，如

```rust
use std::future::Future;

fn bad() -> impl Future<Output = u8> {
    let x = 5;
    borrow_x(&x) // ERROR: `x` does not live long enough
}

async fn borrow_x(x: &u8) -> u8 { *x }
```

一个常用的解决方法：就是将具有引用参数的 `async fn` 函数转变成一个具有 `'static` 生命周期的 `Future` ，具体可以通过将参数和对 `async fn` 的调用放在同一个 `async` 异步块来实现。通过将参数移动到 `async` 语句块内， 将它的生命周期扩展到 `'static`， 并跟返回的 `Future` 保持了一致，代码如下

```rust
use std::future::Future;

async fn borrow_x(x: &u8) -> u8 { *x }

fn good() -> impl Future<Output = u8> {
    // 通过异步块去调用borrow_x
    async {
        let x = 5;
        borrow_x(&x).await
    }
}
```



## 3.2 符合 Send 的 Future

我们知道 `async_std::task::spawn` 里的 `Future` 可能会被发送到另一个线程运行，因此这个 `Future` 必须实现 `Send` 标记特型 。只有当 `Future` 包含的所有值都符合 `Send` 要求时，它自己才符合 `Send` 要求：所有函数参数、局部变量、甚至匿名临时值都必须安全地转移给另一个线程。



例如先看一个错误的例子

```rust
async fn reluctant() -> String {
    let string = std::rc::Rc::new("ref-counted string".to_string());

    test().await;

    // 这里用到了 string，此时可能是在另一个线程上执行
    format!("string: {}", string)
}

async fn test() {
    println!("test")
}

fn main() {
    async_std::task::spawn(reluctant());
}
```

报错如下

```bash
error: future cannot be sent between threads safely
   --> src/main.rs:411:28
    |
411 |     async_std::task::spawn(reluctant());
    |                            ^^^^^^^^^^^ future returned by `reluctant` is not `Send`
    |
    = help: within `impl Future<Output = String>`, the trait `std::marker::Send` is not implemented for `Rc<String>`
note: future is not `Send` as this value is used across an await
   --> src/main.rs:398:11
    |
396 |     let string = std::rc::Rc::new("ref-counted string".to_string());
    |         ------ has type `Rc<String>` which is not `Send`
397 |
398 |     test().await;
    |           ^^^^^^ await occurs here, with `string` maybe used later
...
401 | }
    | - `string` is later dropped here
note: required by a bound in `async_std::task::spawn`
   --> /Users/shuxinlin/.cargo/registry/src/mirrors.tuna.tsinghua.edu.cn-df7c3c540f42cdbd/async-std-1.12.0/src/task/spawn.rs:28:29
    |
28  |     F: Future<Output = T> + Send + 'static,
    |                             ^^^^ required by this bound in `async_std::task::spawn`

error: could not compile `hello` due to previous error
```

从报错信息可知

* `reluctant()` 不满足 `Send` 要求，因为 `Rc` 不满足 `Send`

  > `reluctant` 返回的 `Future` 在 `await` 之后使用 `string` 的值，所以 `Future` 会包含一个 `Rc<String>` 值。`Rc` 指针不能安全地在线程之间共享，所以 `Future` 本身也是不符合 `Send` 的
  
* `String` 影响了 `Future`，它的作用域跨越了 `await`



**有两种改正方式**

1、限制非 `Send` 值的作用域，使其不跨越任何 `await` 表达式的作用域，因此它们也不需要保存在函数的 `Future` 中

```rust
async fn reluctant() -> String {
    let result = {
        let string = std::rc::Rc::new("ref-counted string".to_string());
        format!("string: {}", string)
        // rc在这里离开了作用域
    };

    test().await;

    result
}

async fn test() {
    println!("test")
}

fn main() {
    async_std::task::spawn(reluctant());
}
```

2、另一种方法是简单的使用 `std::sync::Arc` 而非 `Rc`。`Arc` 使用原子更新来管理引用计数，`Arc` 指针符合`Send` 要求。

> 当然，如果 `Future` 不符合 `Send` 要求，也实在不容易把它改成符合 `Send` 要求，也是可以使用`async_std::task::spawn_local()` 在当前线程上运行它，且使用 `async_std::task::block_on` 阻塞它执行即可



## 3.3 async 生成的 Future 是什么类型

**impl Future 结构**

异步函数 `async fn` 的返回值是一个的 `impl Future<OutPut>` 的结构（异步块也一样）。如果给一个普通的函数返回 `impl Future`，它的行为和 `async fn` 是一致的。如下代码

> 完整代码看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/async-test1.rs)

```rust
use std::future::Future;

#[tokio::main]
async fn main() {
    let name1 = "zhangsan".to_string();
    let name2 = "lisi".to_string();
    
    // 用await执行 Feature
    say_hello1(&name1).await;
    say_hello2(&name2).await;
}

async fn say_hello1(name: &str) -> usize {
    println!("Hello {}", name);
    42
}

// async fn 关键字相当于一个返回 impl Future<Output> 的语法糖
fn say_hello2<'fut>(name: &'fut str) -> impl Future<Output = usize> + 'fut {
    async move {
        println!("Hello {}", name);
        42
    }
}
```

代码里的 `say_hello1` 和 `say_hello2` 是等价的，`say_hello1` 使用了 `async`，`say_hello2` 的返回值自己返回了 `Impl Future` 结构；以上代码是使用 `await` 来执行 `Future`，也可以将其提供给一个 `executor` 来执行，如下代码

> 完整代码看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/async-test2.rs)

```rust
use futures::executor::block_on;
use std::future::Future;

fn main() {
    let name1 = "zhangsan".to_string();
    let name2 = "lisi".to_string();

    // Future 还可以直接用 executor 执行
    block_on(say_hello1(&name1));
    block_on(say_hello2(&name2));
}

async fn say_hello1(name: &str) -> usize {
    println!("Hello {}", name);
    42
}

// async fn 关键字相当于一个返回 impl Future<Output> 的语法糖
fn say_hello2<'fut>(name: &'fut str) -> impl Future<Output = usize> + 'fut {
    async move {
        println!("Hello {}", name);
        42
    }
}
```



实际 `async fn` 异步函数或 `async block` 异步块的返回值 `impl Future<OutPut>` 结构不是一个具体的类型，它相当于 `T: Future`，那这个 `T` 是什么呢？先看下面的代码：

```rust
fn main() {
    let fut = async { 42 };

    println!("type of fut is: {}", get_type_name(&fut));
  
    println!("type of hello fut is: {}", get_type_name(&hello("Tyr")));
}

fn get_type_name<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

async fn hello(name: &str) -> String {
    format!("hello {}", name)
}
```

运行后输出如下：

```bash
type of fut is: core::future::from_generator::GenFuture<xxx::main::{{closure}}>
type of hello fut is: core::future::from_generator::GenFuture<xxx::hello::{{closure}}>
```

由此可见，`async` 代码块返回值是一个叫 `GenFuture` 的结构，它是一个实现了 `Future` 的 `generator`，它内部有一个闭包，这个闭包是 `async { 42 }` 产生的。简单看下 `GenFuture` 的定义（可以在 `Rust` 源码中搜 [from_generator](https://doc.bccnsoft.com/docs/rust-1.36.0-docs-html/src/std/future.rs.html#20-22)），它是一个泛型结构，内部数据 `T` 要满足 `Generator trait`：

```rust
struct GenFuture<T: Generator<ResumeTy, Yield = ()>>(T);

pub trait Generator<R = ()> {
    type Yield;
    type Return;
    fn resume(
        self: Pin<&mut Self>, 
        arg: R
    ) -> GeneratorState<Self::Yield, Self::Return>;
}
```

[Generator](https://doc.rust-lang.org/std/ops/trait.Generator.html) 是 `Rust nightly` 的一个 `trait`，还没有进入到标准库。看下官网展示的使用例子：

```rust
#![feature(generators, generator_trait)]

use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

fn main() {
    // 得到一个generator
    let mut generator = || {
        yield 1;
        return "foo"
    };

    match Pin::new(&mut generator).resume(()) {
        GeneratorState::Yielded(1) => {}
        _ => panic!("unexpected return from resume"),
    }
    match Pin::new(&mut generator).resume(()) {
        GeneratorState::Complete("foo") => {}
        _ => panic!("unexpected return from resume"),
    }
}
```

可以看到，如果你创建一个闭包，里面有 `yield` 关键字，就会得到一个 `Generator`。



## 3.4 异步块

`Rust` 除了异步函数，还支持异步块。可以在异步块中使用 `await` 表达式。普通的块语句会返回最后一个表达式的值，而异步块会返回最后一个表达式的 `Future`。例如

```rust
let serve_one = async { // 异步块
  use async_std::net;
  
  let listener = net::TcpListener::bind("localhost:8087").await?;
  let (mut socket, _addr) = listener.accept().await?;
  ...
  
};
```

* 代码会将 `serve_one` 初始化为一个 `Future`（当被轮询时），以监听并处理单个 `TCP` 连接
* 在异步块中使用 `?` 处理错误，它只会从块中而不是围绕它的函数中返回，块中的 `return` 也只会从块中返回，而不是函数返回




注意：`Rust` 无法判断异步块的返回类型是什么，这在使用 `?` 运算符时可能会导致问题，例如

```rust
let input = async_std::io::stdin();

let future = async {
    let mut line = String::new();
   
    // 这会返回 std::io::Result<usize>
    input.read_line(&mut line).await?;
  
    println!("read line : {}", line);
  
    Ok(())
};
```

以上代码运行会报错，`read_line` 会返回 `Result<(), std::io::Error>`，但是因为 `?` 运算符会使用 `From` 特型将手头的错误类型转换为场景要求的任何类型，所以异步块的返回类型 `Result<(), E>` 中的 `E` 可以是实现了 `From<std::io::Error>` 的任意类型。

解决方式：可以通过明确写出块的最终 `Ok` 的类型来解决这个问题，如

```rust
let future = async {
    ......
  
    Ok::<(), std::io::Error>(())
};
```



### 3.4.1 async move

如果异步块引用了围绕它的代码中定义的变量，那么它的 `Future` 就会捕获这些变量的值，就像闭包那样。与 `move` 闭包一样，也可以使用`async move` 启动该块以获取捕获的值的所有权，而不仅仅持有对它们的引用。`async move` 好处是不用解决借用生命周期的问题，坏处就是无法跟其它代码实现对变量的共享，例如

```rust
// 多个不同的 `async` 语句块可以访问同一个本地变量，只要它们在该变量的作用域内执行
async fn blocks() {
    let my_string = "foo".to_string();

    let future_one = async {
        // ...
        println!("{my_string}");
    };

    let future_two = async {
        // ...
        println!("{my_string}");
    };

    // 运行两个 Future 直到完成
    let ((), ()) = futures::join!(future_one, future_two);
}

// 由于`async move`会捕获环境中的变量，因此只有一个`async move`语句块可以访问该变量，
// 但是它也有非常明显的好处： 变量可以转移到返回的 Future 中，不再受借用生命周期的限制
fn move_block() -> impl Future<Output = ()> {
    let my_string = "foo".to_string();
  
    async move {
        // ...
        println!("{my_string}");
    }
}
```

上面调用 `many_requests` 调用 `cheapo_request` 时也可以写成异步块的形式，例如

```rust
pub async fn many_requests(requests: Vec<(String, u16, String)>) -> Vec<std::io::Result<String>> {
    use async_std::task;

    let mut handles = vec![];
    for (host, port, path) in requests {
        handles.push(task::spawn_local(async move { // 这是一个async move块
            cheapo_request(&host, port, &path).await // 这里调用 cheapo_request 可以使用引用
        }));
    }

    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    results
}
```

这是一个 `async move` 块，因为它的 `Future` 获取了 `String` 的值 `host` 和 `path` 的所有权，和 `move` 闭包一样。然后该 `Future` 会将引用传递给 `cheapo_request`。`Rust` 的借用检查器可以看到块的 `await` 表达式接手了 `cheapo_request` 返回的 `Future` 的所有权，因此对 `host` 和 `path` 的引用的生命周期不可能比它借来的已捕获变量的生命周期长。



**在同步函数中使用异步块**

可以在同步函数中使用异步块，实现跟异步函数一样的效果。例如为 `cheapo_request` 示例改写为同步函数，然后该函数返回一个异步块的 `Future`

```rust
fn cheapo_request_async_block<'a>(
    host: &'a str,
    port: u16,
    path: &'a str,
) -> impl Future<Output = std::io::Result<String>> + 'a { // 返回类型约束为 impl Future
    // 这里返回一个异步块
    async move {
        let mut socket = net::TcpStream::connect((host, port)).await?;

        let request = format!("GET {} HTTP/1.1 \r\nHost: {}\r\n\r\n", path, host);

        // 这里等待系统调用，会阻塞
        socket.write_all(request.as_bytes()).await?;

        socket.shutdown(net::Shutdown::Write)?;

        let mut response = String::new();

        // 这里等待系统调用，会阻塞
        socket.read_to_string(&mut response).await?;

        Ok(response)
    }
}
```

这个同步函数跟上面异步的 `cheapo_request` 效果是一样的



## 3.5 手写 Future 状态机代码

例如有一个`async` 函数：首先它创建一个文件，然后往文件里写入 `“hello world!”`。这个函数有两个 `await`，创建文件的时候会异步创建，写入文件的时候会异步写入。最终，整个函数对外返回一个 `Future`

```rust
async fn write_hello_file_async(name: &str) -> anyhow::Result<()> {
    let mut file = fs::File::create(name).await?;
  
    file.write_all(b"hello world!").await?;

    Ok(())
}
```

以上函数可以这样调用：

```rust
// 使用await的方式执行Future
write_hello_file_async("/tmp/hello").await?;
```

因为 `executor` 在处理 `Future` 时，会不断地调用它的 `poll()` 方法，于是上面的 `write_hello_file_async("/tmp/hello").await?` 调用实际上相当于：

```rust
match write_hello_file_async.poll(cx) {
    Poll::Ready(result) => return result,
    Poll::Pending => return Poll::Pending
}
```

再来看下 `write_hello_file_async` 函数内部的代码，其处理等价于以下代码：

```rust
let fut = fs::File::create(name);

match fut.poll(cx) {
    Poll::Ready(Ok(file)) => {
        
        // 只有在处理完 create()，才能处理 write_all()
        let fut = file.write_all(b"hello world!");
      
        match fut.poll(cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => return Poll::Pending,
        }
    }
    Poll::Pending => return Poll::Pending,
}
```

由于 `async` 函数返回的是一个 `Future`，所以，需要把这样的代码封装在一个 `Future` 的实现里，对外提供出去。因此需要实现一个数据结构，把内部的状态保存起来，并为这个数据结构实现 `Future`。比如：

```rust
enum WriteHelloFile {
    // 初始阶段，用户提供文件名
    Init(String),
  
    // 等待文件创建，此时需要保存 Future 以便多次调用
    // 这是伪代码，impl Future 不能用在这里
    AwaitingCreate(impl Future<Output = Result<fs::File, std::io::Error>>),
  
    // 等待文件写入，此时需要保存 Future 以便多次调用
    AwaitingWrite(impl Future<Output = Result<(), std::io::Error>>),

    // Future 处理完毕
    Done,
}

impl WriteHelloFile {
    pub fn new(name: impl Into<String>) -> Self {
        Self::Init(name.into())
    }
}

// 为这个数据结构实现Future
impl Future for WriteHelloFile {
    type Output = Result<(), std::io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}

fn write_hello_file_async(name: &str) -> WriteHelloFile {
    WriteHelloFile::new(name)
}
```

接着把 `write_hello_file_async` 异步函数，转化成了一个返回 `WriteHelloFile Future` 的函数。来看这个 `Future` 如何实现：

```rust
impl Future for WriteHelloFile {
    type Output = Result<(), std::io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
      
        loop {
            match this {
                // 如果状态是 Init，那么就生成 create Future，把状态切换到 AwaitingCreate
                WriteHelloFile::Init(name) => {
                    let fut = fs::File::create(name);
                    *self = WriteHelloFile::AwaitingCreate(fut);
                }
              
                // 如果状态是 AwaitingCreate，那么 poll create Future
                // 如果返回 Poll::Ready(Ok(_))，那么创建 write Future
                // 并把状态切换到 Awaiting
                WriteHelloFile::AwaitingCreate(fut) => match fut.poll(cx) {
                    Poll::Ready(Ok(file)) => {
                        let fut = file.write_all(b"hello world!");
                        *self = WriteHelloFile::AwaitingWrite(fut);
                    }
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Pending => return Poll::Pending,
                },
              
                // 如果状态是 AwaitingWrite，那么 poll write Future
                // 如果返回 Poll::Ready(_)，那么状态切换到 Done，整个 Future 执行成功
                WriteHelloFile::AwaitingWrite(fut) => match fut.poll(cx) {
                    Poll::Ready(result) => {
                        *self = WriteHelloFile::Done;
                        return Poll::Ready(result);
                    }
                    Poll::Pending => return Poll::Pending,
                },
              
                // 整个 Future 已经执行完毕
                WriteHelloFile::Done => return Poll::Ready(Ok(())),
            }
        }
    }
}
```

从上面可以看到，这个 `Future` 完整实现的内部结构 ，其实就是一个状态机的迁移。上面这段伪代码的功能和之前异步函数是等价的：

```rust
async fn write_hello_file_async(name: &str) -> anyhow::Result<()> {
    let mut file = fs::File::create(name).await?;
    file.write_all(b"hello world!").await?;

    Ok(())
}
```

`Rust` 在编译 `async fn` 或者 `async block` 时，就会生成类似的状态机的实现。



# 4 Pin 和 Unpin

## 4.1 Pin

`Pin` 的定义如下：

```rust
pub struct Pin<P> {
    pointer: P, // 注意 pointer 不是 pub 的
}

impl<P: Deref> Deref for Pin<P> {
    type Target = P::Target;
    fn deref(&self) -> &P::Target {
        Pin::get_ref(Pin::as_ref(self))
    }
}

impl<P: DerefMut<Target: Unpin>> DerefMut for Pin<P> {
    fn deref_mut(&mut self) -> &mut P::Target {
        Pin::get_mut(Pin::as_mut(self))
    }
}
```

* `Pin` 自身是一个智能指针，因为它实现了 `Deref` 和 `DerefMut`

* `Pin` 内部包裹了另一个指针 `P`，指针 `P` 指向的数据称为 `T`，`Pin` 能保证数据 `T` 永远不会被移动。简单理解，`Pin` 是用于防止值被移动的类型，确保它在内存中的位置不会改变，也称为固定一个值

  > 例外情况：如果 `P` 指针指向的数据 `T` 实现了 `Unpin`，则即使 `Pin` 住了，`T` 还是可以被移动，此时 `Pin` 了相当于没 `Pin`，比如这时 `Pin<Box<T>>` 就等价于 `Box<T>`

* `Pin` 包裹的内容只能是指针，所以对于 `Pin` 而言，一般都是 `Pin<Box<T>>`、`Pin<&mut T>` 等表示方式（不能是 `Pin<T>`，如 `Pin<i32>` 是错误的），它们是典型的固定指针



可以使用 `Pin::new` 从普通指针创建固定指针 `Pin`，然后使用 `Pin::into_inner` 取回该指针。`Pin` 本身会传递指针自己的 `Deref` 实现和 `DerefMut` 实现。例如

```rust
let mut string = "pinned?".to_string();
let mut pinned: Pin<&mut String> = Pin::new(&mut string);

pinned.push_str(" not");
Pin::into_inner(pinned).push_str(" so much.");

let new_home = string;
assert_eq!(new_home, "pinned? not so much.");
```

因为 `String` 是实现了 `UnPin` 的，即使在制作出 `Pin<&mut String>` 之后，仍然可以完全可变地址访问字符串，并且一旦这个 `Pin` 被 `into_inner` 消耗，可变引用消失后就可以将其转移给新变量。



## 4.2 Unpin

[Unpin](https://doc.rust-lang.org/std/marker/trait.Unpin.html) 是一个标记特型 `(marker trait)`，定义如下

```rust
pub auto trait Unpin {}
```

* `Pin` 是防止一个类型在内存中被移动，而 `Unpin` 相反，实现 `Unpin` 的数据结构是可以在内存中安全的移动的，它的作用类似于 `Send/Sync`，通过类型约束来告诉编译器哪些行为是合法的

* 因为 `Unpin` 是一个 `auto` 特型，编译器默认会给所有类型实现 `Unpin`，这些类型都是可以移动的。唯独有几个例外，它们不能被移动，它们实现的是 `!Unpin`，例如 `PhantomPinned` ，还有 `async` 生成的 `impl Future` 的结构体。`Pin` 只对实现 `!Unpin` 的类型才有钉住的效果。如果希望一个数据结构不能被移动，可以使用 `!Unpin`，可以为其添加 `PhantomPinned` 字段来隐式声明 `!Unpin`

* 注意：实现 `Unpin` 的数据即使被 `Pin` 钉住，这些数据 `T` 依旧可以被移动，此时 `Pin` 了的效果相当于没 `Pin`，比如这时 `Pin<Box<T>>` 就等价于 `Box<T>`



## 4.3 自引用结构的移动问题

**自引用结构**

在上面手动实现 `Future` 状态机的代码中，代码中引用了 `file` 这样一个局部变量，其实是有点问题的

```rust
WriteHelloFile::AwaitingCreate(fut) => match fut.poll(cx) {
    Poll::Ready(Ok(file)) => {
        let fut = file.write_all(b"hello world!");
        *self = WriteHelloFile::AwaitingWrite(fut);
    }
  
    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
    Poll::Pending => return Poll::Pending,
}
```

这里 `file` 被 `fut` 引用，但 `file` 会在这个作用域被丢弃。所以需要把它保存在数据结构中，可以用一个新的 `AwaitingWriteData` 数据结构来存放 `file` 和 `fut` ，然后在 `WriteHelloFile` 中引用它。如

```rust
enum WriteHelloFile {
    // 初始阶段，用户提供文件名
    Init(String),
    // 等待文件创建，此时需要保存 Future 以便多次调用
    AwaitingCreate(impl Future<Output = Result<fs::File, std::io::Error>>),
    // 等待文件写入，此时需要保存 Future 以便多次调用
    AwaitingWrite(AwaitingWriteData),
    // Future 处理完毕
    Done,
}

struct AwaitingWriteData {
    fut: impl Future<Output = Result<(), std::io::Error>>,
    file: fs::File,
}
```

此时，在同一个数据结构 `AwaitingWriteData` 内部，`fut` 指向了对 `file` 的引用，结构体内部某个成员是对另外一个成员的引用，这样的数据结构叫**自引用结构（Self-Referential Structs）。**



**自引用数据结构的移动问题**

自引用结构有一个很大的问题是：一旦它被移动，原本的指针就会指向旧的地址。所以需要有机制来保证这种情况不会发生，`Pin` 就可以解决这个问题。



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-05%2020.38.13.png)



下面看一个自引用数据结构移动的例子

> 完整代码可看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/self-ref.rs)

```rust
#[derive(Debug)]
struct SelfReference {
    name: String,
    // 在初始化后指向 name
    name_ptr: *const String,
}

impl SelfReference {
    pub fn new(name: impl Into<String>) -> Self {
        SelfReference {
            name: name.into(),
            name_ptr: std::ptr::null(),
        }
    }

    pub fn init(&mut self) {
        self.name_ptr = &self.name as *const String;
    }

    pub fn print_name(&self) {
        println!(
            "struct {:p}: (name: {:p} name_ptr: {:p}), name: {}, name_ref: {}",
            self,
            &self.name,
            self.name_ptr,
            self.name,
            // 在使用 ptr 是需要 unsafe
            // SAFETY: 这里 name_ptr 潜在不安全，会指向旧的位置
            unsafe { &*self.name_ptr },
        );
    }
}

fn main() {
    let data = move_creates_issue();
    println!("data: {:?}", data);
    // 如果把下面这句注释掉，程序运行会直接 segment error
    // data.print_name();
    print!("\\n");
    mem_swap_creates_issue();
}

fn move_creates_issue() -> SelfReference {
    let mut data = SelfReference::new("Tyr");
    data.init();

    // 不 move，一切正常
    data.print_name();

    let data = move_it(data);

    // move 之后，name_ref 指向的位置是已经失效的地址
    // 只不过现在 move 前的地址还没被回收挪作它用
    data.print_name();
    data
}

fn mem_swap_creates_issue() {
    let mut data1 = SelfReference::new("Tyr");
    data1.init();

    let mut data2 = SelfReference::new("Lindsey");
    data2.init();

    data1.print_name();
    data2.print_name();

    // 使用swap()函数交换两者，这里发生了move
    std::mem::swap(&mut data1, &mut data2);
  
    data1.print_name();
    data2.print_name();
}

fn move_it(data: SelfReference) -> SelfReference {
    data
}
```

* 代码创建了一个自引用结构 `SelfReference`，它里面的 `name_ref` 指向了 `name`。正常使用它时，没有任何问题，但一旦对这个结构做 `move` 操作，`name_ref` 指向的位置依然是 `move` 前 `name` 的地址，就会引发问题，如下图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-05%2020.44.28.png)



* 代码中 `std::mem:swap` 之后，两个数据的内容交换，这里会发生 `move` 操作，然而由于 `name_ref` 指向的地址还是旧的，所以整个指针体系就错乱了，如下图

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-05%2020.45.20.png)



代码输出如下，可以看到 `swap` 之后，`name_ref ` 指向的内容确实和 `name` 不一样了，这就是自引用结构带来的问题

```bash
struct 0x7ffeea91d6e8: (name: 0x7ffeea91d6e8 name_ptr: 0x7ffeea91d6e8), name: Tyr, name_ref: Tyr
struct 0x7ffeea91d760: (name: 0x7ffeea91d760 name_ptr: 0x7ffeea91d6e8), name: Tyr, name_ref: Tyr
data: SelfReference { name: "Tyr", name_ptr: 0x7ffeea91d6e8 }

struct 0x7ffeea91d6f0: (name: 0x7ffeea91d6f0 name_ptr: 0x7ffeea91d6f0), name: Tyr, name_ref: Tyr
struct 0x7ffeea91d710: (name: 0x7ffeea91d710 name_ptr: 0x7ffeea91d710), name: Lindsey, name_ref: Lindsey
struct 0x7ffeea91d6f0: (name: 0x7ffeea91d6f0 name_ptr: 0x7ffeea91d710), name: Lindsey, name_ref: Tyr
struct 0x7ffeea91d710: (name: 0x7ffeea91d710 name_ptr: 0x7ffeea91d6f0), name: Tyr, name_ref: Lindsey
```

这里第二行打印 `name_ref` 还是指向了 `“Tyr”`，因为 `move` 后，之前的内存失效，但是内存地址还没有被挪作它用，所以还能正常显示 `“Tyr”`。但这样的内存访问是不安全的，如果把 `main` 中这句代码注释掉，程序就会 `crash`：

```rust
fn main() {
    let data = move_creates_issue();
    println!("data: {:?}", data);
    // 如果把下面这句注释掉，程序运行会直接 segment error
    // data.print_name();
    print!("\\n");
    mem_swap_creates_issue();
}
```

所以，`Pin` 对解决这类自己引用数据结构的 `move` 问题很关键，如果试图移动被 `Pin` 住的数据结构，要不编译器会编译不通过；要不就是强行我们使用 `unsafe` 包裹，自己负责其安全性。针对以上问题，来看下使用 `Pin` 修正后的代码，如

> 完整代码可看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/self-ref-pin.rs)

```rust
use std::{marker::PhantomPinned, pin::Pin};

#[derive(Debug)]
struct SelfReference {
    name: String,
    // 在初始化后指向 name
    name_ptr: *const String,
    // PhantomPinned 占位符
    _marker: PhantomPinned,
}

impl SelfReference {
    pub fn new(name: impl Into<String>) -> Self {
        SelfReference {
            name: name.into(),
            name_ptr: std::ptr::null(),
            _marker: PhantomPinned,
        }
    }

    pub fn init(self: Pin<&mut Self>) {
        let name_ptr = &self.name as *const String;
        // SAFETY: 这里并不会把任何数据从 &mut SelfReference 中移走
        let this = unsafe { self.get_unchecked_mut() };
        this.name_ptr = name_ptr;
    }

    pub fn print_name(self: Pin<&Self>) {
        println!(
            "struct {:p}: (name: {:p} name_ptr: {:p}), name: {}, name_ref: {}",
            self,
            &self.name,
            self.name_ptr,
            self.name,
            // 在使用 ptr 是需要 unsafe
            // SAFETY: 因为数据不会移动，所以这里 name_ptr 是安全的
            unsafe { &*self.name_ptr },
        );
    }
}

fn main() {
    move_creates_issue();
}

fn move_creates_issue() {
    let mut data = SelfReference::new("Tyr");
    let mut data = unsafe { Pin::new_unchecked(&mut data) };
    SelfReference::init(data.as_mut());

    // 不 move，一切正常
    data.as_ref().print_name();

    // 现在只能拿到 pinned 后的数据，所以 move 不了之前
    move_pinned(data.as_mut());
    println!("{:?} ({:p})", data, &data);

    // 你无法拿回 Pin 之前的 SelfReference 结构，所以调用不了 move_it
    // move_it(data);
}

fn move_pinned(data: Pin<&mut SelfReference>) {
    println!("{:?} ({:p})", data, &data);
}

#[allow(dead_code)]
fn move_it(data: SelfReference) {
    println!("{:?} ({:p})", data, &data);
}
```

由于数据结构被包裹在 `Pin` 内部，所以在函数间传递时，变化的只是指向 `data` 的 `Pin`，避免了移动带来的问题，如下图

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230106000101.png)



## 4.4 Future 的移动问题

**Future 中 poll 函数的 Pin**

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

`Future` 的定义中，`poll` 函数的 `self` 是 `Pin<&mut Self>` 类型。`Pin` 对于 `Future` 很重要，它是为了解决`Future` 内部自引用的问题。



因为 `async/await` 就是通过 `Generator` 实现的，`Generator` 是通过匿名结构体实现的。如果 `async` 函数中存在跨 `await` 的引用，会导致底层 `Generator` 存在跨 `yield` 的引用，那根据 `Generator` 生成的匿名结构体就会是一个自引用结构体。然后这个自引用结构体会 `impl Future`，异步的运行时在调用 `Future::poll()` 函数查询状态时，需要一个可变借用（即 `&mut Self`）。如果这个 `&mut Self` 不包裹在 `Pin` 里面的话，开发者可能自己`impl Future` 就会利用 `std::mem::swap()` 之类的函数 `move` 掉 `&mut Self`。



**Future 的移动问题**

先来分析一下下面例子

```rust
use async_std::io::prelude::*;
use async_std::{io, net};

// 打开给定地址的TCP连接，并返回服务器发送的任何内容（String形式）
async fn fetch_string(address: &str) -> io::Result<String> {
    // 恢复点1: 函数体刚开始执行时
  
    let mut socket = net::TcpStream::connect(address).await?; // 恢复点2，await是恢复点

    let mut buf = String::new();

    socket.read_to_string(&mut buf).await?; // 恢复点3，执行应该在await处恢复

    Ok(buf)
}
```

由代码可知，代码中有 3 处恢复点，恢复点即异步函数代码中可以暂停执行的点

1. 第1处是函数体开始执行时
2. 第2处是 `net::TcpStream::connect(address).await` 的 `await`
3. 第3处是 `socket.read_to_string(&mut buf).await` 的 `await`

注意：对 `read_to_string` 的调用借用了对 `socket` 和 `buf` 的引用。在同步函数中，所有局部变量都存在于栈中，但在异步函数中，在 `await` 中仍然存活的局部变量必须位于 `Future` 中，这样当再次轮询时它们才是可用的。借入对这样一个变量的引用，就是借入了 `Future` 中的一部分。



假如这样调用它，此时没有用 `await` 等待，`response` 是一个 `Future`，由于刚创建这个 `response Future`，因此它认为执行应该从 `fetch_string` 函数体顶部的第一处恢复点开始，在这种状态下，`response Future` 存储了函数传进来的参数值，比如这里是 `address` 参数

```rust
let response = fetch_string("localhost:8080");
```

假设对 `response Future` 进行了几次轮询，且它在函数体中执行到了第 3 处恢复点，如果 `read_to_string` 的结果还没完成，此时轮询会返回 `Poll::Pending`，此时 `response Future` 会保存下一次轮询时恢复执行需要的所有信息。例如在恢复点 3 时，表示执行应该在 `await` 处恢复，那时正在轮询 `read_to_string` 返回的子 `Future`

* 此时处于活跃状态的变量是 `socket` 和 `buf`
* `address` 的值在 `Future` 中不会再出现，因为该函数已经不需要它了

```rust
socket.read_to_string(&mut buf).await?;
```

因为 `Rust` 要求已经借出的值不能再移动，假设我们把 `reponse` 这个 `Future` 移动了，如

```rust
let response = fetch_string("localhost:8080");

let new_var = response; // 这里是错误的用法，response被借出了
```

此时会报错，因为 `Rust` 的借用检查器无法找出 `response` 所有还需要用到的引用并调整它们，此时引用不会指向新位置的 `socket` 和 `buf`，而是继续执行它们在当前处于未初始状态的 `response` 中的旧位置，`socket` 和 `buf` 变成了悬空指针。



如果 `Future` 本身已移动，则存储在 `Future` 中的变量也会移动，这意味着 `socket` 和 `buf` 的借用不仅会影响 `fetch_string` 可以用自己的变量做什么，还会影响其调用者可以安全地用 `response` （也就是持有这些变量的 `Future`）做什么。异步函数的 `Future` 是借用检查器的盲点，`Rust` 为了保证其内存安全提供了一个解决方法：

* `Future` 在首次创建时总是可以安全的移动，只有在轮询时才会变得不安全
* 在一开始，通过异步函数创建的 `Future` 仅包含一个恢复点和参数值，这些仅仅存在于尚未开始执行的异步函数主体的作用域内，只有当轮询 `Future` 时才会借用其内容

因此，每一个 `Future` 的生命周期都有两个阶段

* 第一阶段从刚创建 `Future` 时开始。因为函数体还没开始执行，所以它的任何部分都不可能被借用。在这一点，移动它和移动其他 `Rust` 值一样是安全的

  > 第一阶段的灵活性让我们能够将 `Future` 传给 `block_on` 和 `spawn` 并调用适配器方法（如 `race` 和 `fuse`），所有这些都会按值获取`Future`，事实上，即使最初创建 `Future` 的那次异步函数调用也必须将其返回给调用者，那同样是一次移动。

* 第二阶段在第一次轮询 `Future` 时开始，一旦函数的主体开始执行，它就可以借用对存储在 `Future` 中的变量的引用，然后等待，保留对 `Future` 持有的变量的借用。从第一次轮询开始，就必须假设  `Future` 不能被安全的移动了

  > 要进入第二阶段，就必须对 `Future` 进行轮询。`poll` 方法要求将 `Future` 作为 `Pin<&mut self>` 值传递。`Pin` 是指针类型（如 `&mut Self`）的包装器，它限制了指针的使用方式，以确保它们的引用目标（如 `Self` ）永远不会再次移动。因此，必须首先生成一个指向`Future` 的以 `Pin` 包装的指针，然后才能对其进行轮询。

这就是 `Rust` 确保内存安全的策略：`Future` 只有在轮询之前移动才不会有危险， `Future` 一旦被轮询就不能移动。这种问题仅限于异步函数和异步块的 `Future`，以及编译器为它们生成的特殊 `Future` 实现。因为它们有能力在函数调用过程中暂停执行并仍持有借用，所以才要小心处理它们的 `Future`。



给定一个异步函数或异步块返回的 `Future`，有以下几种方法可以获得指向它的固定指针

* `pin!` 宏：它来自 `futures-lite crate`，它会用新的 `Pin<&mut T>` 类型的变量遮蔽 `T` 类型的变量。新变量会指向原始值，而原始值已移至栈中的匿名临时位置。当新变量超出作用域时，原始值会被丢弃

  > 例如上面自己实现的 `block_on` 例子，用 `pin!` 宏在 `block_on` 实现中固定了想要轮询的 `Future`

* 标准库的 `Box::pin` 构造函数，它能获取任意类型 `T` 值的所有权，将其移动到堆中，并返回 ` Pin<Box<T>>`

* `Pin<Box<T>>` 可以实现 `From<Box<T>>`，因此 `Pin::from(boxed)` 会取得 `boxed` 的所有权，并返回指向堆上同一个 `T` 的固定过的 `Box`

注意：获取指向这些 `Future` 的固定指针的每一种方法，都需要放弃对 `Future` 的所有权，并且无法再取回。



一旦固定了 `Future`，如果想轮询它，那么所有  `Pin<pointer to T>` 类型都会有一个 `as_mut` 方法，该方法会解引用指针并返回 `poll` 所需的 `Pin<&mut T>`。`as_mut` 方法还可以再不放弃所有权的情况下轮询 `Future`，例如上面 `block_on` 的实现就是这样处理的

```rust
 // 这里用到了 pin 宏
    pin!(future);

    loop {
        // as_mut方法会解引用指针
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => parker.park(), 
        }
    }
```

这里 `pin！` 宏已经将 `future` 重新声明为 `Pin<&mut F>`，因此可以将其传给 `poll`。但是可变引用不是 `Copy` 类型，因此 `Pin<&mut F>` 也不是 `Copy` 类型，意味着直接调用 `future.poll()` 将取得 `future` 的所有权，进而导致循环的下一次迭代留下未初始化的变量。为了避免这种情况，这里调用 `future.as_mut()` 为每次循环迭代重新借入新的 `Pin<&mut F>`



## 4.5 Pin 的应用

`Pin` 可以分为栈上还是堆上，取决于你要 `Pin` 的那个指针 `P` 是在栈上还是堆上。比如 `Pin<&mut T>` 是栈上，`Pin<Box<T>>` 是在堆上。



### 4.5.1 将值固定到栈上

可以用 `Pin` 来解决指针指向的数据被移动的问题，例如

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

#[derive(Debug)]
struct Test {
    a: String,
    b: *const String,
    _marker: PhantomPinned,
}

impl Test {
    fn new(txt: &str) -> Self {
        Test {
            a: String::from(txt),
            b: std::ptr::null(),
            _marker: PhantomPinned, // 这个标记可以让我们的类型自动实现特征 !Unpin
        }
    }

    fn init(self: Pin<&mut Self>) {
        let self_ptr: *const String = &self.a;
        let this = unsafe { self.get_unchecked_mut() };
        this.b = self_ptr;
    }

    fn a(self: Pin<&Self>) -> &str {
        &self.get_ref().a
    }

    fn b(self: Pin<&Self>) -> &String {
        assert!(!self.b.is_null(), "Test::b called without Test::init being called first");
        unsafe { &*(self.b) }
    }
}


```

* 代码中使用了一个标记类型 `PhantomPinned` 将自定义结构体 `Test` 变成了 `!Unpin` (编译器会自动帮我们实现)，因此该结构体无法再被移动

* 一旦类型实现了 `!Unpin` ，那将它的值固定到栈上就是不安全的行为，因此在代码中使用了 `unsafe` 语句块来进行处理，这里也可以使用 [`pin_utils`](https://docs.rs/pin-utils/) 来避免 `unsafe` 的使用。此时如果去尝试移动被固定的值，就会导致编译错误，比如下面这样的调用方式 

```rust
pub fn main() {
    // 此时的test1可以被安全的移动
    let mut test1 = Test::new("test1");
  
    // 新的test1由于使用了Pin，因此无法再被移动，这里的声明会将之前的test1遮蔽掉
    let mut test1 = unsafe { Pin::new_unchecked(&mut test1) }; // 这里用 Pin::new_unchecked创建 Pin
    Test::init(test1.as_mut());

    let mut test2 = Test::new("test2");
    let mut test2 = unsafe { Pin::new_unchecked(&mut test2) };
    Test::init(test2.as_mut());

    println!("a: {}, b: {}", Test::a(test1.as_ref()), Test::b(test1.as_ref()));
  
    std::mem::swap(test1.get_mut(), test2.get_mut());
  
    println!("a: {}, b: {}", Test::a(test2.as_ref()), Test::b(test2.as_ref()));
}
```

代码中尝试把 `&mut Test` 钉在栈上，然后尝试去调用 `get_mut()` 作为参数传给 `std::mem::swap()`，此时编译不通过，`Rust` 编译器从编译阶段就阻止我们去犯错。



### 4.5.2 将值固定到堆上

将一个 `!Unpin` 类型的值固定到堆上，会给予该值一个稳定的内存地址，它指向的堆中的值在 `Pin` 后是无法被移动的。而且与固定在栈上不同，堆上的值在整个生命周期内都会被稳稳地固定住。例如

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

#[derive(Debug)]
struct Test {
    a: String,
    b: *const String,
    _marker: PhantomPinned,
}

impl Test {
    fn new(txt: &str) -> Pin<Box<Self>> {
        let t = Test {
            a: String::from(txt),
            b: std::ptr::null(),
            _marker: PhantomPinned,
        };
      
        let mut boxed = Box::pin(t);
        let self_ptr: *const String = &boxed.as_ref().a;
        unsafe { boxed.as_mut().get_unchecked_mut().b = self_ptr };

        boxed
    }

    fn a(self: Pin<&Self>) -> &str {
        &self.get_ref().a
    }

    fn b(self: Pin<&Self>) -> &String {
        unsafe { &*(self.b) }
    }
}

pub fn main() {
    let test1 = Test::new("test1");
    let test2 = Test::new("test2");

    println!("a: {}, b: {}",test1.as_ref().a(), test1.as_ref().b());
  
    // std::mem::swap(test1.get_mut(), test2.get_mut());
    // std::mem::swap(&mut *test1, &mut *test2);
  
    println!("a: {}, b: {}",test2.as_ref().a(), test2.as_ref().b());
}
```

代码使用 `Box::pin()` 把 `Test` 钉在了堆上，取消注释中的任意一行都会编译不通过，因为 `Test` 是  `!Unpin`的。



### 4.5.3 将固定住的 Future 变为 Unpin

`async` 函数返回的 `Future` 默认就是 `!Unpin` 的，即不可被移动的。在实际应用中，一些函数会要求它们处理的 `Future` 是 `Unpin` 的，即可移动的，此时必须要使用以下的方法先将 `Future` 进行固定:

- `Box::pin`：创建一个 `Pin<Box<T>>`
- `pin_utils::pin_mut!`： 创建一个 `Pin<&mut T>`

固定后获得的 `Pin<Box<T>>` 和 `Pin<&mut T>` 既可以用于 `Future` ，又会自动实现 `Unpin`。

```rust
use pin_utils::pin_mut;

// 函数的参数是一个 Future，但是要求该 Future 实现 Unpin
fn execute_unpin_future(x: impl Future<Output = ()> + Unpin) { /* ... */ }

let fut = async { /* ... */ };
// 下面代码报错: 默认情况下，fut 实现的是 !Unpin，并没有实现 Unpin
// execute_unpin_future(fut);

// 方式1:使用 Box 进行固定
let fut = async { /* ... */ };
let fut = Box::pin(fut);
execute_unpin_future(fut); // OK

// 方式2:使用 pin_mut! 进行固定
let fut = async { /* ... */ };
pin_mut!(fut);
execute_unpin_future(fut); // OK
```



# 5 参考

* [陈天 · Rust 编程第一课-异步处理](https://time.geekbang.org/column/article/455413)
* [async/await异步编程](https://course.rs/advance/async/intro.html)
* [Rust程序设计（第2版）](https://book.douban.com/subject/36547630/)
* [Rust 的 Pin 与 Unpin](https://folyd.com/blog/rust-pin-unpin/)

