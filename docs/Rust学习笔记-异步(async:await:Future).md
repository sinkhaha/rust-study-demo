# 1 async/await 和 Future

async/await 是 Rust 的异步编程模型，是产生和运行并发任务的手段。



一般而言，async 定义了一个可以并发执行的任务，而 await 则触发这个任务并发执行。Rust 中，async 用来创建 Future，await 来触发 Future 的调度和执行，并等待Future执行完毕。async/await 只是一个语法糖，它使用状态机将 Future 包装起来进行处理。



JavaScript 也是通过 async 的方式提供了异步编程，Rust 的 Future 跟 JavaScript 的 Promise 非常类似。它们的区别：

* JavaScript 的 Promise 和线程类似，一旦创建就开始执行，对 Promise 的 await 只是等待这个Promise执行完成并得到结果
* Rust 的 Future，只有在主动 await 后才开始执行



## 1.1 同步/多线程/异步例子

下面分别用同步的方式、多线程的方式、异步的方式，实现读写文件的需求：读取 Cargo.toml 和 Cargo.lock 并将它们转换成 yaml 写入 /tmp 文件夹下

**1、使用同步的方式实现**

> 完整代码可看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/sync-io.rs)

```rust
use anyhow::Result;
use serde_yaml::Value;
use std::fs;

fn main() -> Result<()> {
    // 读取 Cargo.toml，IO 操作 1
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

缺点：因为是同步读取，在读 Cargo.toml 时，整个主线程被阻塞，直到 Cargo.toml 读完，才能继续读 Cargo.lock 文件，读取两个文件的总共等待时间是 time_for_file1 + time_for_file2。整个主线程，只有在运行 toml2yaml 的时间片内，才真正在执行计算任务，读取文件以及写入文件等这些IO操作，CPU 都在闲置；后面的写入文件也有类似问题



**2、使用多线程的方式实现**

此方式把文件读取和写入操作放入单独的线程中执行

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

* 优点：读取两个文件是并发执行（写入也类似），大大缩短等待时间，读取的总共等待的时间是 max(time_for_file1, time_for_file2)

* 缺点：不适用于同时读太多文件的场景；因为每读一个文件会创建一个线程，在操作系统中，线程的数量是有限的，创建过多的线程会大大增加系统的开销

  > 大多数操作系统对 I/O 操作提供了非阻塞接口，Rust 可以利用 async/await 异步处理，进而最大程度的利用 CPU 资源

  

**3、使用 async/await 异步实现**

> 完整代码可看[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/async-and-await/examples/async-io.rs)

```rust
use anyhow::Result;
use serde_yaml::Value;
use tokio::{fs, try_join};

#[tokio::main]
async fn main() -> Result<()> {
    let f1 = fs::read_to_string("./Cargo.toml");
    let f2 = fs::read_to_string("./Cargo.lock");
  
    // 等待两个异步io操作完成
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

fn toml2yaml(content: &str) -> Result<String> {
    let value: Value = toml::from_str(&content)?;
    Ok(serde_yaml::to_string(&value)?)
}
```

这里使用了` tokio::fs`，而不是 `std::fs`，`tokio::fs` 的文件操作都会返回一个 Future，然后用 try_ join 轮询这些Future，得到它们运行后的结果。此时文件读取的总时间是 max(time_for_file1, time_for_file2)，性能和使用线程的版本几乎一致，但是消耗的线程资源要少很多。



**try_join和join宏的作用：**是用来轮询多个 Future ，它会依次处理每个 Future，遇到阻塞就处理下一个，直到所有 Future 产生结果（类似JavaScript的Promise.all）。



注意代码不能写成以下方式：

```rust
// 读取 Cargo.toml，IO 操作 1
let content1 = fs::read_to_string("./Cargo.toml").await?;
// 读取 Cargo.lock，IO 操作 2
let content1 = fs::read_to_string("./Cargo.lock").await?;
```

因为 `.await ` 会运行 Future 一直到 该Future 执行结束，所以此写法依旧是先读取 Cargo.toml，再读取 Cargo.lock，并没有达到并发的效果，这样和同步的版本没有区别。



**.await 的作用：**在 async fn 函数中使用`.await`可以等待另一个异步调用的完成，使用同步的方式实现了异步的执行效果。`.await` 不会阻塞当前的线程，而是异步的等待 Future A的完成，在等待的过程中，该线程还可以继续执行 Future B，最终实现了并发处理的效果。



## 1.2 Future 定义

Future 是 Rust 异步编程的核心， [Future](https://doc.rust-lang.org/std/future/trait.Future.html) trait的定义：

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

Future 有一个关联类型 Output；还有一个 poll() 方法，它返回 `Poll<Self::Output>`。Poll 是个枚举，有 Ready 和 Pending 两个状态。通过调用 poll() 方法可以推进 Future 的进一步执行，直到被切走为止

> 在当前 poll 中，若 Future 完成了，则返回 `Poll::Ready(result) `，即得到 Future 的值并返回；若Future 还没完成，则返回 `Poll::Pending()`，此时 Future 会被挂起，需要等某个事件将其唤醒（wake唤醒函数）



## 1.3 executor 调度器

executor 是一个 Future 的调度器。操作系统负责调度线程，但它不会去调度用户态的协程（比如 Future），所以任何使用了协程来处理并发的程序，都需要有一个 executor 来负责协程的调度。



Rust 的 Future 是惰性的：只有在被 poll 轮询时才会运行。其中一个推动它的方式就是在 async 函数中使用 `.await` 来调用另一个 async 函数，但是这个只能解决 async 内部的问题，那些最外层的 async 函数，需要靠执行器 executor 来推动 。



**Rust中的executor**

Rust 虽然提供 Future 这样的协程，但它在语言层面并不提供 executor，当不需要使用协程时，不需要引入任何运行时；而需要使用协程时，可以在生态系统中选择最合适的 executor。

> Golang也支持协程，但在语言层面自带了一个用户态的调度器

 Rust 有如下4中常见的 executor ：

* [futures](https://github.com/rust-lang/futures-rs)：这个库自带了很简单的 executor
* [tokio](https://github.com/tokio-rs/tokio)：提供 executor，当使用 #[tokio::main] 时，就隐含引入了 tokio 的 executor
* [async-std](https://github.com/async-rs/async-std) ：提供 executor，和 tokio 类似
* [smol](https://github.com/smol-rs/smol) ：提供 async-executor，主要提供了 block_on



**wake通知机制**

executor 会管理一批 Future (最外层的 async 函数)，然后通过不停地 poll 推动它们直到完成。 最开始，执行器会先 poll 一次 Future ，后面就不会主动去 poll 了，如果 poll 方法返回 `Poll::Pending`，就挂起 Future，直到收到某个事件后，通过 wake()函数去唤醒被挂起 Future，Future 就可以去主动通知执行器，它才会继续去 poll，执行器就可以执行该 Future。这种 wake 通知然后 poll 的方式会不断重复，直到 Future 完成。



Waker 提供了 wake() 方法：其作用是可以告诉执行器，相关的任务可以被唤醒了，此时执行器就可以对相应的 Future 再次进行 poll 操作。



Context 是 Waker 的一个封装，先看下 poll 方法里的 Context：

```rust
pub struct Context<'a> {
    waker: &'a Waker,
    _marker: PhantomData<fn(&'a ()) -> &'a ()>,
}
```

Waker 的定义和相关的代码非常抽象，内部使用了一个 vtable 来允许各种各样的 waker 的行为：

```rust
pub struct RawWakerVTable {
    clone: unsafe fn(*const ()) -> RawWaker,
    wake: unsafe fn(*const ()),
    wake_by_ref: unsafe fn(*const ()),
    drop: unsafe fn(*const ()),
}
```

Rust 自身不提供异步运行时，它只在标准库里规定了一些基本的接口，可以由各个运行时自行决定怎么实现。所以在标准库中，只能看到这些接口的定义，以及“高层”接口的实现，比如 Waker 下的 wake 方法，只是调用了 vtable 里的 wake() 而已

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

vtable具体的实现并不在标准库中，而是在第三方的异步运行时里，比如futures 库的 waker vtable [定义](https://github.com/rust-lang/futures-rs/blob/master/futures-task/src/waker.rs)。



## 1.4 Rust 异步处理流程

**Reactor Pattern模式**

Reactor Pattern 是构建高性能事件驱动系统的一个很典型模式，executor 和 reactor 是 Reactor Pattern 的组成部分。Reactor pattern 包含三部分：

* task：待处理的任务。任务可以被打断，并且把控制权交给 executor，等待之后的调度
* executor：一个调度器。维护等待运行的任务（ready queue），以及被阻塞的任务（wait queue）
* reactor：维护事件队列。当事件来临时，通知 executor 唤醒某个任务等待运行

executor 会调度执行待处理的任务，当任务无法继续进行却又没有完成时，它会挂起任务，并设置好合适的唤醒条件。之后，如果 reactor 得到了满足条件的事件，它会唤醒之前挂起的任务，然后 executor 就有机会继续执行这个任务。这样一直循环下去，直到任务执行完毕。



**Rust中异步处理的流程**

Rust 使用 Future 做异步处理就是一个典型的Reactor Pattern模式。

> 以 tokio 为例：async/await 提供语法层面的支持，Future 是异步任务的数据结构，当 .await 时，executor 就会调度并执行它

tokio 的调度器会运行在多个线程上，运行线程上自己的 ready queue 上的任务（Future），如果没有，就去别的线程的调度器上偷一些过来运行（work-stealing 调度机制）。当某个任务无法再继续取得进展，此时 Future 运行的结果是 `Poll::Pending`，那么调度器会挂起任务，并设置好合适的唤醒条件（Waker），等待被 reactor 唤醒。而reactor 会利用操作系统提供的异步 I/O（如epoll / kqueue / IOCP），来监听操作系统提供的 IO 事件，当遇到满足条件的事件时，就会调用 Waker.wake() 唤醒被挂起的 Future，这个 Future 会回到 ready queue 等待执行。



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

这是一个简单的 TCP 服务器，服务器每收到一个客户端的请求，就会用 `tokio::spawn` 创建一个异步任务，放入 executor 中执行。这个异步任务接受客户端发来的按行分隔（分隔符是 “\r\n”）的数据帧，服务器每收到一行，就加个前缀把内容也按行发回给客户端。



假设客户端输入了很大的一行数据，服务器在执行` r.next().await ` 时，如果接收不完一行的数据时，这个 Future 返回 `Poll::Pending`，此时它被挂起。当后续客户端的数据到达时，reactor 会知道这个 socket 上又有数据了，于是找到 socket 对应的 Future，将其唤醒，继续接收数据。这样反复下去，最终` r.next().await` 得到 `Poll::Ready(Ok(line))`，于是它返回 `Ok(line)`，程序继续往下走，进入到 `w.send()` 的阶段。



可以用 telnet 来测试：

```rust
❯ telnet localhost 8080
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
hello
I got: hello
Connection closed by foreign host.
```



## 1.5 async 的生命周期

async fn 函数如果拥有引用类型的参数，那它返回的 Future 的生命周期就会被这些参数的生命周期所限制

```rust
async fn foo(x: &u8) -> u8 { *x }
```

上面的函数跟下面的函数是等价的:

```rust
fn foo<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
    async move { *x }
}
```

**说明  async fn 函数返回的 Future 必须满足以下条件:** 当 x 依然有效时， 该 Future 就必须继续等待( `.await` )，也就是说 x 必须比 Future 活得更久。



在一般情况下，在函数调用后就立即 `.await` 不会存在任何问题，例如`foo(&x).await`。但是，若 `Future` 被先存起来或发送到另一个任务或者线程，就可能存在问题。



如下例子会报错，因为 x  的生命周期只到  bad 函数的结尾，但是 borrow_x 返回的 Future 显然会活得更久

```rust
use std::future::Future;

fn bad() -> impl Future<Output = u8> {
    let x = 5;
    borrow_x(&x) // ERROR: `x` does not live long enough
}

async fn borrow_x(x: &u8) -> u8 { *x }
```

一个常用的解决方法：就是将具有引用参数的 `async fn` 函数转变成一个具有 `'static` 生命周期的 Future ，具体可以通过将参数和对 `async fn` 的调用放在同一个 async 语句块来实现。通过将参数移动到 async 语句块内， 将它的生命周期扩展到 `'static`， 并跟返回的 Future 保持了一致，代码如下

```rust
use std::future::Future;

async fn borrow_x(x: &u8) -> u8 { *x }

fn good() -> impl Future<Output = u8> {
    async {
        let x = 5;
        borrow_x(&x).await
    }
}
```



## 1.6 async move

async 可以使用 move 关键字来将环境中变量的所有权转移到语句块内，就像闭包那样，好处是不用解决借用生命周期的问题，坏处就是无法跟其它代码实现对变量的共享

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



## 1.7 async 生成的 Future 是什么类型

**impl Future结构**

异步函数 async fn 或  async block 的返回值是一个的 `impl Future<OutPut>` 的结构。如果给一个普通的函数返回 impl Future，它的行为和 async fn 是一致的。如下代码

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

代码里的 say_hello1 和 say_hello2 是等价的，say_hello1使用了async，say_hello2自己返回了 Impl Future结构；以上代码是使用 await 来执行 Future，也可以将其提供给一个 executor 来执行，如下代码

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



**异步函数返回值的具体类型**

实际 async fn函数 或 async block 的返回值 `impl Future<OutPut>` 结构不是一个具体的类型，它相当于 `T: Future`，那这个 T 是什么呢？先看下面的代码：

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

由此可见，async 代码块返回值是一个叫 GenFuture的结构，它是一个实现了 Future 的 generator，它内部有一个闭包，这个闭包是 `async { 42 } `产生的。简单看下 GenFuture 的定义（可以在 Rust 源码中搜 [from_generator](https://doc.bccnsoft.com/docs/rust-1.36.0-docs-html/src/std/future.rs.html#20-22)），它是一个泛型结构，内部数据 T 要满足 Generator trait：

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

[Generator](https://doc.rust-lang.org/std/ops/trait.Generator.html) 是 Rust nightly 的一个 trait，还没有进入到标准库。看下官网展示的使用例子：

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

可以看到，如果你创建一个闭包，里面有 yield 关键字，就会得到一个 Generator。



## 1.8 异步使用场景的注意事项

### 1.8.1 处理计算密集型任务

要避免在异步任务中处理大量计算密集型的任务，因为效率不高，且还容易饿死其它任务，CPU 密集型任务更适合使用线程，而非 Future。



**饿死其他任务**

因为 Future 的调度是协作式多任务，即除非 Future 主动放弃 CPU，不然它就会一直被执行，直到运行结束。如下例子：

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

这段代码的 task 2 会没有机会执行到，因为 task 1 有一个死循环，task 1 不执行结束（不让出 CPU），task 2 就没有机会被调度。

> 如果真的需要在 tokio（或者其它异步运行时）下运行计算密集型的代码，那么最好使用 yield 来主动让出 CPU，将线程交还给调度器，自己则进入就绪队列等待下一轮的调度，比如 [tokio::task::yield_now()](https://docs.rs/tokio/1.13.0/tokio/task/fn.yield_now.html)，这样可以避免某个计算密集型的任务饿死其它任务。



**在线程和异步任务间做同步时**

当把计算密集型任务放在线程中执行，IO密集型任务放在 tokio 中执行，可以`使用 channel 在 线程 和 future 之间做同步`， 即channel 在 计算密集型 和 IO 密集型任务之间同步。如下例子：

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

这是一个 TCP server 例子，客户端输入过来的一行文字，会被计算出一个 POW（Proof of Work）的哈希：调整 nonce，不断计算哈希，直到哈希的头三个字节全是零为止。服务器要返回计算好的哈希和获得该哈希的 nonce。这是一个典型的`计算密集型任务`，所以使用线程来处理。



而在 tokio task 和 thread 间使用 channel 进行同步。这里使用了一个 ubounded MPSC channel 从 tokio task 侧往 thread 侧发送消息，每条消息都附带一个 oneshot channel 用于 thread 侧往 tokio task 侧发送数据。

> MPSC：Multi-Producer Single-Consumer，多生产者，单消费者



用 telnet 连接进行测试，发送 “hello world!”，会得到不同的哈希和 nonce

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



### 1.8.2 异步代码中使用 Mutex

在使用 Mutex 等同步原语时，要注意标准库的 MutexGuard 无法跨越 `.await`，所以，此时要使用对异步友好的 Mutex，如` tokio::sync::Mutex`。如下例子

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

这个例子模拟了一个数据库的异步 commit() 操作。如果我们需要在多个 tokio task 中使用这个 DB，需要使用 `Arc<Mutext<DB>>`。然而，db1.lock() 拿到锁后，需要运行 db.commit().await，这是一个异步操作。

> 因为 tokio 实现了 work-stealing 调度，Future 有可能在不同的线程中执行，普通的 MutexGuard 编译直接就会出错，所以需要使用 [tokio 的 Mutex](https://docs.rs/tokio/1.13.0/tokio/sync/struct.Mutex.html)



# 2 Pin 和 Unpin

## 2.1 手写 Future 状态机代码

有如下一个 async 函数：首先它创建一个文件，然后往文件里写入 “hello world!”。这个函数有两个 await，创建文件的时候会异步创建，写入文件的时候会异步写入。最终，整个函数对外返回一个 Future

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

因为 executor 在处理 Future 时，会不断地调用它的 poll() 方法，于是上面的 `write_hello_file_async("/tmp/hello").await?` 调用实际上相当于：

```rust
match write_hello_file_async.poll(cx) {
    Poll::Ready(result) => return result,
    Poll::Pending => return Poll::Pending
}
```



再来看下 write_hello_file_async 函数内部的代码，其处理等价于以下代码：

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

由于async 函数返回的是一个 Future，所以，需要把这样的代码封装在一个 Future 的实现里，对外提供出去。因此，需要实现一个数据结构，把内部的状态保存起来，并为这个数据结构实现 Future。比如：

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

接着把 write_hello_file_async 异步函数，转化成了一个返回 WriteHelloFile Future 的函数。来看这个 Future 如何实现：

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

从上面可以看到，这个 Future 完整实现的内部结构 ，其实就是一个状态机的迁移。上面这段伪代码的功能和之前异步函数是等价的：

```rust
async fn write_hello_file_async(name: &str) -> anyhow::Result<()> {
    let mut file = fs::File::create(name).await?;
    file.write_all(b"hello world!").await?;

    Ok(())
}
```

Rust 在编译 async fn 或者 async block 时，就会生成类似的状态机的实现。



## 2.2 Pin 和 Unpin

### 2.2.1 Pin

Pin的定义如下：

```rust
pub struct Pin<P> {
    pointer: P,
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

Pin 包裹一个指针（一个可以解引用成 T 的指针类型 P，而不是直接拿原本的类型 T），并且能确保该指针指向的数据不会被移动，所以，对于 Pin 而言，你看到的都是 `Pin<Box<T>>`、`Pin<&mut T>`，但不会是 `Pin<T>`。



Pin 的目的是，把 T 的内存位置锁住，从而避免移动后自引用类型带来的引用失效问题。



### 2.2.2 为什么要 Pin

在上面手动实现 Future 的状态机中，引用了 file 这样一个局部变量：

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

其实这个代码是有问题的，file 被 fut 引用，但 file 会在这个作用域被丢弃。所以需要把它保存在数据结构中，我们可以生成一个 AwaitingWriteData 数据结构，把 file 和 fut 都放进去，然后在 WriteHelloFile 中引用它。此时，在同一个数据结构内部，fut 指向了对 file 的引用，这样的数据结构，叫**自引用结构（Self-Referential Structure）。**如下代码：

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

**自引用结构有一个很大的问题是：一旦它被移动，原本的指针就会指向旧的地址。**所以需要有某种机制来保证这种情况不会发生，Pin 就是为这个目的而设计的一个数据结构，可以 Pin 住指向一个 Future 的指针。

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-05%2020.38.13.png)





**自引用数据结构的危害**

自引用数据结构并非只在异步代码里出现，只不过异步代码在内部生成用状态机表述的 Future 时，很容易产生自引用结构。看一个和 Future 无关的自引用数据结构的例子

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

    std::mem::swap(&mut data1, &mut data2);
    data1.print_name();
    data2.print_name();
}

fn move_it(data: SelfReference) -> SelfReference {
    data
}
```

代码创建了一个自引用结构 SelfReference，它里面的 name_ref 指向了 name。正常使用它时，没有任何问题，但一旦对这个结构做 move 操作，name_ref 指向的位置依然是 move 前 name 的地址，这就引发了问题。看下图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-05%2020.44.28.png)



同样的，如果我们使用 `std::mem:swap`，也会出现类似的问题，一旦 swap，两个数据的内容交换，然而，由于 name_ref 指向的地址还是旧的，所以整个指针体系都混乱了：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-05%2020.45.20.png)

代码输出如下，可以看到，swap 之后，name_ref 指向的内容确实和 name 不一样了，这就是自引用结构带来的问题

```bash
struct 0x7ffeea91d6e8: (name: 0x7ffeea91d6e8 name_ptr: 0x7ffeea91d6e8), name: Tyr, name_ref: Tyr
struct 0x7ffeea91d760: (name: 0x7ffeea91d760 name_ptr: 0x7ffeea91d6e8), name: Tyr, name_ref: Tyr
data: SelfReference { name: "Tyr", name_ptr: 0x7ffeea91d6e8 }

struct 0x7ffeea91d6f0: (name: 0x7ffeea91d6f0 name_ptr: 0x7ffeea91d6f0), name: Tyr, name_ref: Tyr
struct 0x7ffeea91d710: (name: 0x7ffeea91d710 name_ptr: 0x7ffeea91d710), name: Lindsey, name_ref: Lindsey
struct 0x7ffeea91d6f0: (name: 0x7ffeea91d6f0 name_ptr: 0x7ffeea91d710), name: Lindsey, name_ref: Tyr
struct 0x7ffeea91d710: (name: 0x7ffeea91d710 name_ptr: 0x7ffeea91d6f0), name: Tyr, name_ref: Lindsey
```

这里第二行打印 name_ref 还是指向了 “Tyr”，因为 move 后，之前的内存失效，但是内存地址还没有被挪作它用，所以还能正常显示 “Tyr”。但这样的内存访问是不安全的，如果把 main 中这句代码注释掉，程序就会 crash：

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

所以，Pin 对解决这类问题很关键，如果你试图移动被 Pin 住的数据结构，要不编译器会通过编译错误阻止你；要不你强行使用 unsafe Rust，自己负责其安全性。



**来看下 Pin 住之后的代码 ：**

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

由于数据结构被包裹在 Pin 内部，所以在函数间传递时，变化的只是指向 data 的 Pin，避免了移动带来的问题。

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230106000101.png)





### 2.2.3 Unpin

[Unpin](https://doc.rust-lang.org/std/marker/trait.Unpin.html) 是个标记trait( marker trait )，定义如下

```rust
pub auto trait Unpin {}
```

Pin 可以防止一个类型在内存中被移动，而 Unpin 相反，Unpin 则相当于声明的数据结构是可以在内存中安全的移动的，它的作用类似于 Send / Sync，通过类型约束来告诉编译器哪些行为是合法的、哪些不是合法的。



在 Rust 中，绝大多数数据结构都是可以移动的，所以它们都自动实现了Unpin，即便这些结构被 Pin 住，它们依旧可以进行移动，例如

```rust
use std::mem;
use std::pin::Pin;

let mut string = "this".to_string();
let mut pinned_string = Pin::new(&mut string);

// We need a mutable reference to call `mem::replace`.
// We can obtain such a reference by (implicitly) invoking `Pin::deref_mut`,
// but that is only possible because `String` implements `Unpin`.
mem::replace(&mut *pinned_string, "other".to_string());
```



**当希望一个数据结构不能被移动，可以使用 `!Unpin`。在 Rust 里，实现了 `!Unpin `的，除了内部结构（比如 Future），主要就是 PhantomPinned，所以如果希望数据结构不能被移动，可以为其添加 PhantomPinned 字段来隐式声明 `!Unpin`。**

```rust
pub struct PhantomPinned;
impl !Unpin for PhantomPinned {}
```



### 2.2.4 Pin 在实践中的应用

**1、将值固定到栈上**

可以用 `Pin` 来解决指针指向的数据被移动的问题

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

代码中使用了一个标记类型 `PhantomPinned` 将自定义结构体 Test 变成了 `!Unpin` (编译器会自动帮我们实现)，因此该结构体无法再被移动。一旦类型实现了 `!Unpin` ，那将它的值固定到栈上就是不安全的行为，因此在代码中我们使用了 unsafe 语句块来进行处理，这里也可以使用 [`pin_utils`](https://docs.rs/pin-utils/) 来避免 `unsafe` 的使用。此时，再去尝试移动被固定的值，就会导致**编译错误** ：

```rust
pub fn main() {
    // 此时的test1可以被安全的移动
    let mut test1 = Test::new("test1");
  
    // 新的test1由于使用了Pin，因此无法再被移动，这里的声明会将之前的test1遮蔽掉
    let mut test1 = unsafe { Pin::new_unchecked(&mut test1) };
    Test::init(test1.as_mut());

    let mut test2 = Test::new("test2");
    let mut test2 = unsafe { Pin::new_unchecked(&mut test2) };
    Test::init(test2.as_mut());

    println!("a: {}, b: {}", Test::a(test1.as_ref()), Test::b(test1.as_ref()));
    std::mem::swap(test1.get_mut(), test2.get_mut());
    println!("a: {}, b: {}", Test::a(test2.as_ref()), Test::b(test2.as_ref()));
}

```



**2、固定到堆上**

将一个 `!Unpin` 类型的值固定到堆上，会给予该值一个稳定的内存地址，它指向的堆中的值在 Pin 后是无法被移动的。而且与固定在栈上不同，堆上的值在整个生命周期内都会被稳稳地固定住。

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
    println!("a: {}, b: {}",test2.as_ref().a(), test2.as_ref().b());
}
```



**3、将固定住的 Future 变为 Unpin**

async 函数返回的 Future 默认就是 `!Unpin` 的。在实际应用中，一些函数会要求它们处理的 Future 是 Unpin 的，此时必须要使用以下的方法先将 Future 进行固定:

- `Box::pin`：创建一个 `Pin<Box<T>>`
- `pin_utils::pin_mut!`： 创建一个 `Pin<&mut T>`

固定后获得的 `Pin<Box<T>>` 和 `Pin<&mut T>` 既可以用于 Future ，**又会自动实现 Unpin**。

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



# 3 如何处理异步IO

## 3.1 异步的 Stream trait

 Stream trait 类似于 Future trait，但 Stream 在完成前可以生成多个值，这种行为跟标准库中的 Iterator trait 类似。不过和 Future 已经在标准库稳定下来不同，Stream trait 目前还只能在 nightly 版本使用。一般跟 Stream 打交道，会使用 futures 库。



**Iterator 和 Stream 的定义分别如下：**

```rust
// Iterator 把所有方法都放在 Iterator trait 里
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;

    fn size_hint(&self) -> (usize, Option<usize>) { ... }
    fn map<B, F>(self, f: F) -> Map<Self, F> where F: FnMut(Self::Item) -> B { ... }
    ... // 还有很多方法
}

//  Stream 把需要开发者实现的基本方法和有缺省实现的衍生方法区别开，放在不同的 trait 里
pub trait Stream {
    type Item;
    fn poll_next(self: Pin<&mut Self>,  cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;

    fn size_hint(&self) -> (usize, Option<usize>) { ... }
}

pub trait StreamExt: Stream {
    fn next(&mut self) -> Next<'_, Self> where Self: Unpin { ... }
    fn map<T, F>(self, f: F) -> Map<Self, F> where F: FnMut(Self::Item) -> T { ... }
    ... // 还有很多方法
}
```

* Iterator 可以不断调用 next() 方法，获得新的值，直到 Iterator 返回 None。但是 Iterator 是阻塞式返回数据的，每次调用 next()，必然 **独占CPU** 直到得到一个结果，而异步的 Stream 是非阻塞的，在等待的过程中会空出 CPU 做其他事情

* Stream 的 poll_next() 方法，它跟 Future 的 poll() 方法很像，和 Iterator 版本的 next() 的作用类似。然而，poll_next() 调用起来不方便，我们需要自己处理 Poll 状态，所以，StreamExt 提供了 next() 方法，返回一个实现了 Future trait 的 Next 结构，这样就可以直接通过 `stream.next().await `来获取下一个值了。



**StreamExt 中 next() 方法以及 Next 结构的实现：**

```rust
pub trait StreamExt: Stream {
    fn next(&mut self) -> Next<'_, Self> where Self: Unpin {
        assert_future::<Option<Self::Item>, _>(Next::new(self))
    }
}

// next 返回了 Next 结构
pub struct Next<'a, St: ?Sized> {
    stream: &'a mut St,
}

// 如果 Stream Unpin 那么 Next 也是 Unpin
impl<St: ?Sized + Unpin> Unpin for Next<'_, St> {}

impl<'a, St: ?Sized + Stream + Unpin> Next<'a, St> {
    pub(super) fn new(stream: &'a mut St) -> Self {
        Self { stream }
    }
}

// Next 实现了 Future，每次 poll() 实际上就是从 stream 中 poll_next()
impl<St: ?Sized + Stream + Unpin> Future for Next<'_, St> {
    type Output = Option<St::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.stream.poll_next_unpin(cx)
    }
}
```

如下例子

```rust
use futures::prelude::*;

#[tokio::main]
async fn main() {
    let mut st = stream::iter(1..10)
        .filter(|x| future::ready(x % 2 == 0))
        .map(|x| x * x);

    while let Some(x) = st.next().await {
        println!("Got item: {}", x);
    }
}
```

代码使用 `stream::iter ` 生成了一个 Stream，并对其进行 filter / map 的操作。最后，遍历整个 stream，把获得的数据打印出来。



**futures 库提供了一些基本的生成 Stream 的方法，除了上面用到的 iter 方法，还有：**

* empty()：生成一个空的 Stream
* once()：生成一个只包含单个值的 Stream
* pending()：生成一个不包含任何值，只返回 Poll::Pending 的 Stream
* repeat()：生成一个一直返回相同值的 Stream
* repeat_with()：通过闭包函数无穷尽地返回数据的 Stream
* poll_fn()：通过一个返回 Poll> 的闭包来产生 Stream
* unfold()：通过初始值和返回 Future 的闭包来产生 Stream

最后三种repeat_with、poll_fn、unfold 引入了闭包，分别使用它们来实现斐波那契数列，对比一下差异：

```rust
use futures::{prelude::*, stream::poll_fn};
use std::task::Poll;

#[tokio::main]
async fn main() {
    consume(fib().take(10)).await;
    consume(fib1(10)).await;
  
    // unfold 产生的 Unfold stream 没有实现 Unpin，所以我们将其 Pin<Box<T>> 一下，使其满足 consume 的接口
    consume(fib2(10).boxed()).await;
}

async fn consume(mut st: impl Stream<Item = i32> + Unpin) {
    while let Some(v) = st.next().await {
        print!("{} ", v);
    }
    print!("\\n");
}

// 使用 repeat_with 创建 stream，无法控制何时结束
fn fib() -> impl Stream<Item = i32> {
    let mut a = 1;
    let mut b = 1;
  
    stream::repeat_with(move || {
        let c = a + b;
        a = b;
        b = c;
        b
    })
}

// 使用 poll_fn 创建 stream，可以通过返回 Poll::Ready(None) 来结束
fn fib1(mut n: usize) -> impl Stream<Item = i32> {
    let mut a = 1;
    let mut b = 1;
  
    poll_fn(move |_cx| -> Poll<Option<i32>> {
        if n == 0 {
            return Poll::Ready(None);
        }
        n -= 1;
        let c = a + b;
        a = b;
        b = c;
        Poll::Ready(Some(b))
    })
}

// 使用 unfold 创建 stream
fn fib2(n: usize) -> impl Stream<Item = i32> {
    stream::unfold((n, (1, 1)), |(mut n, (a, b))| async move {
        if n == 0 {
            None
        } else {
            n -= 1;
            let c = a + b;
            // c 作为 poll_next() 的返回值，(n, (a, b)) 作为 state
            Some((c, (n, (b, c))))
        }
    })
}
```

值得注意的是，使用 unfold 方法时，同时使用了局部变量和 Future，所以生成的 Stream 没有实现 Unpin，在使用时需要将其 pin 住，解决方式：使用`Pin<Box<T>>` 将数据 Pin 在堆上，即可以使用 StreamExt 的 boxed() 方法来生成一个 `Pin<Box<T>>`。



## 3.2 异步 IO 接口

所有同步的IO，如 Read / Write / Seek trait，前面加一个 Async，就构成了对应的异步 IO 接口。

> Read/Write 是进行 IO 的读写，而 Seek 是在 IO 中前后移动当前的位置



注意 futures 下定义的 IO trait 以及 tokio 下定义的 IO trait，双方有些许的差别，它们都有各自的定义：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230305230601.png)



因为在 tokio / futures 库实现的早期，社区还没有形成比较统一的异步 IO trait，不同的接口背后也有各自不同的考虑，这种分裂就沿袭下来。虽然 Rust 的异步 IO trait 有这样的分裂，但 tokio-util 提供了相应的 [Compat](https://docs.rs/tokio-util/0.6.9/tokio_util/compat/index.html) 功能，可以让你的数据结构在二者之间自如切换。



**AsyncRead**

futures 下 [AsyncRead](https://docs.rs/futures/0.3.17/futures/io/trait.AsyncRead.html) 的定义如下：

```rust
pub trait AsyncRead {
    fn poll_read(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>, 
        buf: &mut [u8]
    ) -> Poll<Result<usize, Error>>;

    unsafe fn initializer(&self) -> Initializer { ... }
    fn poll_read_vectored(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>, 
        bufs: &mut [IoSliceMut<'_>]
    ) -> Poll<Result<usize, Error>> { ... }
}
```

而 tokio 下 [AsyncRead](https://docs.rs/tokio/1.14.0/tokio/io/trait.AsyncRead.html) 的定义如下：

```rust
pub trait AsyncRead {
    fn poll_read(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>, 
        buf: &mut ReadBuf<'_>
    ) -> Poll<Result<()>>;
}
```

不同：tokio 的 poll_read() 方法需要 [ReadBuf](https://docs.rs/tokio/1.14.0/src/tokio/io/read_buf.rs.html#27-31)，而 futures 的 poll_read() 方法需要`&mut [u8]`。此外，futures 的 AsyncRead 还多了两个缺省方法。



**AsyncWrite**

futures 下的 [AsyncWrite](https://docs.rs/futures/0.3.17/futures/io/trait.AsyncWrite.html) 定义如下：

```rust
pub trait AsyncWrite {
    fn poll_write(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>, 
        buf: &[u8]
    ) -> Poll<Result<usize, Error>>;
    fn poll_flush(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>
    ) -> Poll<Result<(), Error>>;
    fn poll_close(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>
    ) -> Poll<Result<(), Error>>;

    fn poll_write_vectored(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>, 
        bufs: &[IoSlice<'_>]
    ) -> Poll<Result<usize, Error>> { ... }
}
```

而 tokio 下的 [AsyncWrite](https://docs.rs/tokio/1.14.0/tokio/io/trait.AsyncWrite.html)_ 的定义如下：

```rust
pub trait AsyncWrite {
    fn poll_write(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>, 
        buf: &[u8]
    ) -> Poll<Result<usize, Error>>;
    fn poll_flush(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>
    ) -> Poll<Result<(), Error>>;
    fn poll_shutdown(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>
    ) -> Poll<Result<(), Error>>;

    fn poll_write_vectored(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>, 
        bufs: &[IoSlice<'_>]
    ) -> Poll<Result<usize, Error>> { ... }
    fn is_write_vectored(&self) -> bool { ... }
}
```

可以看到，AsyncWrite 二者的差距就只有 poll_close() 和 poll_shutdown() 命名上的分别。



**实现异步 IO 接口**

异步 IO 主要应用在文件处理、网络处理等场合，而这些场合的数据结构都已经实现了对应的接口，比如 File 或者 TcpStream，它们也已经实现了 AsyncRead / AsyncWrite，所以基本上不用自己实现异步 IO 接口。不过有些情况，可能会把已有的数据结构封装在自己的数据结构中，此时需要自己实现相应的异步 IO 接口，如下代码

```rust
use anyhow::Result;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, ReadBuf},
};

#[pin_project]
struct FileWrapper {
    #[pin]
    file: File,
}

impl FileWrapper {
    pub async fn try_new(name: &str) -> Result<Self> {
        let file = File::open(name).await?;
        Ok(Self { file })
    }
}

impl AsyncRead for FileWrapper {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.project().file.poll_read(cx, buf)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut file = FileWrapper::try_new("./Cargo.toml").await?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).await?;
    println!("{}", buffer);
    Ok(())
}
```

这段代码封装了 `tokio::fs::File` 结构，因为想读取内部的 file 字段，但又不想把 File 暴露出来，因此实现了 AsyncRead trait。



# 参考

* [陈天 · Rust 编程第一课-异步处理](https://time.geekbang.org/column/article/455413)
* [async/await异步编程](https://course.rs/advance/async/intro.html)

