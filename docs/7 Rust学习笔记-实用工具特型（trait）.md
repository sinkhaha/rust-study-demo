# 1 内存相关:Clone / Copy / Drop

> `Rust` 中的 `trait` 可以简单理解为其他语言中的接口

## 1.1 Clone trait

[Clone trait](https://doc.rust-lang.org/std/clone/trait.Clone.html) ：定义了数据被克隆（深拷贝）的行为，深拷贝时栈内存和堆内存会一起拷贝。克隆一个值会为它拥有的任何值复制一份副本，所以克隆是一个昂贵的动作（时间消耗和内存占用）。



**Clone 定义如下**

```rust
pub trait Clone: Sized {
    fn clone(&self) -> Self; // 没缺省实现
    fn clone_from(&mut self, source: &Self) { // 有缺省实现
        *self = source.clone()
    } 
}
```

* `clone()` 方法的参数是 `&self`，所以在 `clone` 一个数据时只需要有已有数据的只读引用。但对 `Rc<T>` 这样在 `clone()` 时维护引用计数的数据结构，在 `clone()` 过程中会改变自己，所以要用 `Cell<T>` 这样提供内部可变性的结构来进行改变
* `clone_from()` 方法会把 `self` 修改成 `source` 的副本，它的默认实现是只克隆 `source`，然后将其转移给 `*self`。例如代码 `a.clone_from(&b)` 和 ` a = b.clone() ` 看起来是等价的，其实不是，如果 `a` 已经存在，在 `clone()` 过程中会分配内存，如果使用 `a.clone_from(&b) ` 可以避免内存分配，提高效率



**自动实现 Clone**

数据结构要实现 `Clone`，通过在数据结构定义上加上派生宏 `#[derive(Clone)]`  即可，**前提是数据结构的每一个字段都已经实现了 `Clone`。**



**例子**

如下代码，定义 `Developer` 结构和 `Language` 枚举，并为它们实现` Clone`

```rust
#[derive(Clone, Debug)]
struct Developer {
  name: String,
  age: u8,
  lang: Language
}

// 如果没有为 Language 实现 Clone 的话，Developer 的派生宏 Clone 将会编译出错
#[allow(dead_code)] // 避免未使用的代码告警提示
#[derive(Clone, Debug)]
enum Language {
  Rust,
  TypeScript,
  Elixir,
  Haskell
}

fn main() {
    let dev = Developer {
        name: "Tyr".to_string(),
        age: 18,
        lang: Language::Rust
    };
    let dev1 = dev.clone();
    println!("dev: {:?}, addr of dev name: {:p}", dev, dev.name.as_str());
    println!("dev1: {:?}, addr of dev1 name: {:p}", dev1, dev1.name.as_str())
}
```

因为是深拷贝，所以对于 `String` 类型的 `name`，其堆上的内存也被 `Clone` 了一份。



## 1.2 Copy trait

[Copy trait](https://doc.rust-lang.org/std/marker/trait.Copy.html) ：定义了数据被浅拷贝的行为，`Copy trait` 没有任何方法，它只是一个标记特型`（marker trait）`



**Copy 定义如下**

```rust
pub trait Copy: Clone {}
```



**自动实现 Copy**

和 `Clone` 一样，也可以用派生宏 `#[derive(Copy)] ` 来为数据结构实现 `Copy`，前提是数据结构的所有字段都已经实现了 `Copy`。因为 `Copy` 是 `Clone` 的子特型，所以要实现 `Copy`，**还必须同时实现 `Clone`**，即添加 `#[derive(Clone, Copy)]` 属性

> 注意：任何实现了 `Drop` 特型的类型都不是 `Copy` 类型。



**复制 和 移动 语义**

1. 如果类型实现了 `Copy`，那么在赋值、函数调用的时候，值会被浅拷贝，执行复制的语义；如果类型没实现 `Copy` ，则所有权会被移动，执行 `Move` 移动的语义（所有权相关的知识可见另一篇文章）

2. 不可变引用 `&T` 都实现了 `Copy`，而可变引用 `&mut T` 没有实现 `Copy`。因为如果可变引用实现了 `Copy`，那么生成一个可变引用然后把它赋值给另一个变量时，就会违背所有权规则：同一个作用域下只能有一个可变引用




**例子**

如下代码，为 `Developer` 结构体和 `Language` 枚举自动实现 `Copy` 宏时，编译会出错。因为 `String` 类型没有实现 `Copy`，所以 `Developer` 数据结构只能实现 `Clone`，无法实现 `Copy`

```rust
#[derive(Clone, Copy, Debug)]
struct Developer {
  name: String,
  age: u8,
  lang: Language
}

#[derive(Clone, Copy, Debug)]
enum Language {
  Rust,
  TypeScript,
  Elixir,
  Haskell
}
```



## 1.3 Drop trait

[Drop trait](https://doc.rust-lang.org/std/ops/trait.Drop.html) ：可以理解为是一个析构器。每当 `Rust` 丢弃一个值时，`Rust` 会自动运行清理代码处理丢弃的工作（如释放该值拥有的任何其他值、堆存储和系统资源等）。丢弃默认调用的就是 `Drop` 特型的 `drop` 方法。

> 会发生丢弃的情况：例如当变量超出作用域时、在表达式语句的末尾、当截断一个向量时会从其尾移除元素、当一个值的拥有者消失时
>



**Drop 定义如下**

```rust
pub trait Drop {
    fn drop(&mut self);
}
```

大部分场景无需为数据结构提供 `Drop` ，**系统默认会依次对数据结构的每个字段做 `drop`**，`Rust` 不允许显式地调用析构函数 `drop`。



如果一个变量的值移动到了别处，导致该变量在超出作用域时正处于未初始化状态，那么 `Rust` 将不会试图丢弃该变量，因为这里没有需要丢弃的值了。



**例子**

`Appellation` 结构体拥有字符串内容，拥有向量缓冲区的堆存储。每当 `Appellation` 被丢弃时，`Rust` 会都自动清理这些内容

```rust
struct Appellation {
  name: String,
  nicknames: Vec<String>
}
```

我们也可以自定义这个清理逻辑，只要实现 `std::ops::Drop` 特型的 `drop` 方法即可，如

```rust
// Appellation实现Drop的drop方法
impl Drop from Appellation {
  fn drop(&mut self) {
    print!("dropping {}", self.name); // 这里仍然可以使用name，还未被丢弃
    if !self.nicknames.is_empty() {
       print!("{}", self.nicknames.join(","));
    }
    println!("");
  }
}

let mut a = Appellation {
  name: "zhangsan".to_string(),
  nicknames: vec!["cloud".to_string(), "king".to_string]
};

println!("before assignment");

a = Appellation { // 这里重新赋值给a，所以会丢弃前面的 Appellation
  name: "lisi".to_string(),
  nicknames: vec!["hera".to_string()]
};

println!("end of block"); // 当这里执行结束后，会丢弃第二个Appellation
```

* 因为`Vec` 实现了 `Drop`，它会丢弃每一个元素，释放它们占用的分配在堆上的缓冲区
* `String` 在内部使用了 `Vec<u8>` 来保存它的文本，所以 `String` 不需要自己实现 `Drop`，它会让 `Vec` 负责释放这些字符
* `Rust` 在丢弃某个值的字段 或 元素之前会先对值本身调用 `drop` ，在调用 `drop` 时，`drop` 方法里收到的值仍然是已经初始化的



**一般要手动实现 `Drop` 的2种情况**

1. 在数据结束生命周期时做一些事情，比如记录日志

```rust
#[derive(Debug)]
struct CustomData {
  name: String,
}

// 实现drop，记录日志
impl Drop for CustomData {
  fn drop(&mut self) {
    println!("记录日志{}", self.name);
  }
}

fn main() {
    let a = CustomData { name: String::from("a") };
    let b = CustomData { name: String::from("b") };
  
    // 在a和b的作用域结束后，会自动调用drop方法，调用顺序是根据变量声明顺序，后进先出，类似执行以下语句
    // b.drop();
    // a.drop();
}

// 运行后输入如下：
// 记录日志b
// 记录日志a
```

2. 需要对资源回收的场景。编译器并不知道我们代码里额外使用了哪些资源，也就无法帮助 `drop` 它们。比如锁资源的释放，可以在 `MutexGuard` 中实现了 `Drop` 来释放锁资源

```rust
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



**互斥的 Drop 和 Copy**

`Copy trait` 和 `Drop trait` 是互斥的，无法为同一种数据类型同时实现 `Copy` 和 `Drop`，否则编译器会报错。

> 因为 `Copy` 是按位做浅拷贝，那么它会默认拷贝的数据没有需要释放的资源；而 `Drop` 恰恰是为了释放额外的资源而生的

例子：此例子只是一个演示例子，此时没有处理内存泄漏的问题

```rust
use std::{fmt, slice};

// 注意这里实现了Copy，是因为 *mut u8/usize 都支持 Copy
#[derive(Clone, Copy)]
struct RawBuffer {
    // 裸指针用 *const / *mut 来表述，这和引用的 & 不同
    ptr: *mut u8,
    len: usize,
}

impl From<Vec<u8>> for RawBuffer {
    fn from(vec: Vec<u8>) -> Self {
        let slice = vec.into_boxed_slice();
        Self {
            len: slice.len(),
            // into_raw 之后，Box 就不管这块内存的释放了，RawBuffer 需要处理释放
            ptr: Box::into_raw(slice) as *mut u8,
        }
    }
}

// 如果 RawBuffer 实现了 Drop trait，就可以在所有者退出时释放堆内存
// 然后，Drop trait 会跟 Copy trait 冲突，要么不实现 Copy，要么不实现 Drop
// 如果不实现 Drop，那么就会导致内存泄漏，但它不会对正确性有任何破坏，比如不会出现 use after free 这样的问题。
// 如果把下面代码的注释去掉，则会编译不通过
// impl Drop for RawBuffer {
//     #[inline]
//     fn drop(&mut self) {
//         let data = unsafe { Box::from_raw(slice::from_raw_parts_mut(self.ptr, self.len)) };
//         drop(data)
//     }
// }

impl fmt::Debug for RawBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.as_ref();
        write!(f, "{:p}: {:?}", self.ptr, data)
    }
}

impl AsRef<[u8]> for RawBuffer {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

fn main() {
    let data = vec![1, 2, 3, 4];

    let buf: RawBuffer = data.into();

    // 因为 buf 允许 Copy，所以这里 Copy 了一份
    use_buffer(buf);

    // buf 还能用
    println!("buf: {:?}", buf);
}

fn use_buffer(buf: RawBuffer) {
    println!("buf to die: {:?}", buf);

    // 这里不用特意 drop，写出来只是为了说明 Copy 出来的 buf 被 Drop 了
    drop(buf)
}
```

在代码中，强行用 `Box::into_raw` 获得堆内存的指针，放入 `RawBuffer` 结构中，这样就接管了这块堆内存的释放。`RawBuffer` 实现了 `Copy trait`，所以无法实现 `Drop trait`，最终会导致内存泄漏，因为该释放的堆内存没有释放。但是这个操作不会破坏 `Rust` 的正确性保证：即便你 `Copy` 了 `N` 份 `RawBuffer`，由于无法实现 `Drop trait`，`RawBuffer` 指向的那同一块堆内存不会释放，所以不会出现 `use after free` 的内存安全问题。



实际上，任何编程语言都无法保证不发生人为的内存泄漏，比如程序在运行时，开发者疏忽了，对哈希表只添加不删除，就会造成内存泄漏。但 `Rust` 会保证即使开发者疏忽了内存泄漏，也不会出现内存安全问题。



# 2 标记:Sized / Send / Sync

## 2.1 Sized trait

[Sized trait](https://doc.rust-lang.org/std/marker/trait.Sized.html) ：它也是一个标记特型，没有任何方法，用于标记有`固定大小`的类型（与之对应的是动态大小的类型，如切片）。



**标记特型的作用**

可以用来进行类型安全检查。多用于泛型类型变量的限界，作为约束条件等。



**Sized 定义如下**

```rust
pub trait Sized { }
```

所有固定大小类型都实现了 `std::marker::Sized` 特型，该特型没有方法或关联类型。



**Sized 的作用**

`Sized` 的唯一用途是作为类型变量的限界：像 `T: Sized` 这样的限界要求 `T` 必须是在编译期已知的固定大小的类型

* `Rust` 自动为所有适用的类型实现了 `Sized`  特型，我们不能自己实现它

* 在使用泛型参数时，`Rust` 编译器默认会自动为泛型参数加上 `Sized` 约束



**固定大小类型**

固定大小类型指其每个值在内存中都有相同大小的类型。`Rust` 中的几乎所有类型都是固定大小的

* 比如每个 `u64` 占用 8 个字节
* 每个 `(f32， f32,  f32)` 元组占用12个字节
* 甚至枚举都有大小（无论实际存的是哪个变体，枚举总会占足够的空间来容纳其最大的变体）
* `Vec<T>` 也是一个固定大小的类型，虽然拥有一个大小可变的堆分配缓冲区，但是 `Vec` 本身其实是一个包含缓冲区指针、容量和长度的结构体



**例子**

```rust
struct Data<T> {
    inner: T,
}

fn process_data<T>(data: Data<T>) {
    todo!();
}
```

它等价于：

```rust
struct Data<T: Sized> { // 约束 T 为固定大小的类型
    inner: T,
}

fn process_data<T: Sized>(data: Data<T>) {
    todo!();
}
```

这样添加约束后定义出的泛型结构，在编译期，大小是固定的，可以作为参数传递给函数。如果没有这个约束，`T`  是大小不固定的类型， `process_data` 函数会无法编译。

在少数情况下，`T` 是可变类型 ，此时自动添加的 `Sized` 约束不适用，需要用 `?Sized` 来摆脱这个约束。



**?Sized 的作用**

`?Sized` 表示任意大小约束，如果显式定义了 `T: ?Sized`，那么 `T` 可以是任意大小的，固不固定大小都可以。



`Rust` 不能将无固定大小的值存储在变量中 或 将它们作为参数传递。只能通过 `&str` 或 `Box<dyn Write>` 这样本身是固定大小的指针来处理它们。因为无固定大小类型处处受限，所以大多数泛型类型变量应该被限制为固定大小的 `Sized` 类型。`Sized` 已经成为`Rust` 的隐式默认值，如

```rust
struct S<T> { ... } 
// 实际 rust 理解为
struct S<T: Sized> { ... }
```

如果不想这种约束，必须显示的标明（在 `Sized` 前加一个问号），`?Sized` 表示不要求固定大小，固不固定都可以

```rust
struct S<T: ?Sized> { ... }
```



例如： `Cow` 枚举中泛型参数 `B` 的约束是 ` ?Sized`

```rust
pub enum Cow<'a, B: ?Sized + 'a> where B: ToOwned,
{
    // 借用的数据
    Borrowed(&'a B),
    // 拥有的数据
    Owned(<B as ToOwned>::Owned),
}
```

此时 `B` 的大小是不固定的。要注意 `Borrowed(&'a B) ` 大小是固定的，因为它内部是对 `B` 的一个引用，而引用的大小是固定的。



**无固定大小的类型**

`Rust` 也有一些无固定大小类型：它们的值大小不尽相同，以下介绍 4。种

1. 字符串切片类型 `str` （注意没有 `&` ）

2. 像 `[T]` （同样没有 `&`） 这样的数组切片类型

   > 因为 `str` 类型 和 `[T]` 类型都表示不定大小的值集，所有它们是无固定大小类型。

3.  `dyn` 类型：它是特型对象的引用目标

4. 结构体类型的最后一个字段（而且只能是最后一个）可以是无固定大小的，并且这样的结构体本身也是无固定大小的，如 `Rc<T>`  引用计数指针的内部实现是指向私有类型 `RcBox<T>`，后者把引用计数和 `T` 保存在一起



这里介绍下第 3 点的特型对象： 

特型对象是指向`实现了给定特型的某个值`的指针。例如类型 `&dyn std::io::Write` 和 `Box<dyn std::io::Write> ` 是指向了 `Write 特型` 的某个值的指针。引用目标可能是文件、网络套接字或某种实现了 `Write` 的自定义类型。由于实现了 `Write` 的类型集是开放式的，所以 `dyn Write` 作为一个类型也是无固定大小的，就是说它的值可以有各种大小。



## 2.2 Send trait / Sync trait

**Send 和 Sync trait定义**

[Send trait](https://doc.rust-lang.org/std/marker/trait.Send.html) 定义如下

```rust
pub unsafe auto trait Send {}
```

[Sync trait](https://doc.rust-lang.org/std/marker/trait.Sync.html) 定义如下

```rust
pub unsafe auto trait Sync {}
```

这两个 `trait` 都是 `unsafe auto trait`，如果实现 `Send / Sync` ，要自己为它们的安全性负责

* `auto` 意味着编译器会在合适的场合，自动为数据结构添加它们的实现
* `unsafe` 代表实现的这个 `trait` 可能会违背 `Rust` 的内存安全准则



**Send / Sync 是并发安全的基础**

* 如果一个类型 `T` 实现了 `Send`，即 `T: Send`， 意味着 `T` 可以安全地从一个线程移动到另一个线程，也就是说所有权可以在线程间移动。此时 `T` 在某个线程中的 `独占访问`是线程安全的
* 如果一个类型 `T` 实现了 `Sync` ，即 `T: Sync`， 则意味着 `&T` 可以安全地在多个线程中共享。此时 `T`  在线程间的 `只读共享` 是安全的
* 一个类型 `T` 满足 `Sync`，当且仅当 `&T` 满足 `Send` 



**支持 Send / Sync 的数据结构**

对于我们自己定义的数据结构，如果其内部的所有域都实现了 `Send / Sync`，那么这个数据结构会被**自动添加 Send / Sync** 。基本上原生数据结构都支持 `Send / Sync`，也就是，绝大多数自定义的数据结构都是满足 Send / `Sync` 的。



**标准库中，不支持 Send / Sync 的数据结构**

* 裸指针 `*const T / *mut T`。它们是不安全的，所以既不是 `Send` 也不是 `Sync`
* `UnsafeCell<T> ` 不支持 `Sync`。也就是，任何使用了 `Cell` 或者 `RefCell` 的数据结构不支持 `Sync`
* 引用计数 `Rc` 不支持 `Send` 也不支持 `Sync`。所以 `Rc` 无法跨线程



**例1：**

跨线程不可以使用 `Rc`，在 Rust 下，创建一个新的线程时使用 `std::thread::spawn`

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T> 
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
```

它的参数是一个闭包，这个闭包需要 `Send + 'static`

* `'static` 表示：闭包捕获的自由变量必须是一个拥有所有权的类型，或者是一个拥有静态生命周期的引用
* `Send` 表示：这些被捕获自由变量的所有权可以从一个线程移动到另一个线程

如下代码，在线程间传递 `Rc`，无法编译通过，因为 `Rc` 的实现不支持 `Send` 和 `Sync`

```rust
use std::{
    rc::Rc,
    thread,
};

// Rc 既不是 Send，也不是 Sync
fn rc_is_not_send_and_sync() {
    let a = Rc::new(1);
    let b = a.clone();
    let c = a.clone();
  
    thread::spawn(move || {
        println!("c= {:?}", c);
    });
}
```



**例2：跨进程怎么使用RefCell**

[RefCell](https://doc.rust-lang.org/std/cell/struct.RefCell.html#impl-Send) 实现了 `Send`，但没有实现 `Sync`，但 `RefCell` 可以在线程间转移所有权，以下例子可以正常运行

```rust
use std::{
    cell::RefCell,
    thread,
};

fn refcell_is_send() {
    let a = RefCell::new(1);
  
    thread::spawn(move || {
        println!("a= {:?}", a);
    });
}
```



**例3：跨进程不可使用 `Arc<RefCell<T>>`**

因为 `Rc` 不能 `Send`，所以无法跨线程使用 `Rc<RefCell<T>>` 这样的数据。而 `Arc` 是支持 `Send / Sync` 的，那么可以使用 `Arc<RefCell<T>>` 来获得一个可以在多线程间共享，且可以修改的类型的类型吗？答案是不可以。

```rust
use std::{
    cell::RefCell,
    sync::Arc,
    thread,
};

// RefCell 现在有多个 Arc 持有它，虽然 Arc 是 Send/Sync，但 RefCell 不是 Sync
fn refcell_is_not_sync() {
    let a = Arc::new(RefCell::new(1));
    let b = a.clone();
    let c = a.clone();
  
    thread::spawn(move || {
        println!("c= {:?}", c);
    });
}
```

因为 `Arc` 内部的数据是共享的，需要支持 `Sync` 的数据结构，但是 `RefCell` 不是 `Sync`，编译失败



**例4：Arc 和 Mutex 实现多线程共享且可修改的类型**

在多线程情况下，只能使用支持 `Send/Sync` 的 `Arc` 和 `Mutex`，构造一个可以在多线程间共享且可以修改的类型，如

```rust
use std::{
    sync::{Arc, Mutex},
    thread,
};

// Arc<Mutex<T>> 可以多线程共享且修改数据
fn arc_mutext_is_send_sync() {
    let a = Arc::new(Mutex::new(1));
    let b = a.clone();
    let c = a.clone();
  
    let handle = thread::spawn(move || {
        let mut g = c.lock().unwrap();
        *g += 1;
    });

    {
        let mut g = b.lock().unwrap();
        *g += 1;
    }

    handle.join().unwrap();
    println!("a= {:?}", a);
}

fn main() {
    arc_mutext_is_send_sync();
}
```



# 3 类型转换:From / Into / AsRef / AsMut

**类型转换的 2 种方式**

1、在类型 `T` 的实现里，为每一种可能的转换提供一个方法

```rust
// 第一种方法，为每一种转换提供一个方法
// 把字符串 s 转换成 Path
let v = s.to_path();
// 把字符串 s 转换成 u64
let v = s.to_u64();
```

缺点：以后每次要添加对新类型的转换，都要重新修改类型 `T` 的实现



2、为类型 `T` 和类型 `U` 之间的转换实现一个数据转换 `trait`，这样可以用同一个方法来实现不同的转换

```rust
// 第二种方法，为 s 和要转换的类型之间实现一个 Into<T> trait
// v 的类型根据上下文得出
let v = s.into();
// 或者也可以显式地标注 v 的类型
let v: u64 = s.into();
```

优点：只需要添加一个对于数据转换 `trait` 的新实现即可



**对值类型的转换 和 对引用类型的转换，Rust 提供了两套不同的 trait**

* 值类型 到 值类型 的转换：`From / Into 或 TryFrom / TryInto`。 `From` 和`Into` 会获取其参数的所有权，对其进行转换，然后将转换结果的所有权返回给调用者。

  > 注意：如果你的数据类型在转换过程中有可能出现错误，可以使用 `TryFrom<T> 和 TryInto<T>`，它们的用法和 `From<T>/Into<T>` 一样，只是 `trait` 内多了一个关联类型 `Error`，且返回的结果是 `Result<T, Self::Error>`

* 引用类型 到 引用类型的转换：`AsRef / AsMut`，`AsRef` 特型和 `AsMut` 特型用于从一种类型借入另一种类型的引用



## 3.1 `From<T>/Into<T>`

[From](https://doc.rust-lang.org/std/convert/trait.From.html) / [Into](https://doc.rust-lang.org/std/convert/trait.Into.html)：约定了数据间如何转换的行为，它们都用于类型转换，它们会接受一种类型的值，并返回另一种类型的值。



**From 定义 和 Into 定义分别如下**

```rust
// From定义
pub trait From<T> {
    fn from(T) -> Self;
}

// Into定义
pub trait Into<T> {
    fn into(self) -> T;
}
```



**推荐使用 From**

* `From` 可以根据上下文做类型推导，使用场景更多

* 实现了 `From` 会自动实现 `Into`

  ```rust
  // 实现 From 会自动实现 Into
  impl<T, U> Into<U> for T where U: From<T> {
      fn into(self) -> U {
          U::from(self)
      }
  }
  
  // 所以大部分情况下，只需要实现 From，然后这两种方式都能做数据转换，比如：
  // 以下两种方式是等价的
  let s = String::from("Hello world!");
  let s: String = "Hello world!".into();
  ```

* `From` 和 `Into` 还是自反的：把类型 `T` 的值转换成类型 `T`，会直接返回。这是因为标准库有如下的实现：

  ```rust
  // From（以及 Into）是自反的
  impl<T> From<T> for T {
      fn from(t: T) -> T {
          t
      }
  }
  ```



**例子**

有了 `From` 和 `Into`，很多函数的接口就可以变得灵活

```rust
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// 函数接收一个 IpAddr 为参数，使用 Into 让更多的类型可以被这个函数使用
fn print(v: impl Into<IpAddr>) {
    println!("{:?}", v.into());
}

fn main() {
    let v4: Ipv4Addr = "2.2.2.2".parse().unwrap();
    let v6: Ipv6Addr = "::1".parse().unwrap();
    
    // IPAddr 实现了 From<[u8; 4]，转换 IPv4 地址
    print([1, 1, 1, 1]); // 1.1.1.1
  
    // IPAddr 实现了 From<[u16; 8]，转换 IPv6 地址
    print([0xfe80, 0, 0, 0, 0xaede, 0x48ff, 0xfe00, 0x1122]); // fe80::aede:48ff:fe00:1122
  
    // IPAddr 实现了 From<Ipv4Addr>
    print(v4); // 2.2.2.2
  
    // IPAddr 实现了 From<Ipv6Addr>
    print(v6); // ::1
}
```



## 3.2 `AsRef<T>/AsMut<T>`

[AsRef](https://doc.rust-lang.org/std/convert/trait.AsRef.html) 的定义如下

```rust
pub trait AsRef<T> where T: ?Sized {
    fn as_ref(&self) -> &T;
}
```

[AsMut](https://doc.rust-lang.org/std/convert/trait.AsMut.html) 的定义如下

```rust
pub trait AsMut<T> where T: ?Sized {
    fn as_mut(&mut self) -> &mut T;
}
```

* 两个特型都限制了 `T` 是不固定大小的类型，如 `str`、`[u8]` 等
* `AsMut<T>` 除了使用 可变引用 生成 可变引用 外，其它都和 `AsRef<T>` 一样
* 如果类型实现了 `AsRef<T>`，那它可以高效的从中借入 `&T`。`AsMut` 是 `AsRef` 针对可变引用的对应类型。



**下面重点看下 `AsRef<T>`**

看标准库中[打开文件](https://doc.rust-lang.org/std/fs/struct.File.html#method.open)的接口 `std::fs::File::open`

```rust
pub fn open<P: AsRef<Path>>(path: P) -> Result<File>
```

它的参数 `path` 是符合 `AsRef` 的类型，所以可以为这个参数传入 `String`、`&str`、`PathBuf`、`Path` 等类型。而且，当你使用 `path.as_ref()` 时，会得到一个 `&Path`。



**例子**

`AsRef` 的使用

```rust
#[allow(dead_code)]
enum Language {
    Rust,
    TypeScript,
    Elixir,
    Haskell,
}

// 实现AsRef
impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        match self {
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::Elixir => "Elixir",
            Language::Haskell => "Haskell",
        }
    }
}

fn print_ref(v: impl AsRef<str>) {
    println!("{}", v.as_ref());
}

fn main() {
    // &str 实现了 AsRef<str>
    print_ref("Hello world!");
  
    // String 实现了 AsRef<str>
    print_ref("Hello world!".to_string());
  
    let lang = Language::Rust;
    // 自己定义的 enum 也实现了 AsRef<str>
    print_ref(lang);
}
```



 # 4 操作符:Deref / DerefMut

## 4.1 操作符

`Rust` 为所有的运算符都提供了 `trait`，可见[操作符的官方文档](https://doc.rust-lang.org/std/ops/index.html)，我们可以为自己的类型重载某些操作符



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-02-19%2015.41.15.png)

## 4.2 Deref 和 DerefMut 

**Deref 的定义**

实现了`std::ops::Deref`，可以指定` * 解引用`运算符在类型上的行为，操作符[Deref](https://doc.rust-lang.org/std/ops/trait.Deref.html) 的定义如下

```rust
pub trait Deref {
    // 解引用出来的结果类型
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}
```



**DerefMut 的定义**

实现了`std::ops::DerefMut`，可以指定 `. 解引用`运算符 在类型上的行为，操作符[DerefMut](https://doc.rust-lang.org/std/ops/trait.DerefMut.html) 的定义如下

```rust
pub trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target;
}
```

`DerefMut`  继承了 `Deref`，只是它额外提供了一个 `deref_mut` 方法，用来获取可变的解引用。以下重点讲解`Deref`。



对于普通的引用，解引用很直观，因为它只有一个指向值的地址，从这个地址可以获取到所需要的值，比如下面的例子：

```rust
let mut x = 42;
let y = &mut x;
// 解引用，内部调用 DerefMut（其实现就是 *self）
*y += 1;
```

但对智能指针来说，拿什么域来解引用就不那么直观了，`Rc<T>` 指针类型就实现了 `Deref` 特型，看下 `Rc` 是怎么实现 `Deref` 的：

```rust
impl<T: ?Sized> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner().value
    }
}
```

如下代码

```rust
use std::rc::Rc;

fn main() {
    let a = Rc::new(1);
    let b = a.clone();
    println!("v = {}", *b); // v = 1
}
```

从代码可以看到，`b` 最终指向了堆上的 `RcBox` 内部的 `value` 的地址，然后如果对其解引用的话，得到了 `value` 对应的值。以下图为例，最终打印出 `v = 1`

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-02-19%2015.40.06.png)

从图中还可以看到，`Deref` 和 `DerefMut` 是自动调用的，`*b` 会被展开为 `*(b.deref())`。



**例子：为自己的数据结构实现 Deref**

在 `Rust` 中，绝大多数智能指针都实现了 `Deref`，如下列子为自己的数据结构实现 `Deref`

```rust
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
struct Buffer<T>(Vec<T>);

impl<T> Buffer<T> {
    pub fn new(v: impl Into<Vec<T>>) -> Self {
        Self(v.into())
    }
}

// 实现Deref
impl<T> Deref for Buffer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 实现DerefMut
impl<T> DerefMut for Buffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn main() {
    let mut buf = Buffer::new([1, 3, 2, 4]);
  
    // 因为实现了 Deref 和 DerefMut，这里 buf 可以直接访问 Vec<T> 的方法
    // 下面这句相当于：(*buf).sort()，也就是 (*&mut buf.0).sort()
    buf.sort();
  
    println!("buf: {:?}", buf);
}
```

注意：

1. 在这个例子里，数据结构 `Buffer` 包裹住了 `Vec`，但这样一来，原本`Vec` 实现了的很多方法，现在使用起来就很不方便，需要用 `buf.0` 来访问。怎么办？可以实现 `Deref` 和 `DerefMut`，这样在解引用时，直接访问到 `buf.0`，省去了代码的啰嗦和数据结构内部字段的隐藏

2. 写 `buf.sort()` 时，并没有做解引用的操作，为什么会相当于访问了 `buf.0.sort()` 呢？这是因为 `sort()` 方法第一个参数是 `&mut self`，此时 `Rust` 编译器会强制做 `Deref / DerefMut` 的解引用，所以这相当于 `(*(&mut buf)).sort()`



# 5 其他:Debug / Display / Default

## 5.1  Debug trait

[Debug trait](https://doc.rust-lang.org/std/fmt/trait.Debug.html)：定义了数据如何被以 `debug` 的方式显示出来的行为，`Debug` 是为开发者调试打印数据结构所设计的，数据结构实现了 `Debug trait`，则可以用 `{:?}` 格式来打印数据结构。



**Debug 定义如下**

```rust
pub trait Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}
```

要实现 `Debug trait` 可以使用派生宏 `#[derive(Debug)]`。例如：

```rust
#[derive(Debug)]
struct Person {
    name: String,
    age: u8,
}

fn main() {
    let zhangsan = Person {
        name: "zhangsan".to_string(),
        age: 18,
    };
   
    // Person { name: "zhangsan", age: 18 }
    println!("{:?}", zhangsan);
}
```



## 5.2 Display trait

[Display trait](https://doc.rust-lang.org/std/fmt/trait.Display.html)：定义了数据如何显示出来的行为。`Display` 是给用户显示数据结构所设计的，数据结构实现了 `Display trait`，则可以用 `{}` 格式来打印数据结构



**Display 定义如下**

```rust
pub trait Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}
```

要实现  `Display trait` ，必须手动实现，不能使用派生宏。例如

```rust
use std::fmt;

struct Person {
    name: String,
    age: u8,
}

// 为 Person结构体实现 Display
impl fmt::Display for Person {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is {} years old", self.name, self.age)
    }
}

fn main() {
    let zhangsan = Person {
        name: "zhangsan".to_string(),
        age: 18,
    };
    let lisi = Person {
        name: "lisi".to_string(),
        age: 42,
    };

    // zhangsan is 18 years old
    // lisi is 42 years old
    println!("{}", zhangsan);
    println!("{}", lisi);
}
```



## 5.3 Default trait

[Default trait](https://doc.rust-lang.org/std/default/trait.Default.html)：定义了数据类型的缺省值如何产生的行为，实现了该特型可以自定义默认值。



**Default 定义如下**

```rust
pub trait Default {
    fn default() -> Self;
}
```

* 某些类型具有默认值：向量 或 字符串默认 为空、数值 默认为 0、`Option` 类型默认为 `None`

* `Rust` 为所有集合类型（如 `Vec`、`HashMap`、`BinaryHeap`）都实现了 `Default`，其 `default` 方法会返回一个空集合。



**怎么实现 Default**

`Rust` 不会为结构体类型隐式实现 `Default`，可以通过派生宏 `#[derive(Default)] `来生成实现，前提是结构体类型中的每个字段都实现了 `Default`



注意：如果类型 `T` 实现了 `Default`，那么标准库就会自动为 `Rc<T>`、 `Arc<T>`、 `Box<T>`、 `Cell<T>`、 `RefCell<T>`、 `Cow<T>`、 `Mutex<T>`、 `RwLock<T>` 实现 1Default1。



**作用**

`Default trait` 用于为类型提供默认值。在初始化一个数据结构时，我们可以部分初始化，然后剩余的部分使用 `Default::default()`。



**例子**

综合使用 `Debug / Display / Default` 的例子

```rust
use std::fmt;

// struct 可以 derive Default，但需要所有字段都实现了 Default
#[derive(Clone, Debug, Default)]
struct Developer {
    name: String,
    age: u8,
    lang: Language,
}

// enum 不能使用 derive 宏实现 Default
#[allow(dead_code)]
#[derive(Clone, Debug)]
enum Language {
    Rust,
    TypeScript,
    Elixir,
    Haskell,
}

// 因为不能用 derive 自动实现，所以得手动实现 Default
impl Default for Language {
    fn default() -> Self {
        Language::Rust
    }
}

impl Developer {
    pub fn new(name: &str) -> Self {
        // 用 ..Default::default() 为剩余字段使用缺省值
        Self {
            name: name.to_owned(),
            ..Default::default()
        }
    }
}

impl fmt::Display for Developer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({} years old): {:?} developer",
            self.name, self.age, self.lang
        )
    }
}

fn main() {
    // 使用 T::default()
    let dev1 = Developer::default();
    // 使用 Default::default()，但此时类型无法通过上下文推断，需要提供类型
    let dev2: Developer = Default::default();
    // 使用 T::new
    let dev3 = Developer::new("Tyr");
    println!("dev1: {}\ndev2: {}\ndev3: {:?}", dev1, dev2, dev3);
}
```



# 参考 

* [陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/421324)
* [Drop 释放资源](https://course.rs/advance/smart-pointer/drop.html)
* [Rust程序设计（第2版）](https://book.douban.com/subject/36547630/)

