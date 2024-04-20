# 1 智能指针

**什么是智能指针**

在 `Rust` 中，凡是需要`做资源回收`的数据结构，且实现了 `Deref / DerefMut / Drop` 特型的，都是智能指针。智能指针是一个表现行为很像指针的数据结构，但除了指向数据的指针外，它还有元数据以提供额外的处理能力。



**指针、引用、智能指针的区别**

* 指针：是一个持有内存地址的值，可以通过`解引用`来访问它指向的内存地址，理论上可以解引用到任意数据类型
* 引用：是一个特殊的指针，它的解引用访问是受限的，只能解引用到它引用数据的类型
* 智能指针：是一个表现行为很像指针的数据结构，但除了指向数据的指针外，它还有`元数据`以提供额外的处理能力



**智能指针和普通胖指针的区别**

智能指针一定是一个胖指针，但胖指针不一定是一个智能指针，胖指针包含一个指向数据的指针和数据的长度信息，在 `Rust` 中，数组、切片、 特型对象等都是胖指针。



例如：`String` 是一个智能指针，而 `&str` 只是一个胖指针，`&str` 有指向堆内存字符串的指针，同时还有关于字符串长度的元数据，`String` 则比 `&str` 多了一个 `capacity` 字段。`String` 对堆上的值有所有权，而 `&str` 是没有所有权的，这是 `Rust` 中智能指针和普通胖指针的区别。如下

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-12%2018.41.21.png)



**智能指针 和 结构体的区别**

智能指针 `String` 是用结构体的形式定义的，定义如下

```rust
pub struct String {
    vec: Vec<u8>,
}
```

* 和普通的结构体不同的是，[String 实现了 Deref 和 DerefMut](https://doc.rust-lang.org/src/alloc/string.rs.html#2454-2470)，**这使得它在解引用时，会得到 `&str`**

```rust
#[stable(feature = "rust1", since = "1.0.0")]
impl ops::Deref for String {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.vec) }
    }
}

#[stable(feature = "derefmut_for_string", since = "1.3.0")]
impl ops::DerefMut for String {
    #[inline]
    fn deref_mut(&mut self) -> &mut str {
        unsafe { str::from_utf8_unchecked_mut(&mut *self.vec) }
    }
}
```

* 由于 `String` 在堆上分配了数据，所以 `String` 还需要为其分配的资源做相应的回收。因为 `String` 内部使用了 `Vec<u8>`，所以它可以依赖 `Vec<T>` 的能力来释放堆内存，标准库中 `Vec<T> ` 通过 [Drop trait](https://doc.rust-lang.org/src/alloc/vec/mod.rs.html#3052-3063) 来释放内存，代码如下：

```rust
unsafe impl<#[may_dangle] T, A: Allocator> Drop for Vec<T, A> {
    fn drop(&mut self) {
        unsafe {
            // use drop for [T]
            // use a raw slice to refer to the elements of the vector as weakest necessary type;
            // could avoid questions of validity in certain cases
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len))
        }
        // RawVec handles deallocation
    }
}
```



# 2 个别常见的智能指针

`Rust` 中有如下常用智能指针

* `String`：字符串类型
* `Box<T> 和 Vec<T>`： 在堆上分配内存
* `Cow<'a, B>`：写时克隆
* `Rc<T> 和 Arc<T>`：引用计数，用于实现共享所有权
* `MutexGuard<T>`、`RwLockReadGuard<T> `、`RwLockWriteGuard<T>`：用于多线程编程的智能指针



## 2.1 `Box<T>`

**什么是 `Box`**

`Box<T>` 是一个智能指针，它可在堆上分配内存，并将值存在堆内存中。绝大多数包含堆内存分配的数据类型，内部都是通过 `Box<T>` 完成的，比如 `Vec<T>`。



[`Box<T>`](https://doc.rust-lang.org/src/alloc/boxed.rs.html#198-201)  的内部有一个 [`Unique<T>`](https://doc.rust-lang.org/src/core/ptr/unique.rs.html#36-44)（包裹了一个 `*const T ` 指针，并唯一拥有这个指针）

```rust
// Box定义
pub struct Box<T: ?Sized,A: Allocator = Global>(Unique<T>, A);

// Unique定义
pub struct Unique<T: ?Sized> {
    pointer: *const T,
    // NOTE: this marker has no consequences for variance, but is necessary
    // for dropck to understand that we logically own a `T`.
    //
    // For details, see:
    // https://github.com/rust-lang/rfcs/blob/master/text/0769-sound-generic-drop.md#phantom-data
    _marker: PhantomData<T>,
}
```



**内存分配器**

在堆上分配内存，需要使用内存分配器（`Allocator`）；内存分配器可以有效地利用剩余内存，并控制内存在分配和释放过程中产生的碎片的数量。



从 `Box` 的定义可以看到 `Box` 有一个缺省的泛型参数 A，它需要满足 [Allocator trait](https://doc.rust-lang.org/std/alloc/trait.Allocator.html)，默认值是 `Global`。`Allocator trait `提供很多方法，如：

* `allocate` ：用于分配内存，对应 `C` 语言的 `malloc/calloc`
* `deallocate`：用于释放内存，对应 `C` 语言的` free`
* `grow / shrink`：用来扩大或缩小堆上已分配的内存，对应 `C` 语言 的 `realloc`



**修改默认的全局分配器**

可以使用 `#[global_allocator]` 标记宏来定义自己的全局分配器，用于替换默认的内存分配器。



例如，使用 [jemalloc](https://crates.io/crates/jemallocator) 内存分配器来分配内存，如下这样设置之后，使用 `Box::new()` 分配的内存就是 `jemalloc`分配出来的了

```rust
use jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {}
```

如果想使用自己写的全局分配器，可以实现 [GlobalAlloc trait](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html)，它和 `Allocator trait` 的区别，主要在于是否允许分配长度为零的内存。



**Box 的内存的分配 和 释放 **

接下来使用 `Box` 实现一个内存分配器（这里主要是看看内存如何分配和释放，没有实际实现某个分配算法）

1、首先看 `Box<T>`内存的分配

定义自己的内存分配器 `MyAllocator`，并实现 `GlobalAlloc trait`。

> 注意：这里不能使用 `println!() `。因为 `stdout` 会打印到一个由 `Mutex` 互斥锁保护的共享全局 `buffer` 中，这个过程中会涉及内存的分配，分配的内存又会触发 `println!()`，最终造成程序崩溃。而是使用 `eprintln!` ，`eprintln!` 直接打印到 `stderr`，不会 `buffer`。代码可见[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/smart-point/src/allocator.rs)

```rust
use std::alloc::{GlobalAlloc, Layout, System};

struct MyAllocator;

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let data = System.alloc(layout);
        eprintln!("ALLOC: {:p}, size {}", data, layout.size());
        data
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        eprintln!("FREE: {:p}, size {}", ptr, layout.size());
    }
}

#[global_allocator]
static GLOBAL: MyAllocator = MyAllocator;

#[allow(dead_code)]
struct Matrix {
    // 使用不规则的数字如 505 可以让 dbg! 的打印很容易分辨出来
    data: [u8; 505],
}

impl Default for Matrix {
    fn default() -> Self {
        Self { data: [0; 505] }
    }
}

fn main() {
    // 在这句执行之前已经有好多内存分配
    let data = Box::new(Matrix::default());
  
    // 输出中有一个 1024 大小的内存分配，是 println! 导致的    
    println!(
        "!!! allocated memory: {:p}, len: {}",
        &*data,
        std::mem::size_of::<Matrix>()
    );

    // data 在这里 drop，可以在打印中看到 FREE
    // 之后还有很多其它内存被释放
}
```

运行`cargo run --bin allocator`，可以看到如下输出，其中 505 大小的内存是 `Box::new() `出来的

```bash
ALLOC: 0x7fbe0dc05c20, size 4
ALLOC: 0x7fbe0dc05c30, size 5
FREE: 0x7fbe0dc05c20, size 4
ALLOC: 0x7fbe0dc05c40, size 64
ALLOC: 0x7fbe0dc05c80, size 48
ALLOC: 0x7fbe0dc05cb0, size 80
ALLOC: 0x7fbe0dc05da0, size 24
ALLOC: 0x7fbe0dc05dc0, size 64
ALLOC: 0x7fbe0dc05e00, size 505
ALLOC: 0x7fbe0e008800, size 1024
!!! allocated memory: 0x7fbe0dc05e00, len: 505
FREE: 0x7fbe0dc05e00, size 505
FREE: 0x7fbe0e008800, size 1024
FREE: 0x7fbe0dc05c30, size 5
FREE: 0x7fbe0dc05c40, size 64
FREE: 0x7fbe0dc05c80, size 48
FREE: 0x7fbe0dc05cb0, size 80
FREE: 0x7fbe0dc05dc0, size 64
FREE: 0x7fbe0dc05da0, size 24
```

**在使用 `Box` 分配堆内存时要注意，`Box::new() `是一个函数，所以传入它的数据会出现在栈上，再移动到堆上。**

如果代码里的 `Matrix` 结构不是 505 个字节，是一个非常大的结构，就有可能出问题；比如下面的代码在堆上分配 16M 内存，如果你在 [playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c824fba820015db5c39d4cd700716c16) 里运行，直接栈溢出

```rust
fn main() {
    // 在堆上分配 16M 内存，但它会现在栈上出现，再移动到堆上
    let boxed = Box::new([0u8; 1 << 24]);
    println!("len: {}", boxed.len());
}
```

如果在本地使用` cargo run —release `编译成 `release` 代码运行，会正常执行，不会出现栈溢出。因为 `cargo run` 或者在 `playground` 下运行，默认是 `debug build`，它不会做任何 `inline` 的优化，而 ` Box::new()` 的实现就一行代码，并注明了要 `inline`，在 `release` 模式下，这个函数调用会被优化掉，如果不 `inline`，整个 16M 的大数组会通过栈内存传递给 `Box::new`，导致栈溢出

```rust
#[cfg(not(no_global_oom_handling))]
#[inline(always)]
#[doc(alias = "alloc")]
#[doc(alias = "malloc")]
#[stable(feature = "rust1", since = "1.0.0")]
pub fn new(x: T) -> Self {
    box x // 关键字
}
```

这里的 `box` 是 `Rust` 内部的关键字，用户代码无法调用，它只出现在 `Rust` 代码中，用于分配堆内存，`box` 关键字在编译时，会使用内存分配器分配内存。



2、再看下`Box<T>` 内存是如何释放的

来看一下 [Box 实现的 Drop trait](https://doc.rust-lang.org/src/alloc/boxed.rs.html#1235-1239)：

```rust
#[stable(feature = "rust1", since = "1.0.0")]
unsafe impl<#[may_dangle] T: ?Sized, A: Allocator> Drop for Box<T, A> {
    fn drop(&mut self) {
        // FIXME: Do nothing, drop is currently performed by compiler.
    }
}
```

目前 `drop trait` 什么都没有做，编译器会自动插入 `deallocate` 的代码。



## 2.2 `Cow<'a, B>`

**什么是 Cow**

`Cow` 是用于提供写时克隆（`Clone-on-Write`）的一个智能指针，它跟虚拟内存管理的写时复制（`Copy-on-write`）有点类似：包裹一个只读借用，但如果调用者需要所有权或者需要修改内容，那么它会 `clone` 借用的数据。



`Cow` 是一个枚举，它可以包含一个对类型 `B` 的只读引用，或者包含对类型 `B` 的拥有所有权的数据，它的定义如下

```rust
pub enum Cow<'a, B> where B: 'a + ToOwned + ?Sized {
  Borrowed(&'a B),
  Owned(<B as ToOwned>::Owned),
}
```

这里引入了一个 `ToOwned trait`，在 `ToOwned trait` 定义中，又引入了一个 `Borrow trait`，`ToOwned trait` 和  `Borrow trait` 都是 [std::borrow](https://doc.rust-lang.org/std/borrow/index.html) 下的 `trait`，它们的定义如下：

```rust
pub trait ToOwned {
    type Owned: Borrow<Self>;
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    fn to_owned(&self) -> Self::Owned;

    fn clone_into(&self, target: &mut Self::Owned) { ... }
}

pub trait Borrow<Borrowed> where Borrowed: ?Sized {
    fn borrow(&self) -> &Borrowed;
}
```

* `type Owned: Borrow<Self>`：说明 `ToOwned trait` 有一个关联类型 `Owned`，它需要使用者去定义，注意这里 `Owned` 不能是任意类型，它必须满足 `Borrow trait`。例如， [str 对 ToOwned trait 的实现](https://doc.rust-lang.org/src/alloc/str.rs.html#203-217)

```rust
impl ToOwned for str {
    type Owned = String;
    #[inline]
    fn to_owned(&self) -> String {
        unsafe { String::from_utf8_unchecked(self.as_bytes().to_owned()) }
    }

    fn clone_into(&self, target: &mut String) {
        let mut b = mem::take(target).into_bytes();
        self.as_bytes().clone_into(&mut b);
        *target = unsafe { String::from_utf8_unchecked(b) }
    }
}
```

可以看到这里关联类型 `Owned` 被定义为 `String`。因为实现 `ToOwned` 的主体是 `str`，所以 `Borrow<Self>` 是` Borrow<str>`，也就是说 [String 是实现了 `Borrow<str>`](ttps://doc.rust-lang.org/src/alloc/str.rs.html#188-193)：

```rust
impl Borrow<str> for String {
    #[inline]
    fn borrow(&self) -> &str {
        &self[..]
    }
}
```

此时它们的关系如下图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-12%2019.23.23.png)





**Borrow 定义成了一个泛型 trait **

因为一个类型可以被借用成不同的引用；如下，`String` 可以被借用为 `&String`，也可以被借用为 `&str`

```rust
use std::borrow::Borrow;

fn main() {
    let s = "hello world!".to_owned();

    // 这里必须声明类型，因为 String 有多个 Borrow<T> 实现
    // 借用为 &String
    let r1: &String = s.borrow();
    // 借用为 &str
    let r2: &str = s.borrow();

    println!("r1: {:p}, r2: {:p}", r1, r2);
}
```



**Cow 的 Deref 实现**

`Cow` 是智能指针，那它自然需要实现 [Deref trait](https://doc.rust-lang.org/src/alloc/borrow.rs.html#332-344)：

```rust
impl<B: ?Sized + ToOwned> Deref for Cow<'_, B> {
    type Target = B;

    fn deref(&self) -> &B {
        match *self {
            Borrowed(borrowed) => borrowed,
            Owned(ref owned) => owned.borrow(),
        }
    }
}
```

实现的原理很简单，根据 `self` 是 `Borrowed` 还是 `Owned`，我们分别取其内容，生成引用：

* 对于 `Borrowed`，直接就是引用
* 对于 `Owned`，调用其 `borrow() `方法，获得引用



虽然 `Cow` 是一个 `enum`，但是通过 `Deref` 的实现，我们可以获得统一的体验，比如 `Cow`，使用的感觉和 `&str / String` 是基本一致的。注意，这种根据 `enum` 的不同状态来进行统一分发的方法是第三种分发手段（前两种是可以使用泛型参数做静态分发和使用 `trait object` 做动态分发）。



**Cow 的作用和使用场景**

* `Cow` 可以在需要时才进行内存的分配和拷贝，在很多应用场合，它可以大大提升系统的效率

* 如果 `Cow<'a, B>` 中的 `Owned` 数据类型是一个需要在堆上分配内存的类型，如 `String、Vec` 等，还能减少堆内存分配的次数

  > 因为相对于栈内存的分配释放来说，堆内存的分配和释放效率要低很多，其内部还涉及系统调用和锁，减少不必要的堆内存分配是提升系统效率的关键手段。



需求场景：在解析 `URL` 时，经常需要将 `querystring` 中的参数，提取成 `KV pair` 来进一步使用。绝大多数语言中，提取出来的 `KV` 都是新的字符串，在每秒钟处理几十 k 甚至上百 k 请求的系统中，这会导致带来了很多次堆内存的分配。

解决方案：在 `Rust` 中，可以用 `Cow` 类型轻松高效处理它，在读取 URL 的过程中：

* 每解析出一个 `key` 或者 `value`，可以用一个 `&str` 指向 `URL` 中相应的位置，然后用 `Cow` 封装它
* 而当解析出来的内容不能直接使用，需要 `decode` 时，比如 “hello%20world”，可以生成一个解析后的 `String`，同样用 `Cow` 封装它

> 代码可见[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/smart-point/src/cow1.rs)

```rust
use std::borrow::Cow;

use url::Url;

fn main() {
    let url = Url::parse("https://tyr.com/rust?page=1024&sort=desc&extra=hello%20world").unwrap();
    let mut pairs = url.query_pairs();

    assert_eq!(pairs.count(), 3);

    let (mut k, v) = pairs.next().unwrap();
    // 因为 k, v 都是 Cow<str> 他们用起来感觉和 &str 或者 String 一样
    // 此刻，他们都是 Borrowed
    println!("key: {}, v: {}", k, v);
    // 当修改发生时，k 变成 Owned
    k.to_mut().push_str("_lala");

    print_pairs((k, v));

    print_pairs(pairs.next().unwrap());
    print_pairs(pairs.next().unwrap());
}

fn print_pairs(pair: (Cow<str>, Cow<str>)) {
    println!("key: {}, value: {}", show_cow(pair.0), show_cow(pair.1));
}

fn show_cow(cow: Cow<str>) -> String {
    match cow {
        Cow::Borrowed(v) => format!("Borrowed {}", v),
        Cow::Owned(v) => format!("Owned {}", v),
    }
}
```

类似 `URL parse` 这样的处理方式，在 `Rust` 标准库和第三方库中很常见。比如 [serde](https://serde.rs/) 库，可以非常高效地对 `Rust` 数据结构，进行序列化 / 反序列化操作，它对 `Cow` 就有很好的支持。例如下面的代码将一个 `JSON` 数据反序列化成 `User` 类型，同时让 `User` 中的 `name` 使用 `Cow` 来引用 `JSON` 文本中的内容

> 代码可见[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/smart-point/src/cow2.rs)

```rust
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Debug, Deserialize)]
struct User<'input> {
    #[serde(borrow)]
    name: Cow<'input, str>,
    age: u8,
}

fn main() {
    let input = r#"{ "name": "zhang san", "age": 18 }"#;
    let user: User = serde_json::from_str(input).unwrap();

    match user.name {
        Cow::Borrowed(x) => println!("borrowed {}", x),
        Cow::Owned(x) => println!("owned {}", x),
    }
  
    println!("age {}", user.age)
}

```



## 2.3 `MutexGuard<T>`

`MutexGuard<T>` 也是智能指针，它可以通过 `Deref` 来提供良好的用户体验，还通过 `Drop trait` 来确保使用到的内存以外的资源在退出时进行释放。



`MutexGuard` 这个结构是在调用 [Mutex::lock](https://doc.rust-lang.org/src/std/sync/mutex.rs.html#279-284) 时生成的：

```rust
pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
    unsafe {
        self.inner.raw_lock();
        MutexGuard::new(self)
    }
}
```

首先，它会取得锁资源，如果拿不到，会在这里等待；如果拿到了，会把 `Mutex` 结构的引用传递给 `MutexGuard`。



 [MutexGuard 的定义](https://doc.rust-lang.org/src/std/sync/mutex.rs.html#190-195) 以及它的 `Deref` 和 `Drop` 的[实现](https://doc.rust-lang.org/src/std/sync/mutex.rs.html#462-487)：

```rust
// 这里用 must_use，当你得到了却不使用 MutexGuard 时会报警
#[must_use = "if unused the Mutex will immediately unlock"]
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
    poison: poison::Guard,
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.lock.poison.done(&self.poison);
            self.lock.inner.raw_unlock();
        }
    }
}
```

当 `MutexGuard` 结束时，`Mutex` 会做 `unlock`，这样用户在使用 `Mutex` 时，可以不必关心何时释放这个互斥锁。因为无论你在调用栈上怎样传递 `MutexGuard` ，哪怕在错误处理流程上提前退出，`Rust` 有所有权机制，可以确保只要 `MutexGuard` 离开作用域，锁就会被释放。



`MutexGuard` 不允许 `Send`，只允许 `Sync`，也就是可以把 `MutexGuard` 的引用传给另一个线程使用，但无法把 `MutexGuard` 整个移动到另一个线程。



**例子**

下面来看一个使用 `Mutex` 和 `MutexGuard` 的例子，代码可见[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/smart-point/src/guard.rs)

```rust
use lazy_static::lazy_static;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// lazy_static 宏可以生成复杂的 static 对象
lazy_static! {
    // 一般情况下 Mutex 和 Arc 一起在多线程环境下提供对共享内存的使用
    // 如果把 Mutex 声明成 static，其生命周期是静态的，不需要 Arc
    static ref METRICS: Mutex<HashMap<Cow<'static, str>, usize>> =
        Mutex::new(HashMap::new());
}

fn main() {
    // 用 Arc 来提供并发环境下的共享所有权（使用引用计数）
    let metrics: Arc<Mutex<HashMap<Cow<'static, str>, usize>>> =
        Arc::new(Mutex::new(HashMap::new()));
  
    for _ in 0..32 {
        let m = metrics.clone();
      
        thread::spawn(move || {
            let mut g = m.lock().unwrap();
            // 此时只有拿到 MutexGuard 的线程可以访问 HashMap
            let data = &mut *g;
            // Cow 实现了很多数据结构的 From trait，
            // 所以我们可以用 "hello".into() 生成 Cow
            let entry = data.entry("hello".into()).or_insert(0);
            *entry += 1;
            // MutexGuard 被 Drop，锁被释放
        });
    }

    thread::sleep(Duration::from_millis(100));

    println!("metrics: {:?}", metrics.lock().unwrap());
}
```

类似 `MutexGuard` 的智能指针有很多用途。比如要创建一个连接池，你可以在 `Drop trait` 中，回收 `checkout `出来的连接，将其再放回连接池。具体可以看看 [r2d2](https://github.com/sfackler/r2d2/blob/master/src/lib.rs#L611) 的实现，它是 `Rust` 下一个数据库连接池的实现。



## 2.4 Rc 和 Arc

`Rc` 和 `Arc` 是引用计数智能指针，用于实现共享所有权。具体可见之前的[另一篇文章](https://zhuanlan.zhihu.com/p/603465225)



# 3 实现自己的智能指针

`Rust` 下 `String` 在栈上占了 24 个字节，然后在堆上存放字符串实际的内容，对于一些比较短的字符串，这很浪费内存。有没有办法在字符串长到一定程度后，才使用标准的字符串呢？



参考 `Cow`，可以用一个 `enum` 来处理：当字符串小于 N 字节时，直接用栈上的数组，否则，使用 `String`。但是这个 N 不宜太大，否则当使用 `String` 时，会比目前的版本浪费内存。



当使用 enum 时，`额外的 tag` + `为了对齐而使用的 padding` 会占用一些内存。因为 String 结构是 8 字节对齐的，我们的 enum 最小 8 + 24 = 32 个字节。



所以，可以设计一个数据结构，内部用一个字节表示字符串的长度，用 30 个字节表示字符串内容，再加上 1 个字节的 tag，正好也是 32 字节，可以和 String 放在一个 enum 里使用。暂且称这个 enum 叫 MyString，它的结构如下图所示：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-03-12%2019.26.25.png)



为了让 MyString 表现行为和 &str 一致，可以通过实现 Deref trait 让 MyString 可以被解引用成 &str。除此之外，还可以实现 Debug/Display 和 `From<T> trait`，让 MyString 使用起来更方便。



实现代码如下:

> 代码可见[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/smart-point/src/mystring.rs)

```rust
use std::{fmt, ops::Deref, str};

const MINI_STRING_MAX_LEN: usize = 30;

// MyString 里，String 有 3 个 word，供 24 字节，所以它以 8 字节对齐
// 所以 enum 的 tag + padding 最少 8 字节，整个结构占 32 字节
// MiniString 可以最多有 30 字节（再加上 1 字节长度和 1字节 tag），就是 32 字节
struct MiniString {
    len: u8,
    data: [u8; MINI_STRING_MAX_LEN],
}

impl MiniString {
    // 这里 new 接口不暴露出去，保证传入的 v 的字节长度小于等于 30
    fn new(v: impl AsRef<str>) -> Self {
        let bytes = v.as_ref().as_bytes();
      
        // 在拷贝内容时一定要使用字符串的字节长度
        let len = bytes.len();
        let mut data = [0u8; MINI_STRING_MAX_LEN];
        data[..len].copy_from_slice(bytes);
        Self {
            len: len as u8,
            data,
        }
    }
}

impl Deref for MiniString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // 由于生成 MiniString 的接口是隐藏的，它只能来自字符串，所以下面这行是安全的
        str::from_utf8(&self.data[..self.len as usize]).unwrap()
        // 也可以直接用 unsafe 版本
        // unsafe { str::from_utf8_unchecked(&self.data[..self.len as usize]) }
    }
}

impl fmt::Debug for MiniString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 这里由于实现了 Deref trait，可以直接得到一个 &str 输出
        write!(f, "{}", self.deref())
    }
}

#[derive(Debug)]
enum MyString {
    Inline(MiniString),
    Standard(String),
}

// 实现 Deref 接口对两种不同的场景统一得到 &str
impl Deref for MyString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match *self {
            MyString::Inline(ref v) => v.deref(),
            MyString::Standard(ref v) => v.deref(),
        }
    }
}

impl From<&str> for MyString {
    fn from(s: &str) -> Self {
        match s.len() > MINI_STRING_MAX_LEN {
            true => Self::Standard(s.to_owned()),
            _ => Self::Inline(MiniString::new(s)),
        }
    }
}

impl fmt::Display for MyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.deref())
    }
}

fn main() {
    let len1 = std::mem::size_of::<MyString>();
    let len2 = std::mem::size_of::<MiniString>();
    println!("Len: MyString {}, MiniString {}", len1, len2);

    let s1: MyString = "hello world".into();
    let s2: MyString = "这是一个超过了三十个字节的很长很长的字符串".into();

    // debug 输出
    println!("s1: {:?}, s2: {:?}", s1, s2);
  
    // display 输出
    println!(
        "s1: {}({} bytes, {} chars), s2: {}({} bytes, {} chars)",
        s1,
        s1.len(),
        s1.chars().count(),
        s2,
        s2.len(),
        s2.chars().count()
    );

    // MyString 可以使用一切 &str 接口，感谢 Rust 的自动 Deref
    assert!(s1.ends_with("world"));
    assert!(s2.starts_with("这"));
}
```

这个简单实现的 MyString，不管它内部的数据是纯栈上的 MiniString 版本，还是包含堆上内存的 String 版本，使用的体验和 &str 都一致，仅仅牺牲了一点点效率和内存，就可以让小容量的字符串，可以高效地存储在栈上并且自如地使用。

> Rust 有个叫 [smartstring](https://github.com/bodil/smartstring) 的第三方库就实现了这个功能。我们的版本在内存上不算经济，对于 String 来说，额外多用了 8 个字节，smartstring 通过优化，只用了和 String 结构一样大小的 24 个字节，就达到了我们想要的结果。



# 4 参考

* [陈天 · Rust 编程第一课-智能指针](https://time.geekbang.org/column/article/422182)

