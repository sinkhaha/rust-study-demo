# 1 闭包

闭包可以简单理解为一种匿名函数，它可以赋值给变量，也可以作为参数传递给其它函数；不同于函数的是，闭包可以捕获环境中的`自由变量`，即使用闭包所在函数的变量，使变量成为闭包类型的一部分。



## 1.1 闭包的表示形式

在 `Rust` 中，闭包的表示形式如下，闭包会借入对捕获变量的引用

```rust
|param1, param2,...| {
    语句1;
    语句2;
    返回表达式 // 闭包中最后一行表达式返回的值，就是闭包执行后的返回值
}

// 如果只有一个返回表达式，可以用
|param1| 返回表达式
```

还可以在闭包前加上 `move` 关键字，表示把变量的所有权从当前作用域移动到闭包的作用域

```rust
move |param1, param2,...| {
    语句1;
    语句2;
    返回表达式 
}
```



**例1：**

```rust
fn main() {
    let a = "Hello";
    let b = "zhangsan";

    let bibao = |msg: &str| {
        // 捕获自由变量a和b
        println!("{}, {}: {}", a, b, msg);
    };

    bibao("how are you?"); // Hello, zhangsan: how are you?% 
}
```

* 当创建闭包时，闭包捕获了上下文中的 `a` 和 `b` 变量，并通过`引用`来使用这两个自由变量。因为闭包也遵循借用和生命周期的规则，所以 `Rust` 不会让闭包的生命周期超出 `a` 和 `b` 的生命周期
* `Rust` 会自动从闭包的使用方式中推断出其参数类型和返回类型



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/rust-chengxusheji/%E9%97%AD%E5%8C%85%E6%8D%95%E8%8E%B7%E8%87%AA%E7%94%B1%E5%8F%98%E9%87%8F.drawio.png)



**例2：**

使用 `move` 关键字，创建新线程的 `thread::spawn` 方法的参数就是一个闭包，其定义如下

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T> 
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
```

1. `F: FnOnce() -> T`：表明 `F` 是一个接受 `0` 个参数、返回 `T` 的闭包
2. `F: Send + 'static`：表示闭包 `F`，需要静态生命周期 或 拥有所有权，并且它还能被发送给另一个线程，说明被闭包 `F` 捕获的变量，也需要静态生命周期或者拥有所有权
3. `T: Send + 'static`：表示闭包 `F` 返回的数据结构 `T`，它能被发送给另一个线程，也需要静态生命周期或者拥有所有权



先看一个编译错误的例子

```rust
use std::thread;

fn main() {
    let s = String::from("zhangsan");
    let y = String::from("lisi");

    let key_fn = || {
        println!("hello: {:?}", s);
    };

    let handle = thread::spawn(|| {
        key_fn();
        println!("hello: {:?}", y);
    });

    handle.join().unwrap();
}
```

该代码存在的问题

* 闭包 `key_fn` 包含对 `s` 的引用，因为在这里闭包是传给了线程，它会在新的线程里被调用，所以不能保证此引用的安全
* `thread::spawn` 创建的新线程无法保证 `y` 和 闭包里的 `s` 被销毁之前，闭包能完成其工作，因为闭包的生命周期不能超过其捕获的变量的生命周期



从 `thread::spawn` 的定义可知，需要使用 `move` 关键字，把变量的所有权从当前作用域移动到闭包的作用域，改正如下

```rust
use std::thread;

fn main() {
    let s = String::from("zhangsan");
    let y = String::from("lisi");

    // 这是一个闭包，move会移动s的所有权给闭包，而不是传入s的引用
    let key_fn = move || {
        let rst = format!("hello: {}", s);
        rst
    };

    // 这里也传入一个闭包，并在新的系统线程中调用闭包，move会移动key_fn 和 y 的所有权
    // 新线程会和调用者并行运行，当闭包返回时，新线程退出（闭包的返回值会作为 JoinHandle 值发送回调用线程）
    let handle = thread::spawn(move || {
        let rst = key_fn();
        println!("{:?}", rst);
        println!("hello: {:?}", y);
    });

    // 这里访问s和y会报错，因为y的所有权已经被移动到线程的闭包参数里了
    // println!("hello: {:?}", y);
    // println!("hello: {:?}", s);

    handle.join().unwrap();

}

// 输出
// "hello: zhangsan"
// hello: "lisi"
```

由代码可知，第一个 `key_fn` 闭包获得了 `s` 的所有权；第二个闭包获得了 `y` 和 `key_fn` 的所有权。

* 如果闭包使用 `move` 后要移动的是可复制类型的值（如 `i32`），那么就会复制该值，这样即使创建了闭包，后面仍然可以使用捕获的变量
* 如果闭包要移动的是不可复制类型的值（如`Vec`），那么就会移动，上面闭包就把 `s` 移动给了新线程，后面调用处的代码就不能再继续使用 `s` 了。也可以先克隆 `s` 并将副本存储在另一个变量中，闭包可以传进去副本变量



## 1.2 闭包的本质

闭包是一种匿名类型，一旦声明，就会产生一个新的类型，但这个类型无法被其它地方使用。这个类型就像一个结构体，会包含所有捕获的变量。 



`Rust` 中闭包产生的匿名数据类型，格式和 `struct` 是一样的。闭包是存储在栈上，并且除了捕获的数据外，闭包本身不包含任何额外函数指针指向闭包的代码。闭包的大小跟参数、局部变量都无关，只跟捕获的变量有关。



**例子：**

闭包的大小只跟捕获的变量有关

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

* `c1` 没有参数，也没捕获任何变量，从代码输出可以看到，`c1` 长度为 0
* `c2` 有一个 `i32` 作为参数，没有捕获任何变量，长度也为 0，可以看出参数跟闭包的大小无关
* `c3` 捕获了一个对变量 `name` 的引用，这个引用是 `&String`，长度为 8。而 `c3` 的长度也是 8
* `c4` 捕获了变量 `name1` 和 `table`，由于用了 `move`，它们的所有权移动到了 `c4` 中。`c4` 长度是 72，恰好等于 `String` 的 24 字节，加上 `HashMap` 的 48 字节
* `c5` 捕获了 `name2`，`name2` 的所有权移动到了 `c5`，虽然 `c5` 有局部变量，但它的大小和局部变量也无关，`c5` 的大小等于 `String` 的 24 字节



## 1.3 闭包类型和函数

函数和闭包都能被当成值来使用。它们有自己的类型，但是不一样，函数是 `fn` 类型，但闭包不是，闭包是 `Fn` 或 `FnOnce` 或 `FnMut` 类型。



**例如**

下面函数的类型是 `fn(&str) -> String`，注意这里是小写的 `fn`。

```rust
// 函数的类型为 fn(&str) -> String
fn hello(val: &str) -> String {
    let rst = format!("hello: {}", val);
    rst
}
```

一个函数可以作为另一个函数的参数，如

```rust
// 第2个参数是函数类型
fn test(val: &str, test_fn: fn(&str) -> String) -> String {
    test_fn(val)
}

// 这个函数的类型是fn(&str) -> String
fn hello(val: &str) -> String {
    let rst = format!("hello: {}", val);
    rst
}

fn main() {
    let s = String::from("zhangsan");

    // 第2个参数传入hello函数
    test(&s, hello);
}
```

闭包的类型跟函数不一样，不能直接用上面 `test` 函数的定义，错误用法如下

```rust
fn main() {
    let s = String::from("zhangsan");

    let key_fn = |val| {
        let rst = format!("hello: {}", val);
        rst
    };
  
    // 调用test函数，第2个参数传入闭包
    // 这里会报错，类型不匹配
    test(&s, key_fn);
}
```

想支持这个闭包，必须修改 `test` 函数参数的类型，改正如下

```rust
fn test<'a, F>(val: &'a str, test_fn: F) -> String
where
    F: Fn(&'a str) -> String,
{
    test_fn(val)
}

fn main() {
    let s = String::from("zhangsan");

    let key_fn = |val| {
        let rst = format!("hello: {}", val);
        rst
    };
    
    // 传入闭包
    test(&s, key_fn);
}
```

* 这里改成泛型函数，函数体没有改动。该数就能接收任意 `F` 型的 `test_fn`，`where` 限制表示 `F` 实现了特定的特型 `Fn(&str) -> String` ，因为以单个 `&str` 为参数并返回` String` 值的所有函数和大多数闭包会自动实现这个 `Fn(&str) -> String` 特型

* 这里还加了个 `'a` 的生命周期标识，因为 `test` 函数的 `val` 和 `test_fn` 函数的参数有相同的生命周期

* `fn(&str) -> String`  ：`fn` 类型，只接受函数

* `Fn(&str) -> String` ：`Fn` 特型，可以接受函数也可以接受闭包

  > -> 是可选的，如果没有返回值可以省略，则返回类型为()，`Fn()` 是 `Fn() -> ()` 的简写形式



## 1.4 闭包性能

`Rust` 中闭包的设计是要快，比函数指针还要快。

大多数语言，闭包会在堆中分配内存、进行动态派发以及垃圾回收，这样会有一些缺陷

* 创建、调用和收集每一个闭包都会花费额外的 `cpu` 时间
* 闭包难以内联，内联是编译器用来消除函数调用开销并实施大量其他优化的关键技术

`Rust` 中，闭包则没有上面这些缺陷，它们没有垃圾回收。闭包和其他类型一样，除非将闭包放在 `Box`、`Vec` 或其他容器中，否则它们不会被分配到堆上。编译器只要知道正在调用的闭包的类型，就可以内联该闭包的代码



闭包最大的问题是变量的多重引用导致生命周期不明确，但 `Rust` 从根本上使用所有权和借用，解决了这个问题

* 如果不使用 `move` 转移所有权，闭包会引用上下文中的变量，这个引用受借用规则的约束，所以只要编译通过，那么闭包对变量的引用就不会超过变量的生命周期，没有内存安全问题

* 如果使用 `move` 转移所有权，上下文中的变量在转移后就无法访问，闭包完全接管这些变量，它们的生命周期和闭包一致，所以也不会有内存安全问题



# 2 三种闭包类型

当闭包作为函数的参数 或者 数据结构的一个域时，需要告诉调用者，对闭包的约束。因为`函数参数`有三种传入方式：转移所有权、可变借用、不可变借用，所以相应的闭包类型也有三种：`FnOnce`、`FnMut`、`Fn`。比如以 `thread::spawn` 为例，它要求传入的闭包满足 `FnOnce trait`。



## 2.1 FnOnce trait

[FnOnce](https://doc.rust-lang.org/std/ops/trait.FnOnce.html) 类型的闭包不仅会拿走被捕获变量的所有权，还会拿走闭包自身的所有权，所以 `FnOnce` 类型的闭包只能被调用一次，当再次调用时，编译器会报错。



**定义如下**

```rust
pub trait FnOnce<Args> {
    type Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}
```

1. `FnOnce` 的参数是一个叫 `Args` 的泛型参数，它没有任何约束

2. 有一个关联类型 `Output`，它是闭包返回值的类型

3. 有一个方法 `call_once` ，对于 `FnOnce` 闭包，会将 `closure()` 扩展为 `closure.call_once()`，要注意的是 `call_once` 第一个参数是 `self`，它会转移 `self（即闭包）` 的所有权到 `call_once` 函数中，所以在第一次调用 `FnOnce` 闭包后，闭包本身也会被消耗掉，后面继续调用闭包会报错。

   

**例1：**

隐式的 `FnOnce` 例子。因为使用了 `move` 关键字，闭包会转移捕获的变量的所有权，所以它是一个 `FnOnce` 的闭包，一旦它被当做 `FnOnce 调用`，闭包自己会被转移到 `call_once` 函数的作用域中，之后就无法再次调用了，所以闭包 `c` 只能调用一次

```rust
fn main() {
    let name = String::from("zhangsan");
  
    // 这个闭包啥也不干，只是把捕获的参数返回
    let c = move |greeting: String| (greeting, name);

    let result = c("hello".to_string());

    println!("result: {:?}", result);

    // 再次调用c闭包会报错
    // c("hi".to_string());
}
```



**例2：**

如果一个闭包并不转移自己的内部数据，那它就不是 `FnOnce`

```rust
fn main() {
    let name = String::from("zhangsan");

    // 这个闭包会 clone 内部的数据返回，所以它不是 FnOnce，它实际时 Fn 类型
    let c = move |greeting: String| (greeting, name.clone());

    // 此时 c 可以被调用多次
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

[FnMut ](https://doc.rust-lang.org/std/ops/trait.FnMut.html)以`可变引用`的方式捕获了环境中的值，因此可以修改捕获变量的值，也就是可写入的闭包。

> 任何需要对值进行可变访问，但不会丢弃任何值的闭包都是 `FnMut` 闭包

`Rust` 认为不可变值可以安全地跨线程共享，但是包含“可变数据”的“不可变闭包”不能安全共享。因为从多个线程调用这样的闭包可能会导致各种竞态条件，多个线程会试图同时读取和写入同一份数据。



**定义如下**

```rust
pub trait FnMut<Args>: FnOnce<Args> {
    extern "rust-call" fn call_mut(
        &mut self, 
        args: Args
    ) -> Self::Output;
}
```

1. `FnMut`  继承了 `FnOnce`，所以 `FnMut` 也拥有 `Output` 这个关联类型和 `call_once` 这个方法；一个 `FnMut` 闭包，可以被传给一个需要 `FnOnce` 的上下文，此时调用闭包相当于调用了 `call_once()`
2. 它还有一个 `call_mut()` 方法，注意 `call_mut()` 传入 `&mut self`，它不移动 `self`，所以 `FnMut` 可以被多次调用



**例1：**

修改闭包捕获的值

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

[Fn](https://doc.rust-lang.org/std/ops/trait.Fn.html) 以`不可变引用`的方式捕获环境中的值 



**定义如下**

```rust
pub trait Fn<Args>: FnMut<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}
```

* `Fn`  继承了 `FnMut`，意味着任何需要 `FnOnce` 或者 `FnMut` 的场合，都可以传入满足 `Fn` 的闭包

* 对于 `Fn` 闭包，`Rust` 会将上面的 `closure()` 扩展为 `closure.call()` ，此方法会通过引用获取 `self`，因此闭包不会被移动



**例1：**

下例子会编译报错，因为闭包实现的是 `FnMut` 特征，需要的是可变借用，但是在 `exec` 中却给它约束为 `Fn trait` ，因此会报错

```rust
fn main() {
    let mut s = String::new();

   // FnMut类型的闭包
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
    let mut s = String::new();

    // FnMut类型的闭包
    let update_string = |str| s.push_str(str);

    exec(update_string);

    println!("{:?}", s);
}

// 改成 FnMut 类型
fn exec<'a, F: FnMut(&'a str)>(mut f: F) {
    f("hello")
}
```



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



## 2.4 3种闭包类型小结

* `Fn` 是不受限制地调用任意多次的闭包和函数系列。此最高类别还包括所有 `fn` 函数
* `FnOnce` 是如果其调用者拥有此闭包，它就只能调用一次的闭包系列
* `FnMut` 是本身会被声明为 `mut`，并且可以多次调用的闭包系列

每个 `Fn` 都能满足 `FnMut` 的要求，每个 `FnMut` 都能满足 `FnOnce` 的要求，所以它们的包含关系是， `FnOnce` 包含 `FnMut`，`FnMut` 包含 `Fn` ，应该说 `Fn` 是 `FnMut` 的子特型，而 `FnMut`  是 `FnOnce`  的子特型，`Fn` 是最严格的类型。



## 2.5 闭包和移动

实际上，一个闭包并不仅仅实现某一种  `Fn trait`，所有的闭包都自动实现了 `FnOnce trait` ，因此任何一个闭包都至少可以被调用一次。



**一个闭包实现了哪种 `Fn trait` 取决于该闭包如何使用被捕获的变量，而不是取决于闭包如何捕获它们，跟是否使用 `move` 没有必然联系。**

1. 实际上使用了 `move` 的闭包依然可能实现了 `Fn`  或 `FnMut`，不一定是 `FnOnce`
2. 没有移出所捕获变量的所有权的闭包，自动实现了 `FnMut trait` 
3. 不需要对捕获变量进行改变的闭包，自动实现了 `Fn trait` 



**例1：**

该例子中的闭包只是对 `s` 进行了不可变借用，实际上，它可以适用于任何一种闭包类型

```rust
fn main() {
    let s = String::new();

    let update_string =  || println!("{}",s);

    exec(update_string);
    exec1(update_string);
    exec2(update_string);
}

// 约束为 FnOnce
fn exec<F: FnOnce()>(f: F)  {
    f()
}

// 约束为 FnMut
fn exec1<F: FnMut()>(mut f: F)  {
    f()
}

// 约束为 Fn
fn exec2<F: Fn()>(f: F)  {
    f()
}
```



**例2：**

```rust
fn main() {
    let mut s = String::new();

    // 此闭包是 FnOnce 类型
    let update_string = |str| -> String {s.push_str(str); s };

    exec(update_string); // 因为exec为FnMut类型，所以会报错
}

// 此时限制了闭包为 FnMut类型，
fn exec<'a, F: FnMut(&'a str) -> String>(mut f: F) {
    f("hello");
}

// 需要改成为FnOnce类型，此时就能正确调用
fn exec<'a, F: FnOnce(&'a str) -> String>(mut f: F) {
    f("hello");
}
```

闭包从捕获环境中移出了变量 `s` 的所有权，因此这个闭包仅自动实现了 `FnOnce`，没有实现 `FnMut` 和 `Fn`。



## 2.6 对闭包的 Copy 和 Clone

闭包是表示包含它们捕获的变量的值（对于 `move` 闭包） 或 对值的引用（对于非 `move` 闭包）的结构体。闭包的 `Copy` 和 `Clone` 规则跟常规结构体是一样的。



### 2.6.1 非 move 闭包

* 一个不修改变量的非 `move` 闭包只持有共享引用，这些引用既能 `Copy` 也能 `Clone` ，所以闭包也能 `Clone` 和 `Copy` 
* 一个会修改值的非 `move` 闭包在其内部表示中也可以有可变引用。可变引用既不能 `Clone`，也不能`Copy` ，使用它们的闭包同样不能 `Clone` 和 `Copy`，而是移动

**例1:**

```rust
fn main() {
    let y = 10;

    let add_y = |x| x + y; // Fn闭包类型

    let copy_of_add_y = add_y; // 此闭包能copy，所以这里也是copy,而非移动

    assert_eq!(add_y(copy_of_add_y(12)), 32); // 这里可以调用闭包2次
}
```

**例2：**

```rust
fn main() {
    let mut x = 0;
    let mut add_to_x = |n| { x += n; x }; // FnMut闭包

    let copy_of_add_to_x = add_to_x; // 这里会进行移动，而非复制

    assert_eq!(add_to_x(copy_of_add_to_x(1)), 2); // 这里使用两次闭包，会错误，因为使用了已经移动出去的值
}
```



### 2.6.2 move 闭包

对于 `move` 闭包，规则更简单。如果 `move` 闭包捕获的所有内容都能 `Copy`，那它就能 `Copy` 。如果 `move` 闭包捕获的所有内容都能 `Clone` ，那它就能 `Clone`。例如

```rust
fn main() {
    let mut greeting = String::from("hello,");

    // FnMut类型闭包
    let greet = move |name| {
        greeting.push_str(name);
        println!("{}", greeting);
    };

    greet.clone()("zhangsan"); // 这里 .clone表示克隆此闭包并调用其克隆体
    greet.clone()("lisi");
}

// 输出
// hello,zhangsan
// hello,lisi
```

当在 `greet` 中使用 `greeting` 时，`greeting` 被移动到了内部表示 `greet` 的结构体中，因为它是一个 `move` 闭包，所以当克隆 `greet` 时，它所有的东西同时被克隆了，`greeting` 有两个副本，它们会在调用 `greet` 的克隆时分别被修改。



# 3 闭包的使用场景

## 3.1 作为参数传递给函数

**场景1：在函数的参数中使用闭包，是闭包一种非常典型的用法**。比如 `Iterator trait` 里面大部分函数都接收一个闭包，例如 `map` 方法

```rust
fn map<B, F>(self, f: F) -> Map<Self, F>
where
    Self: Sized,
    F: FnMut(Self::Item) -> B,
{
    Map::new(self, f)
}
```

`Iterator` 的 `map()` 方法接收一个 `FnMu`t，它的参数是 `Self::Item`，返回值是没有约束的泛型参数 `B`。`Self::Item` 是 `Iterator::next() `方法返回的数据，被 `map` 之后，可以得到另一个结果。



## 3.2 作为返回值被函数返回

**场景2：闭包也可以作为函数的返回值**，例如

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

**场景3：闭包还有一种用法，为它实现某个 trait，使其也能表现出其他行为，而不仅仅是作为函数被调用。**比如说有些接口既可以传入一个结构体，又可以传入一个函数或者闭包。



看一个 [tonic](https://github.com/hyperium/tonic)（`Rust` 下的 `gRPC` 库）的例子：

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

`Interceptor` 有一个 `call` 方法，它可以让 `gRPC Request` 被发送出去之前被修改，一般是添加各种头，比如 `Authorization` 头。我们可以创建一个结构体，为它实现 `Interceptor`，不过大部分时候 `Interceptor` 可以直接通过一个闭包函数完成。为了让传入的闭包也能通过 `Interceptor::call()` 来统一调用，可以为符合某个接口的闭包实现 `Interceptor trait`。



# 4 参考

* [Rust程序设计（第2版）](https://book.douban.com/subject/36547630/)
* [陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/424009)
* [Rust语言圣经-闭包](https://course.rs/advance/functional-programing/closure.html)

