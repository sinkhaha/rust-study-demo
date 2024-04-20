`Rust` 使用线程的 3 种方法

* 分叉与合并（`fork-join`）并行
* 通道
* 共享可变状态



# 1 分叉与合并并行

分叉与合并并行：适用于完全独立的任务的同时进行，每个任务都用单独的线程去运行。



分叉（`fork`）是启动一个新线程，合并（`join`）是等待线程完成。



优点：

* 简单：容易看出结果是否正常

* 避免了瓶颈：分叉和合并没有对共享资源的锁定，每个线程自由运行，有助于降低任务切换的开销

  

缺点：

* 要求工作单元彼此隔离
* 有时分叉与合并程序在线程联结后要花费一些时间来组合各线程的计算结果



## 1.1 启动和联结

函数 `std::thread::spawn` 接受一个 `FnOnce` 闭包 或 函数型的参数，会启动一个新线程来运行该闭包或函数中的代码，如

```rust
use std::thread;

thread::spawn(|| {
  println!("hello form a child thread");
});
```

`spawn` 会返回一个名为 `JoinHandle` 的值，该类型可以使用线程的 `join()` 函数等待线程结束。如果没有调用 `join()` 联结等待线程执行完，那么 `Rust` 程序会在 `main` 返回后立即退出，即使它的子线程还在运行，这些子线程不会调用析构器，而是直接被“杀死”了。



如下例子，为每个 `worklist` 启动一个线程

```rust
use std::{io, thread};

fn process_files(filenames: Vec<String>) -> io::Result<()> {
    println!("{:?}", filenames);
    Ok(())
}

fn process_files_in_parallel(filenames: Vec<String>) -> io::Result<()> {
    // 分块大小
    let chunk_size = 2;

    // 分块并创建新变量
    let worklists: Vec<Vec<String>> = filenames
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect();

    // 分叉：对每一个块启动一个线程处理
    let mut thread_handles = vec![];
    for worklist in worklists {
        thread_handles.push(thread::spawn(move || process_files(worklist)))
    }

    // 联结：等待所有线程结束
    for handle in thread_handles {
        handle.join().unwrap()?; // 这里用unwrap处理Result值
    }

    Ok(())
}

fn main() {
    let filenames = vec![
        "Hello".to_string(),
        "World".to_string(),
        "Rust".to_string(),
        "Programming".to_string(),
    ];

    process_files_in_parallel(filenames).unwrap();
}
```



**rayon库**

`rayon` 库封装了更好的分叉与合并式的 `API`。它提供了两种运行并发任务的方式，在这两种情况下，`rayon` 会用自己的工作线程池来尽可能拆分工作，例如

```rust
use rayon::prelude::*;

// 并行做两件事
let (v1, v2) = rayon::join(fn1, fn2);

// 并行做 N件事
giant_vector.par_iter().for_each(|value| {
  do_thing_with_value(value);
});
```

`par_iter()` 方法会创建 `ParallelIterator`



## 1.2 跨线程错误处理

在 `Rust` 中，`panic` 是安全且局限于每个线程的，`panic` 不会自动从一个线程传播到依赖它的其他线程。一个线程中的 `panic` 在其他线程中会报告为错误类型 `Result`。



上面例子的 `handle.join()` 返回的完整类型是 `std::thread::Result<std::io::Result<()>>`，

`thread::Result` 是 `spawn/join` 的一部分，而 `io::Result` 是我们自己程序的一部分。

如果子线程出现了 `panic`，就返回一个错误 `Err`，此时使用的 `unwrap()` 也会发生 `panic`，所以父线程也会出现 `panic`，这样就显示的将 `panic` 从子线程传播到了父线程。



`handle.join()` 会将子线程的返回值传回给父线程，上面例子就会是 `process_files()` 函数的返回值，当子线程完成时，这个返回值会被保存下来，并且 `JoinHandle::join()` 会把该值传回给父线程。



# 2 通道

通道是一种单向通道，用于将值从一个线程发送到另一个线程，简单来说，通道是一个线程安全的队列。使用通道，线程可以通过彼此传值来进行通信，无须使用锁或共享内存。



通道一端用于发送数据，一端用于接收数据。两端通常由两个不同的线程拥有，通道用于发送 `Rust` 中的值，值的所有权会从发送线程转移给接收线程。有点类似 `Unix` 管道，不过 `Unix` 管道用于发送字节

* `sender.send(item) `：将单个值发送到通道
* `receiver.recv()`：则会从通道中接收到值，如果通道为空，会一直阻塞到有值为止



## 2.1 发送值 和接收者

**发送者**

通道是 `std::sync::mpsc` 模块的一部分，可以使用 `mpsc::channel` 创建一个通道，如

> `std::sync::mpsc` 中的 `mpsc` 是 `Multiple Producers, Single Consumer`，即多生产者、单消费者

```rust
use std::sync::mpsc;
use std::{fs, thread};

let (sender, receiver) = mpsc::channel();// 创建通道，返回发送者 和 接收者

// 创建新线程
let handle = thread::spawn(move || {
   for filename in documents {
      let text = fs::read_to_string(filename)?;

      // 在新线程中发消息
      // sender 的所有权会通过这个 move闭包转移给新线程
      if sender.send(text).is_err() {
          break;
      }
   }

   Ok(())
});

```

* 通道是有类型的，这里在新线程中使用通道发送每个文件的文本，所以 `sender` 和 `receiver` 的类型分别是 `Sender<String>` 和 `Receiver<String>`，也可以写成 `mpsc::channel()::<String>` 来明确创建一个字符串类型的通道

* `Sender<T>` 实现了 `Clone` 特型。要获得具有多个发送者的通道，只要创建一个常规通道并根据需要多次克隆发送着即可，可以将每个 `Sender` 值转移给不同的线程



**接收值**

可以启动一个循环来接收值，如

```rust
while let Ok(text) = receiver.recv() {
  do_something_with(text);
}
```

* 当通道恰好为空，接收线程在其他线程发送值之前会阻塞
* 当通道为空且 `sender` 已被丢弃时，循环将正常退出。上面的发送代码中，当读取后发送线程退出时，接收者循环就会退出，因为发送代码中运行一个拥有变量 `sender` 的闭包，当闭包退出时，`sender` 会被丢弃



`Receiver<T>` 不能被克隆，如果需要让多个线程从同一个通道接收值，需要使用 `Mutex` 互斥锁。



`send` 方法和 `recv` 方法都会返回 `Result` ，如果 `receiver` 接收者已被丢弃，那么 `send` 方法调用就会失败；如果通道中没有值在等待并且 `sender` 已被丢弃，则 `recv` 方法调用也会失败，因为如果不失败，`recv` 就只能永远等待：而没有 `sender`，任何线程都无法再发出下一个值。



丢弃通道的某一端都是正常的“挂断”方式，完成后就会关闭连接。无论接收者是故意退出还是出错退出，读取者线程都可以悄悄的自行关闭。



**同步通道**

如果发送值的速度快于接收值和处理的速度，会导致通道中积压的值不断增长。可以利用 `Unix` 的背压技巧解决，迫使超过的发送者放慢速度：`Unix` 系统上的每个管道都有固定的大小，如果进程时图写入暂时已满的管道，那么系统就会简单地阻塞该进程直到管道中有了空间。在 `Rust` 中的等效设计称为**同步通道**，如

```rust
use std::sync::mpsc;

let (sender, receiver) = mpsc::sync_channel(1000);
```

同步通道创建时可以指定它能容纳多少个值，对于同步通道，`sender.send(value)` 可能是一个阻塞操作



## 2.2 线程安全 Send / Sync

`Rust` 中的线程安全取决于两个特型：`std::marker::Send` 和 `std::marker::Sync`

* 实现了 `Send` 的类型可以安全地 按值传给另一个线程，它们可以跨线程移动
* 实现了 `Sync` 的类型可以安全地将一个值的不可变引用传给另一个线程，它们可有跨线程共享

这里的安全指的是没有数据竞争和其他未定义行为。



对于跨线程边界传输数据的函数，`Send` 和 `Sync` 会作为函数类型签名中的限界。当你生成 `spawn` 一个线程时，传入的闭包必须实现了 `Send` 特型，意味着它包含的所有值都必须是 `Send` 的。同样，如果要通过通道将值发送到另一个线程，则该值必须是 `Send` 的。



比如 `mpsc::Receiver` 是 `Send` ，但不是 `Sync` ，它是为了保证 `mpsc` 通道的接收端一次只能被一个线程使用。又比如引用计数智能指针类型 `std::rc::Rc<T>` 既不是 `Send` 也不是 `Sync` 。跨线程共享不可变数据可以使用原子化引用计数 `Arc`。



# 3 共享可变状态

## 3.1 互斥锁 Mutex

互斥锁 （`mutex`） 或 锁（`lock`） 用于强制多个线程在访问某些数据时轮流读写。



**互斥锁的作用**

* 防止数据竞争，即多个竞争线程同时读取或写入同一个内存的情况
* 即使不存在数据竞争，如果没有互斥锁，不同线程的操作也可能会以任意方式相互交错
* 互斥锁支持使用不变条件进行编程



**使用互斥锁的例子**

例如要在 `Rust` 中实现等待列表

```rust
type PlayerId = u32; // 每个玩家都有一个唯一id

const GAME_SIZE: usize = 8;

// 等待列表永远不会超过GAME_SIZE个玩家
type WaitingList = Vec<PlayerId>;
```

等待列表会被存储为 `FernEmpireApp` 中的一个字段，这是在服务器启动期间在 `Arc` 中设置的一个单例。每个线程都有一个 `Arc` 指向它，它包含程序中所需的全部共享配置，其中大部分是只读的，由于等待列表既是共享的又是可变的，因此必须由 `Mutex` 提供保护

```rust
use std::sync::Mutex;

// 所有线程都可以共享对这个大型上下文结构体的访问
struct FernEmpireApp {
  ...
  waiting_list: Mutex<WaitingList>,
  ...
}
```

在 `Rust` 中，受保护的数据存储于 `Mutex` 内部。

```rust
use std::sync::Arc;

let app = Arc::new(FernEmpireApp {
  ...
  waiting_list::Mutex::new(vec![]); // 
  ...
});
```

`Arc` 用于跨线程共享不可变数据，`Mutex` 用于跨线程共享可变数据。



现在可以使用实现互斥锁的 `join_waiting_list` 方法了，如

```rust
impl FernEmpireApp {
  // 往下一个游戏的等待列表中添加一个晚集，如果有足够的待进入玩家，则立即启动一个新游戏
  fn join_waiting_list(&self, player: PlayerId) {
    // self.waiting_list.lock() 锁定互斥锁，并授予内部数据的访问权， guard的作用域是一个临界区
    let mut guard = self.waiting_list.lock().unwrap();
    
    // 开始执行游戏逻辑
    guard.push(player);
    
    if guard.len() == GAME_SIZE {
      let players = guard.split_off(0);
      // drop(guard);// 这里可以手动丢弃guard，这样会释放锁，也可以自动丢弃
      self.start_game(players);
    }
  }
}
```

获取数据唯一方式就是调用 `.lock()` 方法 `self.waiting_list.lock()`，这里会阻塞，直到获得互斥锁。当 `gurad` 被丢弃时，锁就被释放了，也可以手动丢弃。



**中毒的互斥锁**

`Mutex::lock()` 返回 `Result` 的原因与 `JoinHandle::join()` 一样的：如果另一个线程发生 `panic`，则可以优雅地失败。当我们编写 `handle.join().unwrap() `时，就是告诉 `Rust` 将 `panic` 从一个线程传播到另一个线程。`mutex::lock().unwrap()` 也如此。



如果线程持有 `Mutex` 期间出现 `panic` ，则 `Rust` 会把 `Mutex` 标记为已"中毒"，之后每当试图锁住已“中毒”的`Mutex` 时都会得到错误结果。如果发生这种情况，使用 `.unwrap()` 调用就会告诉 `Rust` 发生了 `panic` ，将 `panic` 从另一个线程传播到本线程。



### 3.1.1 使用互斥锁的多消费者通道

`Rust` 的通道是多生产者，单一消费者的。一个通道只能有一个 `Receiver`。如果有一个线程池，则不能让其中的多个线程使用单个 `mpsc` 通道作为共享列表。有一种简单的解决办法：在 `Receiver· 周围包装一个 `Mutex` 然后再共享。例如：

```rust
pub mod shard_channel {
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct SharedReceiver<T>(Arc<Mutex<Receiver<T>>>);

    impl<T> Iterator for SharedReceiver<T> {
        type Item = T;

        // 从已包装的接收者中获取下一个条目
        fn next(&mut self) -> Option<T> {
            let guard = self.0.lock().unwrap();
            guard.recv().ok()
        }
    }

    // 创建一个新通道，它的接收者可以跨线程共享。这会返回一个发送者和一个接收者，就像标准库中的channel()
    pub fn shard_channel<T>() -> (Sender<T>, SharedReceiver<T>) {
        let (sender, receiver) = channel();

        (sender, SharedReceiver(Arc::new(Mutex::new(receiver))))
    }
}

```



## 3.2 死锁

线程在尝试获取自己正持有的锁时会让自己陷入死锁

```rust
let mut guard1 = self.waiting_list.lock().unwrap();

let mut guard2 = self.waiting_list.lock().unwrap(); // 死锁
```

假设第一次调用 `self.waiting_list.lock()` 成功，获得了锁，那第二次调用时看到锁已被持有，所以线程就会阻塞自己，等待锁被释放，它会永远等下去，因为这个正在等待的线程就是持有锁的线程。



在实际代码中，这两个 `lock()` 调用可能位于不同的方法，其中一个调用另一个。比如涉及多个线程 或 每个线程同时获取多个互斥锁，也可能导致死锁。



`Rust` 的借用系统不能保护你免于死锁。最好的保护方式是保持临界区尽可能小。



通道也有可能陷入死锁。例如两个线程可能会互相阻塞，每个线程都在等待从另一个线程接收消息。



## 3.3 读写锁 RwLock

互斥锁只有一个 `lock` 方法，而读写锁 `RwLock` 有两个，一个 `read()`，一个 `write()`。

* `RwLock::write` 方法类似于 `Mutex::lock`，它会等待对受保护数据的独占的 `mut` 访问。

* `RwLock::read` 方法提供了非 `mut` 访问，它的优点是可能不怎么等待，因为本来就可以让多个线程同时安全的读取。



使用互斥锁，任何时刻，受保护的数据只能有一个读取者或写入者。读写锁，任何时刻，受保护的数据可以有一个写入者 或 多个读取者。



## 3.4 条件变量 Condvar

在关闭服务器的过程中，主线程可能需要等所有其他线程都完成才能退出，比如 `JoinHandle.join()` 方法。大多数情况下，`Rust` 没有内置的阻塞式 `API` ，我们可以使用条件变量来构建自己的 `API` 。



在 `Rust` 中，`std::sync::Condvar` 类型实现了条件变量，它有 `wait()` 方法和 `notify_all()` 方法，`wait()` 方法会阻塞线程，直到其他线程调用了 `notify_all()` 方法。



当所需条件变为真时，就调用 `Condvar::notify_all` （或`notify_one`）来唤醒所有等待的线程

```rust
self.has_data_condvar.notify_all();
```

要进入睡眠状态并等待条件为真时，可以使用 `Condvar::wait()`

```rust
while !guard.has_data() {
  guard = self.has_data_condvar.wait(gurad).unwrap();
}
```



## 3.5 原子化类型

`std::sync::atomic` 模块包含用于无锁并发编程的原子化类型，原子化类型的方法可以编译成专门的机器语言指令

* `AtomicIsize` 和 `AtomicUsize` 是与单线程 `isize` 类型和 `usize` 类型对应的共享整数类型
* `AtomicI8`、`AtomicI16`、`AtomicI32`、`AtomicI64` 及其无符号变体（如 `AtomicU8`）是共享整数类型，对于单线程中的类型 `i8`、`i16` 等
* `AtomicBool` 是一个共享的 `bool` 值
* `AtomicPtr<T>` 是不安全指针类型 `*mut T` 的共享值



多个线程可以同时读取和写入一个原子化的值并不会导致数据竞争



## 3.6 全局变量

用 `const` 声明的常量是不可变的，默认情况下，`static` 静态变量也是不可变的，因此无法获得一个 `mut` 引用

```rust
static PACKETS_SERVED: usize = 0;  // 静态变量
```

此时`PACKETS_SERVED` 是静态变量，永远不能改变它。`static` 静态变量可以声明为 `mut`，但是访问它是不安全的。



要找到一种安全的方法来声明可变静态变量，例如支持递增 `PACKETS_SERVED` 并保持其线程安全的最简单方式是让它变成原子化整数

```rust
use std::sync::atomic::AtomicUsize;

static PACKETS_SERVED: AtomicUsize = AtomicUsize::new(0);  
```

接着增加计数就简单了

```rust
use std::sync::atomic::Ordering;

PACKETS_SERVED.fetch_add(1, Ordering.SeqCst);
```

原子化全局变量仅限于简单的整数和布尔值。



**要创建任何其他类型的全局变量，就要解决2个问题**

1. 变量必须以某种方式称为线程安全的，否则它就不能是全局变量：为了安全起见，静态变量必须同时是 `Sync`和非 `mut` 的，`Rust` 具有用于安全地共享变化的值类型: `Mutex`、`RwLock` 和原子化类型

2. 静态初始化程序只能调用被专门标记为 `const` 的函数，编译器可以在编译期间对其进行求值

   

`Atomic` 类型的构造函数都是 `const` 函数。可以直接在函数的签名前加上 `const` 来定义自己的 `const` 函数，如

```rust
const fn mono_to_rgba(level: u8) -> Color {
  Color {
    red: level,
    green: level,
    blue: level,
    alpha: 0xFF
  }
}

const WHITE: Color = mono_to_rgba(255);
const BLACK: Color = mono_to_rgba(000);
```



静态调用仅限于常量函数、常量元组、常量结构体、常量元组变体，所以以下写法是错误的

```rust
static HOSTNAME: Mutex<String> = Mutex::new(String::new()); // 错误
```



虽然 `AtomicUsize::new()` 和 `String::new() ` 是 `const fn`，但 `Mutex::new()` 不是，为了绕过这些限制，可以使用 `lazy_static crate`。通过 `lazy_static!` 宏定义的变量允许你使用任何喜欢的表达式进行初始化，该表达式会在第一次解引用时运行，并保存该值以供后续操作使用。例如声明一个全局 `Mutex` 控制的 `HashMap`

```rust
use lazy_static::lazy_static;

use std::sync::Mutex;

lazy_static! {
  static ref HOSTNAME: Mutex<String> = Mutex::new(String::new());
}
```



# 4 参考

* [Rust程序设计（第2版）](https://book.douban.com/subject/36547630/)

