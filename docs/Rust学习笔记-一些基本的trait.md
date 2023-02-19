# 1 内存相关：Clone/Copy/Drop

## 1.1 Clone trait

[clone trait](https://doc.rust-lang.org/std/clone/trait.Clone.html) ：定义了数据被深拷贝的行为，深拷贝时栈内存和堆内存会一起拷贝



**定义如下**

```rust
pub trait Clone: Sized {
    fn clone(&self) -> Self; // 没缺省实现
    fn clone_from(&mut self, source: &Self) { ... } // 有缺省实现
}
```

clone_from()方法的作用：例如代码`a.clone_from(&b) `和` a = b.clone() `看起来是等价的，其实不是，如果 a 已经存在，在 clone() 过程中会分配内存，如果使用 `a.clone_from(&b) `可以避免内存分配，提高效率。



**怎么实现**

可以通过`派生宏#[derive(Clone)]`实现Clone trait。数据结构要实现Clone trait，**前提是数据结构的每一个字段都已经实现了 Clone trait**。



**例子**

定义Developer 结构和 Language 枚举

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

因为是深拷贝，所以对于String类型的name，其堆上的内存也被 Clone 了一份。



注意：clone 方法的参数是 &self，在 clone 一个数据时只需要有已有数据的只读引用。但对 `Rc<T>` 这样在clone() 时维护引用计数的数据结构，clone() 过程中会改变自己，所以要用 `Cell<T>` 这样提供内部可变性的结构来进行改变。



## 1.2 Copy trait

[Copy trait](https://doc.rust-lang.org/std/marker/trait.Copy.html) ：定义了数据被浅拷贝的行为。Copy trait没有任何方法，它只是一个`标记 trait（marker trait）`



**定义如下**

```rust
pub trait Copy: Clone {}
```



**怎么实现**

1. 如果要实现 Copy trait 的话，**必须实现 Clone trait**，然后实现一个空的 Copy trait

2. 和 Clone trait一样，也可以用派生宏 `#[derive(Copy)] `来为数据结构实现 Copy trait，**前提是数据结构的所有字段都已经实现了 Copy trait**



**例子**

如下代码，为 Developer 和 Language 加上Copy宏时，编译后出错。因为 String 类型没有实现 Copy，所以Developer 数据结构只能 clone，无法 copy

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

注意：

1. 如果类型实现了 Copy，那么在赋值、函数调用的时候，值会被拷贝，执行 Copy 复制的语义；否则所有权会被移动，执行Move的语义（[clone/copy和所有权相关可见另一篇文章](https://zhuanlan.zhihu.com/p/587134567)）

2. 不可变引用都实现了 Copy，而可变引用 `&mut T` 没有实现 Copy。因为如果可变引用实现了 Copy trait，那么生成一个可变引用然后把它赋值给另一个变量时，就会违背所有权规则：同一个作用域下只能有一个可变引用

   

## 1.3 Drop trait

[Drop trait](https://doc.rust-lang.org/std/ops/trait.Drop.html) ：允许在变量离开作用域时执行某些自定义操作(释放资源、执行收尾工作)



**定义如下**

```rust
pub trait Drop {
    fn drop(&mut self);
}
```

大部分场景无需为数据结构提供 Drop trait，**系统默认会依次对数据结构的每个域做 drop**，rust不允许显式地调用析构函数 drop



**需要手动实现 Drop的2种情况**

1. 在数据结束生命周期时做一些事情，比如记日志

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
    // d.drop();
    // c.drop();
}

// 运行后输入如下：
// 记录日志b
// 记录日志a
```

2. 需要对资源回收的场景。编译器并不知道你额外使用了哪些资源，也就无法帮助你 drop 它们。比如说锁资源的释放，在 MutexGuard 中实现了 Drop 来释放锁资源

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



**互斥的Drop和Copy**

Copy trait 和 Drop trait 是互斥的，无法为同一种数据类型同时实现 Copy 和 Drop，否则编译器会报错。

> 因为Copy 是按位做浅拷贝，那么它会默认拷贝的数据没有需要释放的资源；而 Drop 恰恰是为了释放额外的资源而生的

例子：此例子只是一个演示例子，没有处理内存泄漏的问题

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

在代码中，强行用 `Box::into_raw` 获得堆内存的指针，放入 RawBuffer 结构中，这样就接管了这块堆内存的释放。RawBuffer 实现了 Copy trait，所以无法实现 Drop trait，最终会导致内存泄漏，因为该释放的堆内存没有释放。但是这个操作不会破坏 Rust 的正确性保证：即便你 Copy 了 N 份 RawBuffer，由于无法实现 Drop trait，RawBuffer 指向的那同一块堆内存不会释放，所以不会出现 use after free 的内存安全问题。



实际上，任何编程语言都无法保证不发生人为的内存泄漏，比如程序在运行时，开发者疏忽了，对哈希表只添加不删除，就会造成内存泄漏。但 Rust 会保证即使开发者疏忽了内存泄漏，也不会出现内存安全问题。



# 2 标记trait：Sized/Send/Sync

## 2.1 Sized trait

[Sized trait](https://doc.rust-lang.org/std/marker/trait.Sized.html) ：没有任何方法，是一个标记 trait，用于`标记有具体大小`的类型。copy trait 也是一个标记trait



**标记trait的作用**

可以用作 trait bound 来进行类型安全检查



**定义如下**

```rust
pub trait Sized { }
```



**Sized固定大小约束**

在使用泛型参数时，Rust 编译器会**自动**为泛型参数加上 Sized 约束。

例如：

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
struct Data<T: Sized> {
    inner: T,
}

fn process_data<T: Sized>(data: Data<T>) {
    todo!();
}
```

这样添加约束后定义出的泛型结构，在编译期，大小是固定的，可以作为参数传递给函数。如果没有这个约束，T 是大小不固定的类型， process_data 函数会无法编译。

在少数情况下，T是可变类型 ，此时自动添加的Sized约束不适用，需要用`?Sized`来摆脱这个约束。



**?Sized任意大小约束**

如果显式定义了`T: ?Sized`，那么 T 就可以是任意大小。

例如： Cow 枚举中泛型参数 B 的约束是` ?Sized`

```rust
pub enum Cow<'a, B: ?Sized + 'a> where B: ToOwned,
{
    // 借用的数据
    Borrowed(&'a B),
    // 拥有的数据
    Owned(<B as ToOwned>::Owned),
}
```

此时 B 的大小是不固定的。要注意 `Borrowed(&'a B) `大小是固定的，因为它内部是对 B 的一个引用，而引用的大小是固定的。



## 2.2 Send trait/Sync trait

**[Send trait](https://doc.rust-lang.org/std/marker/trait.Send.html) 定义如下**

```rust
pub unsafe auto trait Send {}
```

**[Sync trait](https://doc.rust-lang.org/std/marker/trait.Sync.html) 定义如下**

```rust
pub unsafe auto trait Sync {}
```

这两个 trait 都是 unsafe auto trait，如果实现Send/Sync trait ，要自己为它们的安全性负责

* auto 意味着编译器会在合适的场合，自动为数据结构添加它们的实现
* unsafe 代表实现的这个 trait 可能会违背 Rust 的内存安全准则



**Send/Sync 是并发安全的基础**

* 如果一个类型 T 实现了 Send trait，意味着 T 可以安全地从一个线程移动到另一个线程，也就是说所有权可以在线程间移动
* 如果一个类型 T 实现了 Sync trait，则意味着 `&T` 可以安全地在多个线程中共享。一个类型 T 满足 Sync trait，当且仅当 &T 满足 Send trait



**对于 Send/Sync 在线程安全中的作用**

* 如果一个类型 `T: Send`，那么 T 在某个线程中的`独占访问`是线程安全的
* 如果一个类型 `T: Sync`，那么 T 在线程间的`只读共享`是安全的



**支持Send/Sync的数据结构**

对于我们自己定义的数据结构，如果其内部的所有域都实现了 Send / Sync，那么这个数据结构会被**自动添加 Send / Sync** 。基本上原生数据结构都支持 Send / Sync，也就是，绝大多数自定义的数据结构都是满足 Send / Sync 的



**标准库中，不支持 Send / Sync 的数据结构主要有：**

* 裸指针 *const T / *mut T。它们是不安全的，所以既不是 Send 也不是 Sync
* `UnsafeCell<T> `不支持 Sync。也就是，任何使用了 Cell 或者 RefCell 的数据结构不支持 Sync
* 引用计数 Rc 不支持 Send 也不支持 Sync。所以 Rc 无法跨线程



**例1：跨线程不可以使用Rc**

在 Rust 下，创建一个新的线程时使用 `std::thread::spawn`

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T> 
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
```

它的参数是一个闭包，这个闭包需要 Send + 'static：

* 'static 表示：闭包捕获的自由变量必须是一个拥有所有权的类型，或者是一个拥有静态生命周期的引用
* Send 表示：这些被捕获自由变量的所有权可以从一个线程移动到另一个线程



如下代码，在线程间传递 Rc，无法编译通过，因为 Rc 的实现不支持 Send 和 Sync

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

[RefCell](https://doc.rust-lang.org/std/cell/struct.RefCell.html#impl-Send) 实现了 Send，但没有实现 Sync，但 RefCell 可以在线程间转移所有权，以下例子可以正常运行

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



**例3：跨进程不可使用`Arc<RefCell<T>>`**

因为 Rc 不能 Send，所以无法跨线程使用 `Rc<RefCell<T>>` 这样的数据。而 Arc 是支持 Send/Sync 的，那么可以使用 `Arc<RefCell<T>>` 来获得一个可以在多线程间共享，且可以修改的类型的类型吗？不可以

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

因为 Arc 内部的数据是共享的，需要支持 Sync 的数据结构，但是 RefCell 不是 Sync，编译失败



**例4：Arc和Mutex实现多线程共享且可修改的类型**

在多线程情况下，只能使用支持 Send/Sync 的 Arc 和 Mutex，构造一个可以在多线程间共享且可以修改的类型

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



# 3 类型转换相关：From/Into/AsRef/AsMut

**类型转换的2种方式**

1、在类型 T 的实现里，为每一种可能的转换提供一个方法

```rust
// 第一种方法，为每一种转换提供一个方法
// 把字符串 s 转换成 Path
let v = s.to_path();
// 把字符串 s 转换成 u64
let v = s.to_u64();
```

缺点：以后每次要添加对新类型的转换，都要重新修改类型 T 的实现



2、为类型 T 和类型 U 之间的转换实现一个数据转换 trait，这样可以用同一个方法来实现不同的转换

```rust
// 第二种方法，为 s 和要转换的类型之间实现一个 Into<T> trait
// v 的类型根据上下文得出
let v = s.into();
// 或者也可以显式地标注 v 的类型
let v: u64 = s.into();
```

优点：只需要添加一个对于数据转换 trait 的新实现即可



**对值类型的转换和对引用类型的转换，Rust 提供了两套不同的 trait**

* 值类型 到 值类型的转换：From / Into 或 TryFrom / TryInto

  > 注意：如果你的数据类型在转换过程中有可能出现错误，可以使用 `TryFrom<T> 和 TryInto<T>`，它们的用法和 `From<T>/Into<T>` 一样，只是 trait 内多了一个关联类型 Error，且返回的结果是 `Result<T, Self::Error>`

* 引用类型 到 引用类型的转换：AsRef / AsMut



## 3.1 `From<T>/Into<T>`

[From](https://doc.rust-lang.org/std/convert/trait.From.html) / [Into](https://doc.rust-lang.org/std/convert/trait.Into.html)：约定了数据间如何转换的行为



**From定义 和 Into定义分别如下**

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



**推荐使用From**

* From 可以根据上下文做类型推导，使用场景更多

* 实现了 From 会自动实现 Into，因为

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

* From 和 Into 还是自反的：把类型 T 的值转换成类型 T，会直接返回。这是因为标准库有如下的实现：

  ```rust
  // From（以及 Into）是自反的
  impl<T> From<T> for T {
      fn from(t: T) -> T {
          t
      }
  }
  ```



**例子**

有了 From 和 Into，很多函数的接口就可以变得灵活

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

[AsRef](https://doc.rust-lang.org/std/convert/trait.AsRef.html)的定义如下

```rust
pub trait AsRef<T> where T: ?Sized {
    fn as_ref(&self) -> &T;
}
```

[AsMut](https://doc.rust-lang.org/std/convert/trait.AsMut.html)的定义如下

```rust
pub trait AsMut<T> where T: ?Sized {
    fn as_mut(&mut self) -> &mut T;
}
```

* 两个 trait 都允许 T 使用大小可变的类型，如 str、[u8] 等
* `AsMut<T>` 除了使用 可变引用 生成 可变引用 外，其它都和` AsRef<T>` 一样



**下面重点看下`AsRef<T>`**

看标准库中[打开文件](https://doc.rust-lang.org/std/fs/struct.File.html#method.open)的接口 `std::fs::File::open`

```rust
pub fn open<P: AsRef<Path>>(path: P) -> Result<File>
```

它的参数 path 是符合 AsRef 的类型，所以可以为这个参数传入 String、&str、PathBuf、Path 等类型。而且，当你使用` path.as_ref()` 时，会得到一个 `&Path`。



**例子**

AsRef 的使用

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



 # 4 操作符相关：Deref/DerefMut

## 4.1 操作符

Rust 为所有的运算符都提供了 trait，可见[操作符的官方文档](https://doc.rust-lang.org/std/ops/index.html)，我们可以为自己的类型重载某些操作符



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-02-19%2015.41.15.png)

## 4.2 Deref 和 DerefMut 

**操作符[Deref](https://doc.rust-lang.org/std/ops/trait.Deref.html) 的定义如下**

```rust
pub trait Deref {
    // 解引用出来的结果类型
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}
```

**操作符[DerefMut](https://doc.rust-lang.org/std/ops/trait.DerefMut.html) 的定义如下**

```rust
pub trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target;
}
```

DerefMut “继承”了 Deref，只是它额外提供了一个 deref_mut 方法，用来获取可变的解引用。以下重点讲解Deref。



对于普通的引用，解引用很直观，因为它只有一个指向值的地址，从这个地址可以获取到所需要的值，比如下面的例子：

```rust
let mut x = 42;
let y = &mut x;
// 解引用，内部调用 DerefMut（其实现就是 *self）
*y += 1;
```

但对智能指针来说，拿什么域来解引用就不那么直观了，看下 Rc 是怎么实现 Deref 的：

```rust
impl<T: ?Sized> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner().value
    }
}
```

可以看到，它最终指向了堆上的 RcBox 内部的 value 的地址，然后如果对其解引用的话，得到了 value 对应的值。以下图为例，最终打印出 v = 1

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-02-19%2015.40.06.png)

从图中还可以看到，Deref 和 DerefMut 是自动调用的，`*b 会被展开为 *(b.deref())`。



**例子：为自己的数据结构实现Deref**

在 Rust 中，绝大多数智能指针都实现了 Deref，如下列子为自己的数据结构实现 Deref

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

1. 在这个例子里，数据结构 Buffer 包裹住了 Vec，但这样一来，原本 Vec 实现了的很多方法，现在使用起来就很不方便，需要用 buf.0 来访问。怎么办？可以实现 Deref 和 DerefMut，这样在解引用时，直接访问到 buf.0，省去了代码的啰嗦和数据结构内部字段的隐藏。

2. 写 buf.sort() 时，并没有做解引用的操作，为什么会相当于访问了 buf.0.sort() 呢？这是因为 sort() 方法第一个参数是 &mut self，此时 Rust 编译器会强制做 Deref/DerefMut 的解引用，所以这相当于 (*(&mut buf)).sort()。



# 5 其他：Debug/Display/Default

## 5.1  Debug trait

[Debug](https://doc.rust-lang.org/std/fmt/trait.Debug.html)：定义了数据如何被以 debug 的方式显示出来的行为



**定义如下**

```rust
pub trait Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}
```



**怎么实现**

实现Debug trait 可以使用派生宏`#[derive(Debug)]`



**作用**

Debug 是为开发者调试打印数据结构所设计的，数据结构实现了 Debug trait，则可以用` {:?} `来打印数据结构



## 5.2 Display trait

[Display](https://doc.rust-lang.org/std/fmt/trait.Display.html)：定义了数据如何显示出来的行为



**定义如下**

```rust
pub trait Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}
```



**怎么实现**

 Display trait 必须开发者手动实现



**作用**

Display 是给用户显示数据结构所设计的，数据结构实现了 Display trait，则可以用`{}`来打印数据结构



## 5.3 Default

[Default trait](https://doc.rust-lang.org/std/default/trait.Default.html)：定义了数据类型的缺省值如何产生的行为



**定义如下**

```rust
pub trait Default {
    fn default() -> Self;
}
```



**怎么实现**

可以通过 derive 宏 `#[derive(Default)] `来生成实现，前提是类型中的每个字段都实现了 Default trait



**作用**

Default trait 用于为类型提供缺省值。在初始化一个数据结构时，我们可以部分初始化，然后剩余的部分使用 Default::default()。



**例子**

综合使用Debug/Display/Default的例子

```rust
use std::fmt;

// struct 可以 derive Default，但需要所有字段都实现了 Default
#[derive(Clone, Debug, Default)]
struct Developer {
    name: String,
    age: u8,
    lang: Language,
}

// enum 不能 derive Default
#[allow(dead_code)]
#[derive(Clone, Debug)]
enum Language {
    Rust,
    TypeScript,
    Elixir,
    Haskell,
}

// 手动实现 Default
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