# 错误处理的主流方法

## 1、使用返回值

使用返回值来表示错误，如C语言和Golang

> 例如，在 C 语言中，如果 fopen(filename) 无法打开文件，会返回 NULL，调用者通过判断返回值是否为 NULL，来进行相应的错误处理



**缺点**

1. 在C语言中，没区分开错误返回和正常返回；Golang则对其做了扩展，在函数返回时，可以携带一个错误对象，区分开了错误返回和正常返回

2. 返回值有它原本的语义，对于开发者来说，需要实时的区别对待正常返回和错误返回有一定的负担

3. 在调用时，错误必须得到处理或者显式的传播，不处理可能会造成隐患

4. 大部分生产环境下的错误是嵌套的，更深层的内部错误信息很难追溯

   

## 2、使用异常

使用异常来处理错误，如JAVA，程序中任何可能出错的地方，都可以抛出异常。

> 异常可以通过栈回溯被一层层自动传递，直到遇到捕获异常的地方，如果回溯到 main 函数还没捕获，程序就会崩溃



**优点**

1. 使用异常来返回错误极大地简化错误处理的流程，解决了返回值的传播问题
2. 在大多数情况下，用异常更容易写代码



**缺点**

1. 开发者会滥用异常

   > 只要有错误，不论是否严重、是否可恢复，都抛个异常，然后捕获一下一了了之。异常处理的开销要比处理返回值大，滥用会有很多额外的开销。

2. 当异常安全无法保证时，程序的正确性会受到很大的挑战

   > 在使用异常处理时，需要特别注意异常安全，尤其是在并发环境下。
   >
   > 
   >
   > 看下面用来切换背景图片的（伪）代码，用于演示异常安全问题：
   >
   > ```java
   > void transition(...) {
   >   lock(&mutex);
   >   delete background;
   >   ++changed;
   >   
   >   background = new Background(...); // 如果发生错误？
   >   
   >   unlock(&mutex);
   > }
   > ```
   >
   > 如果在创建新的背景时失败，抛出异常，会跳过后续的处理流程，一路栈回溯到 try catch 的代码，那么，这里锁住的 mutex 无法得到释放，而已有的背景被清空，新的背景没有创建，程序进入到一个奇怪的状态。
   >



  ## 3、使用类型系统(Rust)

错误信息可以通过已有的类型携带，使用一个内部包含正常返回类型和错误返回类型的复合类型，通过类型系统来强制错误的处理和传递，如 Haskell/Scala/Swift/Rust。

> 最典型的包含了错误类型的复合类型是 Haskell 的 Maybe 和 Either 类型，Rust也是参考了Haskell



**优点**

1. 可以用函数式编程的方法简化错误的处理
2. 这种方法依旧是通过返回值返回错误，但是错误被包裹在一个完整的、必须处理的类型中，比 Golang 的方法更安全



# Rust可恢复的错误:Option/Result

Rust中处理错误的方式： Option 和 Result



## Option

**Option定义**

[Option](https://doc.rust-lang.org/std/option/enum.Option.html)是一个 enum

```rust
pub enum Option<T> {
    None, // 表示没有值
    Some(T), // 表示有值，值的类型为T
}
```



**使用例子如下**

```rust
fn main() {
    // 除数不为0的情况
    let result1 = divide(2.0, 3.0);
    // 匹配并处理错误
    match result1 {
        Some(x) => println!("结果 {}", x), // 结果 0.6666666666666666
        None => println!("除数为0"), 
    }

    // 除数为0的情况
    let result2 = divide(2.0, 0.0);
    match result2 {
        Some(x) => println!("结果 {x}"),
        None => println!("除数为0"), // 除数为0
    }
  
    // 也可以直接用unwrap取值（项目中不建议使用）
    // 除数不为0
    // let result3 = divide(2.0, 3.0).unwrap();
    // println!("结果 {}", result3); // // 结果 0.6666666666666666
}

/**
 * 求商函数，返回Option
 */
fn divide(numerator: f64, denominator: f64) -> Option<f64> {
    if denominator == 0.0 {
        None
    } else {
        Some(numerator / denominator)
    }
}
```



## Result

**Result定义**

[Result](https://doc.rust-lang.org/std/result/enum.Result.html)是一个比Option更复杂的enum，定义如下：

```rust
#[must_use = "this `Result` may be an `Err` variant, which should be handled"]
pub enum Result<T, E> {
    Ok(T), // 返回成功
    Err(E), // 返回错误
}
```



**使用例子如下**

例1：此例子用于演示`错误的使用方式`

> 没有处理Result的返回值，编译时会报警

```rust
use std::io::Error;

fn main() {
    let path = "/tmp/file.txt";
    read_file(path); // 没有处理返回值，此时编译会报警
}

fn read_file(path: &str) -> Result<String, Error> {
    std::fs::read_to_string(path)
}
```

使用`cargo run`运行，输出如下

```bash
warning: unused `Result` that must be used
  --> src/main.rs:12:5
   |
12 |     read_file(path);
   |     ^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(unused_must_use)]` on by default
   = note: this `Result` may be an `Err` variant, which should be handled

```

因为Result类型声明时有个 `#[must_use] ` 标注，如果该类型返回的值没有被显式使用，编译时会告警，确保错误能被妥善处理。



例2：此例子演示正确的使用方式

```rust
use std::io::Error;

fn main() {
    let path = "/tmp/file.txt";

    let rst = read_file(path);
    
    // 匹配对应返回值并处理
    match rst {
        Ok(file) => { println!("file={}", file) }
        Err(e) => { println!("Error reading file path={} e={}", path, e)}
    }
}

fn read_file(path: &str) -> Result<String, Error> {
    std::fs::read_to_string(path)
}
```

假设此时path路径不存在，使用`cargo run`运行，此时发生错误，会被match匹配到错误，输出如下

```bash
Error reading file path=/tmp/file.txt e=No such file or directory (os error 2)
```



**Rust 还为 Option 和 Result 提供了大量的辅助函数，如 map / map_err / and_then，可以方便的用函数式编程进行错误处理。**



## Option和Result的unwrap/expect方法

Option和Result类型都有一个unwrap和expect方法。



**unwrap和except的作用**

如果Option或Result返回成功，可以从成功结果中直接取到值，如果失败，则触发panic!不可恢复错误。expect可以自定义错误信息，能更友好的给到开发者提示。

**使用例子如下**

```rust
use std::io::Error;

fn main() {
    let path = "/tmp/file.txt";
    // 如果路径不存在，对Result使用unwrap强制转成T会触发panic
    let rst = read_file(path).unwrap();
  
    // 也可以用expect，可以带上自定义的错误信息提示
    // let rst = read_file(path).expect("读取文件失败");

    println!("结果 {}", rst);
}

fn read_file(path: &str) -> Result<String, Error> {
    std::fs::read_to_string(path)
}
```



**使用场景**

1. 如编写“示例demo”或“测试”等一些不需要处理错误的场景，此时我们的逻辑确保了Result一定是返回成功Ok值，肯定不会发生错误的情况下可直接用unwrap

   > 因为如果使用Result类型，match在匹配时，必须分别处理Ok和Err两种情况，比较繁琐，特殊的场景用unwrap简单很多

2. 项目中建议不使用unwrap，因为代码中有bug可能会触发panic!，使得程序崩溃



# Rust不可恢复的错误

## panic！

Rust 中提供了特殊的异常处理能力`panic!`；`panic!`是不可恢复的错误，发生后程序会停止运行。



**使用方式如下**

```rust
fn main() {
    panic!("这是错误"); // 发生panic后，程序直接停止
}
```

可以在运行时，加上`RUST_BACKTRACE=1`环境变量，作用是可以显示回溯，它会展开运行的栈并输出所有的信息。

> 即运行 `RUST_BACKTRACE=1 cargo run `



## catch_unwind()

std::panic模块有一个catch_unwind() 方法。作用是可以把调用栈回溯到发生panic的地方，和其它语言的 `try {} catch {}` 一样。



**使用例子如下**

把 Rust 代码整个封装在 catch_unwind() 函数所需要传入的闭包中。一旦代码含有导致 panic! 的代码(包括第三方 crates 的代码)，都会被捕获，并被转换为一个 Result。

```rust
use std::panic;

fn main() {
    let result = panic::catch_unwind(|| {
        println!("hello!");
    });
    assert!(result.is_ok());
  
    let result = panic::catch_unwind(|| {
        panic!("oh no!");
    });
    assert!(result.is_err());
  
    println!("panic captured: {:#?}", result);
}
```



**使用场景：**

* 在其他语言中嵌入 Rust（不希望rust的任何panic!导致其他语言在运行时崩溃）
* 测试框架（测试时可能引起崩溃，但是我们不希望崩溃）



# 错误传播

## ?操作符

?操作符的作用：传播错误。Result使用 ? 操作符时，其内部被展开成如下类似的代码：

```rust
match result {
  Ok(v) => v,
  Err(e) => return Err(e.into())
}
```



**使用方式如下**

?操作符只能使用在以Option或者Result作为返回值的函数体中。

```rust
use std::fs::File;
use std::io::Read;

fn read_file(name: &str) -> Result<String, std::io::Error> {
  let mut f = File::open(name)?;
  let mut contents = String::new();
  f.read_to_string(&mut contents)?;
  Ok(contents)
}
```

注意：在不同的错误类型之间无法直接使用“?操作符”的，需要实现 From trait 在二者之间建立起转换的桥梁(具体看下面的`错误类型的转换`)



# 自定义错误类型

## Error trait

Result 里 E 是一个代表错误的数据类型。为了规范这个代表错误的数据类型的行为，Rust 中[Error trait](https://doc.rust-lang.org/std/error/trait.Error.html)定义如下：

```rust
pub trait Error: Debug + Display {
    // 有错误则返回错误的原因，没有则返回None
    fn source(&self) -> Option<&(dyn Error + 'static)> { ... }
    // 返回错误的描述信息
    fn description(&self) -> &str { ... }
    // 用于获取错误的原因
    fn cause(&self) -> Option<&dyn Error> { ... }
    fn provide(&'a self, demand: &mut Demand<'a>) { ... }
}
```



## 自定义错误类型

**使用步骤**

1. 自定义一个BusinessError表示业务错误类型
2. 自定义错误类型要实现Debug+Display，这两个trait提供了格式化输出的功能，可以更方便地输出错误信息



**使用例子如下**

```rust
use std::fmt;

// 自定义一个BusinessError业务错误类型
struct BusinessError {
    code: usize, // 根据自己需求定义属性，这里定义一个业务错误码
}

// 实现Display，根据错误码显示不同的错误信息
impl fmt::Display for BusinessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        // 不同的错误码，输出不同的错误信息
        let err_msg = match self.code {
            10001 => "参数缺失",
            10002 => "资源不存在",
            _ => "未知错误",
        };

        write!(f, "{}", err_msg)
    }
}

// 实现Debug，打印错误时，输出的错误信息
impl fmt::Debug for BusinessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BusinessError {{ code: {} }}",
            self.code
        )
    }
}

// 触发BusinessError错误的函数
fn trigger_miss_params_error() -> Result<(), BusinessError> {
    Err(BusinessError {
        code: 10001,
    })
}

fn main(){
    match trigger_miss_params_error() {
        Err(e) => eprintln!("{}", e), // 匹配到错误后输出
        _ => println!("OK"),
    }

    // 打印错误，Err(BusinessError { code: 10001 })
    eprintln!("{:?}", trigger_miss_params_error()); 
}
```



## 错误类型的转换

**From trait**

[From trait](https://doc.rust-lang.org/std/convert/trait.From.html)_定义了一个从某种类型转换到另一种类型的方法。其定义如下：

```rust
pub trait From<T> {
    fn from(T) -> Self; // 接受一个类型为 T 的参数，并返回一个 Self 类型的值
}
```

可以利用From trait进行错误类型之间的转换。要使用 From trait 实现类型转换，需要在需要转换的类型上实现 From trait。例如下面的例子，把`std::io::Error`错误类型转换成BusinessInternalError错误类型。



**使用步骤**

1. 定义一个错误类型BusinessInternalError
2. BusinessInternalError实现`From<std::io::Error>`，即把std::io::Error错误类型转换成BusinessInternalError错误类型
3. 通过?操作符，可以把std::io::Error错误类型隐式的转换成BusinessInternalError错误类型



**使用例子如下**

```rust
use std::fs::File;
use std::io;
use std::fmt;

// 定义一个业务内部错误
struct BusinessInternalError {
    kind: String, // 错误类型
    message: String, // 错误信息
}

// 是将 io::Error 错误转换成自定义的 BusinessInternalError 错误
impl From<io::Error> for BusinessInternalError {
    fn from(error: io::Error) -> Self {
        BusinessInternalError {
            kind: String::from("io"),
            message: error.to_string(),
        }
    }
}

// 实现Debug，格式化输出错误信息
impl fmt::Debug for BusinessInternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BusinessInternalError {{ kind: {} message: {} }}",
            self.kind,
            self.message
        )
    }
}

// 这里main方法的Result统一返回一个BusinessInternalError类型，不需要关心是不是std::io::Error类型
fn main() -> Result<(), BusinessInternalError> {
    // File::open返回的是std::io::Error，因为已经为BusinessInternalError实现了From trait，所以这里通过？操作符可以隐式的将io::Error错误类型转成BusinessInternalError
    File::open("tmp.txt")?;
    Ok(())
}
```



# 统一化不同的错误类型

在项目中，我们定义了多个不同的错误类型，怎么在一个函数中怎么返回不同的错误类型？

> 可以使用`Boy<dyn Error>` 或 自定义错误类型



## `Boy<dyn Error>`

`Boy<dyn Error>`它表示一个`指向实现了 Error trait 的类型`的智能指针。dyn关键字表示动态类型，它的作用是定一个变量的类型是动态类型，即在编译时无法确定类型。



**使用例子如下**

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  let content = get_config_content()?;
  println!("{}", content);
  Ok(())
}

// 返回Box<dyn Error>
fn get_config_content() -> Result<String, Box<dyn Error>> {
    // 获取系统的环境变量CONFIG_FILE，可能会发生VarError错误
    let file = std::env::var("CONFIG_FILE")?;
    // 读取文件的内容，可能会发生错误std::io::Error
    let content = std::fs::read_to_string(file)?;
    Ok(content)
}
```



**缺点：**

1. `Box<dyn Error>` 类型是动态类型，它的类型信息会在运行时丢失，会导致在处理错误时，无法根据类型来处理不同的错误情况

2. 它不能包含错误码

   > 由于 `Box<dyn Error>` 类型只能存储一个智能指针，它并不能存储错误码


> 可以使用下面的“自定义错误类型”解决`Box<dyn Error>` 的缺点



## 更完善的自定义错误类型

**使用例子如下**

```rust
use std::fs::read_to_string;
use std::error::Error;

// 定义错误类型MyCustomError
#[derive(Debug)]
enum MyCustomError {
    EnvironmentVariableNotFound,
    IOError(std::io::Error),
}

// 自定义错误类型MyCustomError实现Error trait后，才能转换成相应的特征对象
impl Error for MyCustomError {}

// 返回MyCustomError错误
fn get_config_content() -> Result<String, MyCustomError> {
   let file = std::env::var("CONFIG_FILE")?;
   let content = read_to_string(file)?;
   Ok(content)
}

// 把VarError转成MyCustomError
impl From<std::env::VarError> for MyCustomError {
    fn from(_: std::env::VarError) -> Self {
        Self::EnvironmentVariableNotFound
    }
}

// 把 std::io::Error 转成MyCustomError
impl From<std::io::Error> for MyCustomError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

// 根据错误码显示不同的错误信息
impl std::fmt::Display for MyCustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 根据不同的错误类型，输出不同的错误信息
        match self {
            MyCustomError::EnvironmentVariableNotFound => write!(f, "环境变量不存在"),
            MyCustomError::IOError(err) => write!(f, "IO错误: {}", err.to_string()),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = get_config_content()?;
    println!("{}", content);
    Ok(())
}
```



**缺点：**

处理起来稍微麻烦，推荐使用第三方的thiserror库进行简化



# thiserror和anyhow库

## thiserror库

**thiserror的作用**

提供了一个派生宏来简化自定义错误类型的过程



**使用步骤**

1、先在Cargo.toml添加依赖

```rust
[dependencies]
thiserror = "1.0"
```

2、例子

* 通过派生宏`#[derive(thiserror::Error)]`来定义自定义错误类型MyCustomError
* #[error]属性：提供了错误消息的格式化功能
* #[from]属性：实现错误类型的转换，`#[from] std::io::Error`即表示IOError是从std::io::Error转换而来
* transparent：表示错误类型是一个透明类型，透明类型是指错误类型与实际错误原因相同

```rust
use std::fs::read_to_string;

#[derive(thiserror::Error, Debug)]
enum MyCustomError {
    #[error("环境变量不存在")]
    EnvironmentVariableNotFound(#[from] std::env::VarError),
    #[error(transparent)]
    IOError(#[from] std::io::Error), 
}

// 方法里可能会发生VarError或std::io::Error错误，都是通过?操作符，转换成MyCustomError错误返回
fn get_config_content() -> Result<String, MyCustomError> {
   // 获取系统的环境变量CONFIG_FILE，变量不存在会发生VarError错误
   let file = std::env::var("CONFIG_FILE")?;
   // 读取文件的内容，文件不存在会发生错误
   let content = read_to_string(file)?;
   Ok(content)
}

fn main() -> Result<(), MyCustomError> {
    let content = get_config_content()?;
    println!("{}", content);
    Ok(())
}
```



## anyhow库

**anyhow的作用**

和thiserror库一样，也是简化定义自定义错误类型的过程。它提供了一个可以包含任何错误类型的统一错误类型 `anyhow::Error`，支持将所有实现了Error trait 的自定义错误类型都转换为`anyhow::Error`类型，可以直接使用 ? 操作符完成这个转换，不必手工转换错误类型



**使用步骤**

1、先在Cargo.toml添加依赖

```toml
[dependencies]
anyhow = "1.0"
```

2、例子

```rust
use std::fs::read_to_string;

use anyhow::Result;

fn main() -> Result<()> {
    let content = get_config_content()?;
    println!("{}", content);
    Ok(())
}

// Result<String>等价于Result<String, anyhow::Error>
fn get_config_content() -> Result<String> {
   // 获取系统的环境变量CONFIG_FILE，可能会发生VarError错误
   let file = std::env::var("CONFIG_FILE")?;
   // 读取文件的内容，可能会发生，可能会发生错误
   let content = read_to_string(file)?;
   Ok(content)
}
```

需要返回Result时，使用`Result<T, anyhow::Error>`或者等价的`anyhow::Result<T>`，就可以用`？`抛出任何类型实现了`std::error::Error`的错误。



## thiserror和anyhow的区别

1. thiserror：提供了一些宏属性（如 `#[from]` 和 `#[error(transparent)]`），用于设计自己的专用错误类型，以便给调用者提供更具体的自定义错误信息，常用于编写第三方库中

2. anyhow提供了一个可以包含任何错误类型的统一错误类型 `anyhow::Error`，只是简单的使用，不需要让调用者关注具体的错误类型，常用于应用程序业务代码中

   

# 参考

* [Error Trait](https://doc.rust-lang.org/std/error/trait.Error.html)

* [thiserror](https://github.com/dtolnay/thiserror)

* [anyhow](https://github.com/dtolnay/anyhow)

* [细说Rust错误处理](https://rustcc.cn/article?id=75dbd87c-df1c-4000-a243-46afc8513074)

*  [陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/415988)

* [Rust语言圣经(Rust Course)-错误处理](https://course.rs/advance/errors.html#%E8%87%AA%E5%AE%9A%E4%B9%89%E9%94%99%E8%AF%AF%E7%B1%BB%E5%9E%8B)

* [Rust语言圣经(Rust Course)-返回值和错误处理](https://course.rs/basic/result-error/result.html)

  

