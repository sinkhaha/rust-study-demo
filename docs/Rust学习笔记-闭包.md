# 1 闭包

闭包可以简单理解为一种匿名函数，它可以赋值给变量也可以作为参数传递给其它函数；不同于函数的是，闭包可以捕获环境中的`自由变量`，成为闭包类型的一部分



## 1.1 闭包的表示形式

在 Rust 中，闭包的表示形式如下

```rust
|param1, param2,...| {
    语句1;
    语句2;
    返回表达式 // 闭包中最后一行表达式返回的值，就是闭包执行后的返回值
}

// 如果只有一个返回表达式，可以用
|param1| 返回表达式
```

还可以在闭包前加上move关键字，表示把变量的所有权从当前作用域移动到闭包的作用域

```rust
move |param1, param2,...| {
    语句1;
    语句2;
    返回表达式 
}
```



**例1：**闭包c 捕获了上下文中的 a 和 b，并通过`引用`来使用这两个自由变量

```rust
fn main() {
    let a = "Hello";
    let b = "zhangsan";

    let bibao = |msg:&str| {
        // 捕获自由变量a和b
        print!("{}, {}: {}", a, b, msg);
    };

    bibao("how are you?"); // Hello, zhangsan: how are you?%  
}
```

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%88%AA%E5%B1%8F2023-02-19%2017.28.21.png)



**例2：使用move关键字**

创建新线程的 thread::spawn 方法的参数就是一个闭包，其定义如下

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T> 
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
```

1. F: FnOnce() -> T：表明 F 是一个接受 0 个参数、返回 T 的闭包
2. F: Send + 'static：说明闭包 F，需要静态生命周期 或 拥有所有权，并且它还能被发送给另一个线程，说明被闭包 F 捕获的变量，也需要静态生命周期或者拥有所有权
3. T: Send + 'static：说明闭包 F 返回的数据结构 T，它能被发送给另一个线程，也需要静态生命周期或者拥有所有权



从定义可知，在使用 thread::spawn 时，需要使用 move 关键字，把变量的所有权从当前作用域移动到闭包的作用域，如下

```rust
use std::thread;

fn main() {
    let s = String::from("hello world");

    let handle = thread::spawn(move || {
        println!("moved: {:?}", s);
    });

    handle.join().unwrap();
}
```



## 1.2 闭包的本质

闭包是一种匿名类型，一旦声明，就会产生一个新的类型，但这个类型无法被其它地方使用。这个类型就像一个结构体，会包含所有捕获的变量。 



Rust中闭包产生的匿名数据类型，格式和 struct 是一样的。闭包是存储在栈上，并且除了捕获的数据外，闭包本身不包含任何额外函数指针指向闭包的代码。**闭包的大小跟参数、局部变量都无关，只跟捕获的变量有关。**



**例子：闭包的大小只跟捕获的变量有关**

```rust
use std::{collections::HashMap, mem::size_of_val};

fn main() {
    // c1长度为 0
    let c1 = || println!("hello world!");
  
    // c2长度也为 0，说明和参数无关
    let c2 = |i: i32| println!("hello: {}", i);
  
    let name = String::from("zhangsan");
    let name1 = name.clone();
  
    let mut table = HashMap::new();
    table.insert("hello", "world");
  
    // 如果捕获一个引用，长度为 8
    let c3 = || println!("hello: {}", name);
  
    // 捕获移动的数据 name1(长度 24) + table(长度 48)，闭包长度 72
    let c4 = move || println!("hello: {}, {:?}", name1, table);
  
    let name2 = name.clone();
  
    // 和局部变量无关，捕获了一个 String name2，闭包长度 24
    let c5 = move || {
        let x = 1;
        let name3 = String::from("lisi");
        println!("hello: {}, {:?}, {:?}", x, name2, name3);
    };

    // c1: 0, c2: 0, c3: 8, c4: 72, c5: 24, main: 0
    println!(
        "c1: {}, c2: {}, c3: {}, c4: {}, c5: {}, main: {}",
        size_of_val(&c1),
        size_of_val(&c2),
        size_of_val(&c3),
        size_of_val(&c4),
        size_of_val(&c5),
        size_of_val(&main),
    )
}
```

代码中分别生成了 5 个闭包

* c1 没有参数，也没捕获任何变量，从代码输出可以看到，c1 长度为 0
* c2 有一个 i32 作为参数，没有捕获任何变量，长度也为 0，可以看出参数跟闭包的大小无关
* c3 捕获了一个对变量 name 的引用，这个引用是 &String，长度为 8。而 c3 的长度也是 8
* c4 捕获了变量 name1 和 table，由于用了 move，它们的所有权移动到了 c4 中。c4 长度是 72，恰好等于 String 的 24 字节，加上 HashMap 的 48 字节
* c5 捕获了 name2，name2 的所有权移动到了 c5，虽然 c5 有局部变量，但它的大小和局部变量也无关，c5 的大小等于 String 的 24 字节



**Rust 闭包的性能和函数差不多。**闭包最大的问题是变量的多重引用导致生命周期不明确，但 Rust 从根本上使用所有权和借用，解决了这个问题

* 如果不使用 move 转移所有权，闭包会引用上下文中的变量，这个引用受借用规则的约束，所以只要编译通过，那么闭包对变量的引用就不会超过变量的生命周期，没有内存安全问题

* 如果使用 move 转移所有权，上下文中的变量在转移后就无法访问，闭包完全接管这些变量，它们的生命周期和闭包一致，所以也不会有内存安全问题

  

# 2 三种闭包类型

当闭包作为函数的参数或者数据结构的一个域时，需要告诉调用者，对闭包的约束。因为`函数参数`有三种传入方式：转移所有权、可变借用、不可变借用，所以相应的闭包类型也有三种：FnOnce、FnMut、Fn。比如以 thread::spawn 为例，它要求传入的闭包满足 FnOnce trait。



## 2.1 FnOnce trait

[FnOnce](https://doc.rust-lang.org/std/ops/trait.FnOnce.html)类型的闭包会拿走被捕获变量的所有权。Once说明该闭包只能被调用一次，当再次调用时，编译器就会报错



**定义如下**

```rust
pub trait FnOnce<Args> {
    type Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}
```

1. FnOnce 的参数是一个叫 Args 的泛型参数，它没有任何约束
2. 有一个关联类型 Output，它是闭包返回值的类型
3. 有一个方法 call_once ，要注意的是 call_once 第一个参数是 self，它会转移 self（闭包） 的所有权到 call_once 函数中



**例1：隐式的 FnOnce 例子**

```rust
fn main() {
    let name = String::from("zhangsan");
  
    // 这个闭包啥也不干，只是把捕获的参数返回
    let c = move |greeting: String| (greeting, name);

    let result = c("hello".to_string());

    println!("result: {:?}", result);

    // 再次调用c闭包会报错
    let result = c("hi".to_string());
}
```

因为使用了move 关键字，它是一个 FnOnce 的闭包，闭包c只能调用一次



**例2：如果一个闭包并不转移自己的内部数据，那么它就不是 FnOnce；然而一旦它被当做 FnOnce 调用，自己会被转移到 call_once 函数的作用域中，之后就无法再次调用了**

```rust
fn main() {
    let name = String::from("zhangsan");

    // 这个闭包会 clone 内部的数据返回，所以它不是 FnOnce
    let c = move |greeting: String| (greeting, name.clone());

    // 所以 c1 可以被调用多次
    println!("c1 call once: {:?}", c("qiao".into()));
    println!("c1 call twice: {:?}", c("bonjour".into()));

    // 然而一旦它被当成 FnOnce 被调用，就无法被再次调用
    println!("result: {:?}", call_once("hi".into(), c));
    // 无法再次调用
    // let result = c("hi".to_string());

    // Fn 也可以被当成 FnOnce 调用，只要接口一致就可以
    println!("result: {:?}", call_once("hola".into(), not_closure));
}

fn call_once(arg: String, c: impl FnOnce(String) -> (String, String)) -> (String, String) {
    c(arg)
}

fn not_closure(arg: String) -> (String, String) {
    (arg, "Rosie".into())
}
```



## 2.2 FnMut trait

[FnMut ](https://doc.rust-lang.org/std/ops/trait.FnMut.html)以`可变借用`的方式捕获了环境中的值，因此可以修改捕获变量的值



**定义如下**

```rust
pub trait FnMut<Args>: FnOnce<Args> {
    extern "rust-call" fn call_mut(
        &mut self, 
        args: Args
    ) -> Self::Output;
}
```

1. FnMut “继承”了 FnOnce，所以 FnMut 也拥有 Output 这个关联类型和 call_once 这个方法；一个 FnMut 闭包，可以被传给一个需要 FnOnce 的上下文，此时调用闭包相当于调用了 call_once()
2. 它还有一个 call_mut() 方法，注意 call_mut() 传入 `&mut self`，它不移动 self，所以 FnMut 可以被多次调用



**例子：修改闭包捕获的值**

```rust
fn main() {
    let mut name = String::from("hello");
    let mut name1 = String::from("hi");

    // 捕获 &mut name，修改了捕获的 name 
    let mut c = || {
        name.push_str(" zhangsan");
        println!("c: {}", name);
    };

    // 捕获 mut name1，注意 name1 需要声明成 mut，修改了捕获的 name1
    let mut c1 = move || {
        name1.push_str("!");
        println!("c1: {}", name1);
    };

    c();
    c1();

    // FnMut 可以被多次调用，因为 call_mut() 使用的是 &mut self，不移动所有权
    call_mut(&mut c);
    call_mut(&mut c1);

    // c 和 c1 这两个符合 FnMut 的闭包，也能作为 FnOnce 来调用
    call_once(c);
    call_once(c1);
}

// 在作为参数时，FnMut 也要显式地使用 mut，或者 &mut
fn call_mut(c: &mut impl FnMut()) {
    c();
}

fn call_once(c: impl FnOnce()) {
    c();
}
```



## 2.3 Fn trait

[Fn](https://doc.rust-lang.org/std/ops/trait.Fn.html)以`不可变借用`的方式捕获环境中的值 



**定义如下**

```rust
pub trait Fn<Args>: FnMut<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}
```

Fn“继承”了 FnMut，意味着任何需要 FnOnce 或者 FnMut 的场合，都可以传入满足 Fn 的闭包。



**例1：此例子会编译报错，因为闭包实现的是 FnMut 特征，需要的是可变借用，但是在 exec 中却给它约束为 Fn trait ，因此会报错**

```rust
fn main() {
    let mut s = String::new();

    let update_string =  |str| s.push_str(str);

    exec(update_string);

    println!("{:?}",s);
}

fn exec<'a, F: Fn(&'a str)>(mut f: F)  {
    f("hello")
}
```

改正如下：

```rust
fn main() {
    let s = "hello, ".to_string();

    let update_string =  |str| println!("{},{}",s,str);

    exec(update_string);

    println!("{:?}",s);
}

fn exec<'a, F: Fn(String) -> ()>(f: F)  {
    f("world".to_string())
}
```

因为无需改变 s，因此闭包中只对 s 进行了不可变借用，那么在 exec 中，将其约束为 Fn  trait就完全正确



**例2：**

```rust
fn main() {
    let v = vec![0u8; 1024];
    let v1 = vec![0u8; 1023];

    // Fn 不移动所有权
    let mut c = |x: u64| v.len() as u64 * x;
    // Fn 移动所有权
    let mut c1 = move |x: u64| v1.len() as u64 * x;

    println!("direct call: {}", c(2));
    println!("direct call: {}", c1(2));

    println!("call: {}", call(3, &c));
    println!("call: {}", call(3, &c1));

    println!("call_mut: {}", call_mut(4, &mut c));
    println!("call_mut: {}", call_mut(4, &mut c1));

    println!("call_once: {}", call_once(5, c));
    println!("call_once: {}", call_once(5, c1));
}

fn call(arg: u64, c: &impl Fn(u64) -> u64) -> u64 {
    c(arg)
}

fn call_mut(arg: u64, c: &mut impl FnMut(u64) -> u64) -> u64 {
    c(arg)
}

fn call_once(arg: u64, c: impl FnOnce(u64) -> u64) -> u64 {
    c(arg)
}
```

## 2.4 move 和 Fn

**一个闭包实现了哪种 Fn trait 取决于该闭包如何使用被捕获的变量，而不是取决于闭包如何捕获它们**，跟是否使用 `move` 没有必然联系。实际上使用了 move 的闭包依然可能实现了 Fn  或 FnMut，不一定是FnOnce



例子：

```rust
fn main() {
    let s = String::new();

    let update_string =  move || println!("{}",s);

    exec(update_string);
}

// FnOnce
fn exec<F: FnOnce()>(f: F)  {
    f()
}
```

闭包中使用了 move 关键字，所以我们的闭包捕获了它；但是由于闭包对s 的使用仅仅是不可变借用，该闭包不仅仅实现了 FnOnce 特征，还实现了 Fn 特征，改成以下代码也能正确运行

```rust
fn main() {
    let s = String::new();

    let update_string =  move || println!("{}",s);

    exec(update_string);
}

// 约束为Fn
fn exec<F: Fn()>(f: F)  {
    f()
}
```



## 2.5 三种闭包类型的关系

实际上，一个闭包并不仅仅实现某一种  Fn trait，规则如下：

1. 所有的闭包都自动实现了 `FnOnce trait` ，因此任何一个闭包都至少可以被调用一次

2. 没有移出所捕获变量的所有权的闭包，自动实现了 `FnMut trait` 

3. 不需要对捕获变量进行改变的闭包，自动实现了 `Fn trait` 



**例1：该例子中的闭包只是对 s 进行了不可变借用，实际上，它可以适用于任何一种类型**

```rust
fn main() {
    let s = String::new();

    let update_string =  || println!("{}",s);

    exec(update_string);
    exec1(update_string);
    exec2(update_string);
}

fn exec<F: FnOnce()>(f: F)  {
    f()
}

fn exec1<F: FnMut()>(mut f: F)  {
    f()
}

fn exec2<F: Fn()>(f: F)  {
    f()
}
```



**例2：该例子说明了第二条规则**

```rust
fn main() {
    let mut s = String::new();

    let update_string = |str| -> String {s.push_str(str); s };

    exec(update_string);
}

fn exec<'a, F: FnMut(&'a str) -> String>(mut f: F) {
    f("hello");
}
```

闭包从捕获环境中移出了变量 s 的所有权，因此这个闭包仅自动实现了 FnOnce，未实现 FnMut 和 Fn。



# 3 闭包的3种使用场景

## 3.1 作为参数传递给函数

**场景1：在函数的参数中使用闭包，是闭包一种非常典型的用法**



比如 Iterator trait 里面大部分函数都接收一个闭包，比如 map方法

```rust
fn map<B, F>(self, f: F) -> Map<Self, F>
where
    Self: Sized,
    F: FnMut(Self::Item) -> B,
{
    Map::new(self, f)
}
```

Iterator 的 map() 方法接收一个 FnMut，它的参数是 Self::Item，返回值是没有约束的泛型参数 B。Self::Item 是 Iterator::next() 方法返回的数据，被 map 之后，可以得到另一个结果。



## 3.2 作为返回值被函数返回

**场景2：闭包也可以作为函数的返回值**



```rust
use std::ops::Mul;

fn main() {
    let c1 = curry(5);
    println!("5 multiply 2 is: {}", c1(2));

    let adder2 = curry(3.14);
    println!("pi multiply 4^2 is: {}", adder2(4. * 4.));
}

fn curry<T>(x: T) -> impl Fn(T) -> T
where
    T: Mul<Output = T> + Copy,
{
    move |y| x * y // 返回一个闭包
}
```



## 3.3 为闭包实现某个trait

**场景3：闭包还有一种用法：为它实现某个 trait，使其也能表现出其他行为，而不仅仅是作为函数被调用。比如说有些接口既可以传入一个结构体，又可以传入一个函数或者闭包。**



看一个 [tonic](https://github.com/hyperium/tonic)（Rust 下的 gRPC 库）的例子：

```rust
pub trait Interceptor {
    /// Intercept a request before it is sent, optionally cancelling it.
    fn call(&mut self, request: crate::Request<()>) -> Result<crate::Request<()>, Status>;
}

impl<F> Interceptor for F
where
    F: FnMut(crate::Request<()>) -> Result<crate::Request<()>, Status>,
{
    fn call(&mut self, request: crate::Request<()>) -> Result<crate::Request<()>, Status> {
        self(request)
    }
}
```

Interceptor 有一个 call 方法，它可以让 gRPC Request 被发送出去之前被修改，一般是添加各种头，比如 Authorization 头。

我们可以创建一个结构体，为它实现 Interceptor，不过大部分时候 Interceptor 可以直接通过一个闭包函数完成。为了让传入的闭包也能通过 `Interceptor::call() `来统一调用，可以为符合某个接口的闭包实现 Interceptor trait。



# 参考

* [陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/424009)
* [Rust语言圣经-闭包](https://course.rs/advance/functional-programing/closure.html)

