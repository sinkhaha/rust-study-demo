# 1 为什么有 unsafe

* `unsafe` 存在的主要原因是 `Rust` 的静态检查太强了；`Rust` 为了内存安全，所做的所有权、借用检查、生命周期等规则往往是普适性的，编译器在分析代码时，一些正确代码会因为编译器无法分析出它的所有正确性，结果将这段代码拒绝，导致编译错误

* 另一个原因是计算机底层的一些硬件就是不安全的（比如操作 `IO` 访问外设），这些操作编译器是无法保证内存安全的，如果 `Rust` 只允许做安全的操作，那就无法完成这些操作，所以需要 `unsafe`



# 2 使用 unsafe 的场景

## 2.1 实现 unsafe trait 

**任何 `trait` 只要声明成 `unsafe`，它就是 `unsafe trait`。在实现 `unsafe trait` 时，也必须定义为 `unsafe`。**



`unsafe trait` 是对 `trait` 的实现者的约束，它表示在实现该 `trait` 时要小心，要保证内存安全，所以实现时需要加上 `unsafe` 关键字

> 但是在调用 `unsafe trait` 时，直接直接调用，不需要在 `unsafe` 块中调用，因为这里的安全已经被实现者保证了，毕竟如果实现者没保证，调用者也做不了什么来保证安全，就像使用 `Send / Sync trait` 一样



如下例子：

```rust
// 这是一个unsafe trait，实现这个 trait 的开发者要保证实现是内存安全的
unsafe trait Foo {
    fn foo(&self);
}

struct Nonsense;

// 使用 unsafe
unsafe impl Foo for Nonsense {
    fn foo(&self) {
        println!("foo!");
    }
}

fn main() {
    let nonsense = Nonsense;
    nonsense.foo();
}
```



### 2.1.1 Send / Sync trait

`Rust` 中的 `Send / Sync` ，这两个 `trait` 都是 `unsafe trait`，定义如下

```rust
pub unsafe auto trait Send {}
pub unsafe auto trait Sync {}
```

- `auto` 表示编译器会在合适的场合，自动为数据结构添加它们的实现
- `unsafe` 代表实现的这个 `trait` 可能会违背 `Rust` 的内存安全准则



**绝大多数数据结构都实现了 `Send / Sync`，但有一些例外，比如 `Rc / RefCell / 裸指针` 等。**因为裸指针没有实现 `Send / Sync`。当你在数据结构里使用裸指针时，如果你的结构是线程安全的，需要为数据结构手动实现 `Send / Sync`。如下 [Bytes](https://docs.rs/bytes/1.1.0/src/bytes/bytes.rs.html#508-510)  就在使用裸指针的情况下实现了 `Send / Sync`：

```rust
pub struct Bytes {
    ptr: *const u8,
    len: usize,
    // inlined "trait object"
    data: AtomicPtr<()>,
    vtable: &'static Vtable,
}

// Vtable must enforce this behavior
unsafe impl Send for Bytes {}
unsafe impl Sync for Bytes {}
```



**注意：**在实现 Send/Sync 时，如果无法保证数据结构的线程安全，错误实现 `Send/Sync` 之后，会导致程序出现莫名其妙的还不太容易复现的崩溃。



如下代码，强行为 `Evil` 实现了 `Send`，而 `Evil` 内部携带的 `Rc` 是不允许实现 `Send` 的。代码通过实现 `Send` 而规避了 `Rust` 的并发安全检查，使其可以编译通过，然而在运行时，有一定的几率出现崩溃：

```rust
use std::{cell::RefCell, rc::Rc, thread};

#[derive(Debug, Default, Clone)]
struct Evil {
    data: Rc<RefCell<usize>>,
}

// 为 Evil 强行实现 Send，这会让 Rc 整个紊乱
unsafe impl Send for Evil {}

fn main() {
    let v = Evil::default();
    let v1 = v.clone();
    let v2 = v.clone();

    let t1 = thread::spawn(move || {
        let v3 = v.clone();
        let mut data = v3.data.borrow_mut();
        *data += 1;
        println!("v3: {:?}", data);
    });

    let t2 = thread::spawn(move || {
        let v4 = v1.clone();
        let mut data = v4.data.borrow_mut();
        *data += 1;
        println!("v4: {:?}", data);
    });

    t2.join().unwrap();
    t1.join().unwrap();

    let mut data = v2.data.borrow_mut();
    *data += 1;

    println!("v2: {:?}", data);
}
```

运行如下，可能会崩溃

```rust
❯ cargo run --example rc_send
v4: 1
v3: 2
v2: 3

❯ cargo run --example rc_send
v4: 1
thread '<unnamed>' panicked at 'already borrowed: BorrowMutError', examples/rc_send.rs:18:32
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Any { .. }', examples/rc_send.rs:31:15
```



 ## 2.2 调用 unsafe 函数

**使用 `unsafe` 关键字声明的函数即为 `unsafe` 函数，一个普通的 `trait` 里可以包含 `unsafe` 函数**。



**unsafe fn ：**是函数对调用者的约束，它告诉函数的调用者要正确使用该函数，如果乱使用会带来内存安全的问题，所以调用 `unsafe fn` 时，需要加 `unsafe`  块把它包裹起来，提醒别人注意这里有 `unsafe` 代码

> 另一种调用 `unsafe` 函数的方法是定义 `unsafe fn`，然后在这个` unsafe fn` 里调用其它的 `unsafe fn`



例如：

```rust
trait Bar {
    // 普通的trait里包含 unsafe 函数，表示调用这个函数的人要保证调用是安全的
    unsafe fn bar(&self);
}

struct Nonsense;

impl Bar for Nonsense {
    unsafe fn bar(&self) {
        println!("bar!");
    }
}

fn main() {
    let nonsense = Nonsense;
 
    // 调用者需要为 安全 负责，使用unsafe block包裹起来
    unsafe { nonsense.bar() };
}
```



**Rust 一些标准库的代码，有时候同样的功能会提供 `unsafe` 和 `safe` 的版本**。比如，[把 &[u8] 里的数据转换成字符串](https://doc.rust-lang.org/src/core/str/converts.rs.html#85-165)：

```rust
// safe 版本，验证合法性，如果不合法返回错误
pub fn from_utf8(v: &[u8]) -> Result<&str, Utf8Error> {
    run_utf8_validation(v)?;
    // SAFETY: Just ran validation.
    Ok(unsafe { from_utf8_unchecked(v) })
}

// unsafe版本，不验证合法性，调用者需要确保 &[u8] 里都是合法的字符
pub const unsafe fn from_utf8_unchecked(v: &[u8]) -> &str {
    // SAFETY: the caller must guarantee that the bytes `v` are valid UTF-8.
    // Also relies on `&str` and `&[u8]` having the same layout.
    unsafe { mem::transmute(v) }
}
```

可以看到，安全的 `str::from_utf8() ` 内部做了一些检查后，实际调用了 `str::from_utf8_unchecked()`；如果不需要做这一层检查，使用 `unsafe` 版本的调用可以高效很多，因为 `unsafe` 的版本就只是一个类型的转换而已。针对这种有两个版本的接口，如果你不是特别明确，一定要调用安全的版本，不要为了性能的优势而去调用不安全的版本，避免出现其他问题。



## 2.3 对裸指针做解引用

裸指针的解引用操作是不安全的，有潜在风险，所以解引用时也需要使用 `unsafe` 来明确告诉编译器，也就是要使用 `unsafe` 块包裹起来。

> 裸指针在生成时无需 `unsafe`，因为它并没有内存不安全的操作



如下代码，是一段对裸指针解引用的操作

```rust
fn main() {
    let mut age = 18;

    // 不可变指针
    let r1 = &age as *const i32;
  
    // 可变指针
    let r2 = &mut age as *mut i32;

    // 使用裸指针，可以绕过 immutable / mutable borrow rule

    // 对指针解引用需要使用 unsafe
    unsafe {
        println!("r1: {}, r2: {}", *r1, *r2);
    }
}

fn immutable_mutable_cant_coexist() {
    let mut age = 18;
    let r1 = &age;
    // 编译错误
    let r2 = &mut age;

    println!("r1: {}, r2: {}", *r1, *r2);
}
```

由代码可知，使用裸指针，**可变指针和不可变指针可以共存**，不像可变引用和不可变引用无法共存。这是因为裸指针的任何对内存的操作，无论是 [ptr::read](https://doc.rust-lang.org/std/ptr/fn.read.html) / [ptr::write](https://doc.rust-lang.org/std/ptr/fn.write.html)，还是解引用，都是 `unsafe` 的操作，所以只要读写内存，裸指针的使用者就需要对内存安全负责。



在上面的例子里，裸指针来源于一个可信的内存地址，所有的代码都是安全的，所以也没有内存不安全的操作。但是，下面的代码就是不安全的，会导致 `segment fault`：

```rust
fn main() {
    // 裸指针指向一个有问题的地址
    let r1 = 0xdeadbeef as *mut u32;

    println!("so far so good!");

    unsafe {
        // 程序崩溃
        *r1 += 1;
        println!("r1: {}", *r1);
    }
}
```

注意使用裸指针时，大部分操作都是 `unsafe` 的，具体可以查阅 [std::ptr 的文档](https://doc.rust-lang.org/std/ptr/index.html)



## 2.4 使用 FFI

最后一种可以使用 `unsafe` 的地方是 `FFI（Foreign Function Interface）`。当 `Rust` 要使用其它语言的能力时（比如 `C/C++` 的库），`Rust` 编译器并不能保证那些语言具备内存安全，所以和第三方语言交互的接口，一律要使用 `unsafe`。



例如，`Rust` 调用 `libc` 的 `malloc / free` 函数时要使用 `unsafe` 包裹

> `libc` 提供了与 `Rust` 支持的各平台上的最基础系统 `C` 库打交道的所有必要设施

```rust
use std::mem::transmute;

fn main() {
    let data = unsafe {
        let p = libc::malloc(8);
        let arr: &mut [u8; 8] = transmute(p);
        arr
    };

    data.copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);

    println!("data: {:?}", data);

    // 使用 unsafe 包裹
    unsafe { libc::free(transmute(data)) };
}
```



# 3 可以使用 unsafe 但不推荐使用的场景

## 3.1 可变静态变量

`Rust` 支持使用 `static mut` 来声明可变静态变量。如果声明了 `static mut` 变量，在访问时都需要使用 `unsafe` 包裹，因为全局静态变量如果可写，会潜在有线程不安全的风险。

> 一般不建议使用 `static mut`。任何需要 `static mut` 的地方，都可以用 `AtomicXXX / Mutex / RwLock` 来取代
>



如下代码：使用了 `static mut`，并试图在两个线程中分别改动它

```rust
use std::thread;

static mut COUNTER: usize = 1;

fn main() {
    let t1 = thread::spawn(move || {
        unsafe { COUNTER += 10 };
    });

    let t2 = thread::spawn(move || {
        unsafe { COUNTER *= 10 };
    });

    t2.join().unwrap();
    t1.join().unwrap();

    unsafe { println!("COUNTER: {}", COUNTER) };
}
```

改进方式1：对于上面的代码，可以使用 `AtomicXXX` 来改进，如下

```rust
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

static COUNTER: AtomicUsize = AtomicUsize::new(1);

fn main() {
    let t1 = thread::spawn(move || {
        COUNTER.fetch_add(10, Ordering::SeqCst);
    });

    let t2 = thread::spawn(move || {
        COUNTER
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| Some(v * 10))
            .unwrap();
    });

    t2.join().unwrap();
    t1.join().unwrap();

    println!("COUNTER: {}", COUNTER.load(Ordering::Relaxed));
}
```

改进方式2：如果无法使用 `AtomicXXX` 来改进，可以使用 `Mutex` 或者 `RwLock` 来提供并发安全的写访问，比如：

```rust
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex, thread};

// 使用 lazy_static 初始化复杂的结构
lazy_static! {
    // 使用 Mutex / RwLock 来提供安全的并发写访问
    static ref STORE: Mutex<HashMap<&'static str, &'static [u8]>> = Mutex::new(HashMap::new());
}

fn main() {
    let t1 = thread::spawn(move || {
        let mut store = STORE.lock().unwrap();
        store.insert("hello", b"world");
    });

    let t2 = thread::spawn(move || {
        let mut store = STORE.lock().unwrap();
        store.insert("goodbye", b"world");
    });

    t2.join().unwrap();
    t1.join().unwrap();

    println!("store: {:?}", STORE.lock().unwrap());
}
```



## 3.2 在宏里使用 unsafe

建议不要在宏中使用 `unsafe`，非常的危险，因为

1. 使用宏的开发者，可能压根不知道 `unsafe` 代码的存在
2. 含有 `unsafe` 代码的宏在被使用到时，相当于把 `unsafe` 代码注入到当前上下文中。在不知情的情况下，开发者到处调用这样的宏，会导致 `unsafe` 代码充斥在系统的各个角落，不好处理
3. 一旦 `unsafe` 代码出现问题，可能很难找到问题的根本原因



## 3.3 使用 unsafe 提升性能

还有一种使用 `unsafe` 纯粹是为了提升性能，比如使用 `unsafe` 略过边界检查、使用未初始化内存等。

这样的 `unsafe` 尽量不要使用，除非通过 `benchmark` 发现用 `unsafe` 可以解决某些性能瓶颈，否则使用起来得不偿失。因为在使用 `unsafe` 代码时，我们已经把 `Rust` 的内存安全性，降低到了和 `C++` 同等的水平。如果你不是在撰写非常基础的库，并且这个库处在系统的关键路径上，也很不建议使用 `unsafe` 来提升性能。



# 4 参考

* [陈天 Rust编程第一课:Unsafe Rust](https://time.geekbang.org/column/article/435484)
* [Rust语言圣经:Unsafe Rust](https://course.rs/advance/unsafe/intro.html)

