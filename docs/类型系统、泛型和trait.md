# 1 类型系统

## 1.1 什么是类型系统

类型系统是对类型进行定义、检查和处理的系统。



强类型和弱类型：按定义后类型之间是否可以隐式转换划分

* 强类型语言：指不可以隐式转换（如： Rust、Java）

* 弱类型语言：指可以隐式转换（如：JavaScript ）




静态类型系统和动态类型系统：按类型的检查时机划分

* 静态类型系统：指在编译期进行类型检查，可进一步细分为显式静态和隐式静态（如：Rust/Java是显式静态语言，Haskell 是隐式静态语言）

* 动态类型系统：指在运行期间进行类型检查（如：JavaScript）




## 1.2 类型安全

Rust是一门类型安全的语言，因为

1、Rust是`强类型`加`静态类型系统`的一门语言；在定义时， Rust 不允许类型的隐式转换，即Rust 是强类型语言；同时在检查时，Rust 使用了静态类型系统，在编译期保证类型的正确



2、从内存的角度看，类型安全是指代码只能按照被允许的方法和被允许的权限(读/写)，访问它被授权访问的内存



以一个长度为 4，存放 u64 数据的数组为例。我们访问这个数组时，只能在这个数组的`起始地址`到`结束地址`之间这 32 个字节的内存中访问，而且访问是按照 8 字节来对齐的，且数组中的每个元素只能做 u64 类型允许的操作。对此，编译器会对代码进行严格检查来保证这个行为。

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230214235030.png)

3、Rust中除了 `let / fn / static / const` 这些定义性语句外，都是表达式，而一切表达式都有类型。

例如：以下伪代码的类型是unit，也就是()类型，表示没有值

```rust
if has_work {
    do_something();
}
```

**Rust中对于一个作用域，无论是if/else/for循环还是函数，最后一个表达式的返回值就是作用域的返回值。如果表达式或者函数不返回任何值，那么它返回一个 unit() 。unit 是只有一个值的类型，它的值和类型都是 () 。**



unit的应用场景除了作为返回值，还可以在数据结构中使用，例如

> * `Result<(), Error>` 表示返回的错误类型中，只关心错误，不关心成功的值
> * `HashMap<K, ()>` ，HashSet是`HashMap<K, ()>` 的一个类型别名



## 1.3 数据类型

作为静态类型语言，Rust 提供了大量的数据类型。



Rust的原生类型：字符、整数、浮点数、布尔值、数组（array）、元组（tuple）、切片（slice）、指针、引用、函数等

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230214235158.png)

在原生类型的基础上，Rust 标准库还支持非常丰富的组合类型，如下面这些：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230214235138.png)

另外在 Rust 已有数据类型的基础上，也可以使用结构体（struct）和标签联合（enum）定义自己的组合类型

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230214235106.png)

## 1.4 类型推导

Rust 支持局部的类型推导，在一个作用域之内，Rust编译器可以根据变量的上下文，推导出变量的类型，这样就不需要显式地进行类型标注。



注意：常量和静态变量的定义，即使上下文中含有类型的信息，也需要为变量提供类型。

> 因为 const/static 主要用于定义全局变量，它们可以在不同的上下文中使用，所以为了代码的可读性，需要明确的类型声明。

```rust
const PI: f64 = 3.1415926;
static E: f32 = 2.71828;

fn main() {
    const V: u32 = 10;
    static V1: &str = "hello";
    println!("PI: {}, E: {}, V {}, V1: {}", PI, E, V, V1);
}
```



**正确推导出类型的例子**

```rust
use std::collections::BTreeMap;

fn main() {
    let mut map = BTreeMap::new();
    map.insert("hello", "world");
    println!("map: {:?}", map);
}
```

Rust可推导出 BTreeMap 的类型 K 和 V 都是字符串引用 &str；如果把 map.insert 语句注释去掉，Rust 编译器就会报错：“cannot infer type for type parameter K”。



**无法推导出类型的例子**

```rust
// 把列表中的偶数过滤出来，生成一个新的列表
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let even_numbers = numbers
        .into_iter()
        .filter(|n| n % 2 == 0)
        .collect(); // collect()是 Iterator trait 的方法，作用是把一个 iterator 转换成一个集合

    println!("{:?}", even_numbers);
}
```

因为很多集合类型(如 `Vec<T>、HashMap<K, V>`)都实现了Iterator，所以编译器无法从上下文中推断出collect() 要返回什么类型。



* 改动方式1：声明even_numbers为类型`Vec<_>`

> 注意：编译器只是无法推断出集合类型，但集合类型内部元素的类型，还是可以根据上下文得出，所以简写成` Vec<_> `

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let even_numbers: Vec<_> = numbers
        .into_iter()
        .filter(|n| n % 2 == 0)
        .collect();

    println!("{:?}", even_numbers);
}
```

* 改动方式2(turbofish写法)：让 collect 返回一个明确的类型；在泛型函数后使用 `::` 来强制使用类型 T，这种写法被称为 turbofish，代码会更简洁。

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let even_numbers = numbers
        .into_iter()
        .filter(|n| n % 2 == 0)
        .collect::<Vec<_>>();

    println!("{:?}", even_numbers);
}
```



# 2 多态

在类型系统中，多态是指在使用相同的接口时，不同类型的对象会采用不同的实现

* 动态类型系统：多态通过鸭子类型实现
* 静态类型系统：多态可以通过参数多态、特设多态和子类型多态实现



**静态类型系统多态的3种形式：**

1. 参数多态：指实现的操作与具体的类型无关，类型是一个`满足某些约束`的参数，如泛型；Rust中，通过泛型来实现参数多态

2. 特设多态：指同一操作不同类型有不同的行为，如重载；Rust中，通过trait实现特设多态

3. 子类型多态：指同一对象可能属于多种类型，在运行时子类型可以被当成父类型使用，如继承和重写；Rust中，通过trait object来实现子类型多态




# 3 泛型

泛型是一种多态，在Rust中，用泛型实现参数多态



## 3.1 数据结构的泛型

数据结构的泛型：是指把数据结构中重复的参数抽出来；在使用泛型类型时，根据不同的参数，会得到不同的具体类型。



### 3.1.1 泛型的约束

泛型的约束：即对泛型参数做一定的规则限制。



Rust中两种表述`泛型参数的约束`的方式(两种方式可以同时使用)

* 类似函数参数的类型声明，用冒号":" 来表示约束，多个约束之间用加号"+" 表示，如`T: Trait_Name`表示T要满足名字为Trait_Name的这个trait
* 使用where语句，在返回值后面、大括号前面使用，如`where T: Trait_Name`



### 3.1.2 结构体中使用泛型

如下结构体：

```rust
pub struct Vec<T, A: Allocator = Global> {
    buf: RawVec<T, A>,
    len: usize,
}

pub struct RawVec<T, A: Allocator = Global> {
    ptr: Unique<T>,
    cap: usize,
    alloc: A,
}
```

T 和 A 是 Vec 的两个泛型参数，在使用泛型参数之前必需要进行声明 `Vec<T, A>`，接着就可以在结构体的字段类型中使用  T 和 A 来替代具体的类型。

* T：是Vec列表里的每个数据的类型

* A：它有一个约束“ A: Allocator” ，即 A 需要满足Allocator trait

  > A 参数有默认值 Global，它是 Rust 默认的全局分配器，所以使用时只需要用 T 即可



### 3.1.3 结构体的方法使用泛型

如下结构体：

```rust
struct Point<T> {
    x: T,
    y: T,
}

// 结构体的方法使用泛型
impl<T> Point<T> {
    fn getx(&self) -> &T {
        &self.x
    }
}

fn main() {
    let p: Point<i32> = Point { x: 1, y: 2 };
    println!("p.x = {} p.y = {}", p.getx(), p.y);
}
```

* `impl<T> Point<T> `中：`impl<T>`是泛型参数的声明，只有提前声明了，才可以在`Point<T>`中使用；此时的`Point<T>` 不再是泛型声明，而是一个完整的结构体类型，因为我们定义的结构体就是 `Point<T>` 而不再是 `Point`



### 3.1.4 枚举中使用泛型

以Cow（Clone-on-Write）枚举为例，Cow是 Rust 中一个很重要的数据结构；在返回数据时，可以用Borrowed返回一个借用的数据（只读），也可以用Owned返回一个拥有所有权的数据（可写）

```rust
pub enum Cow<'a, B: ?Sized + 'a> where B: ToOwned,
{
    // 借用的数据
    Borrowed(&'a B),
    // 拥有的数据
    Owned(<B as ToOwned>::Owned),
}
```

泛型参数B有3个约束 

1. 生命周期约束 'a：B 的生命周期是 'a，用 B: 'a 来表示，当 Cow 内部的类型 B 生命周期为 'a 时，Cow 自己的生命周期也是 'a

2. 长度可变 `?Sized ` 约束：? 代表可以放松问号之后的约束，Rust 默认的泛型参数都需要是 Sized，即固定大小的类型，所以 ?Sized 代表用可变大小的类型

3. “where B: ToOwned” 约束：表示符合 ToOwned trait，ToOwned是一个 trait，它可以把借用的数据克隆出一个拥有所有权的数据



**Cow里Owned方法中`<B as ToQwned>::Owned `的含义**：它对 B 做了一个强制类型转换，转成 ToOwned trait，然后访问 ToOwned trait 内部的 Owned 类型。

> 在 Rust 里，子类型可以强制转换成父类型，B 符合 ToOwned 约束，所以 B 是 ToOwned trait 的子类型，因而 B 可以安全地强制转换成 ToOwned



注意：在 Rust 里，**生命周期标注也是泛型的一部分**，一个生命周期 'a 代表任意的生命周期，和 T 代表任意类型是一样的。



上面Cow例子中，泛型参数的约束都发生在开头 struct的定义中，很多时候也可以在不同的实现方法时逐步添加约束，如下

```rust
use std::fs::File;
use std::io::{BufReader, Read, Result};

// 定义一个带有泛型参数 R 的 reader，此处我们不限制 R
struct MyReader<R> {
    reader: R,
    buf: String,
}

// 实现MyReader的new函数时，不需要限制 R
impl<R> MyReader<R> {
    // 函数的泛型
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::with_capacity(1024),
        }
    }
}

// 定义MyReader的process函数时，需要用到 R 的方法，此时限制 R 必须实现 Read trait
impl<R> MyReader<R>
where
    R: Read,
{
    pub fn process(&mut self) -> Result<usize> {
        self.reader.read_to_string(&mut self.buf)
    }
}

fn main() {
    let f: File = File::open("/etc/hosts").unwrap();
    let mut reader = MyReader::new(BufReader::new(f));

    let size = reader.process().unwrap();
    println!("total size read: {}", size);
}
```



## 3.2 函数的泛型

函数的泛型：在声明一个函数时，参数或返回值的类型可以由泛型参数声明。



例1：id() 是一个泛型函数，它接收一个带有泛型类型的参数，返回一个泛型类型

```rust
fn id<T>(x: T) -> T {
    return x;
}

fn main() {
    let int = id(10);
    let string = id("zhangsan");
    println!("{}, {}", int, string);
}
```

`id<T>`声明了泛型参数，x 参数的类型为 T



### 3.2.1 单态化处理(静态分派)

对于泛型函数，Rust 会进行单态化处理，单态化就是在编译时，把所有用到的泛型函数的泛型参数展开，生成若干个函数。



例如，上面例1中 id() 编译后会得到一个处理后的多个版本：

```rust
fn id_i32(x: i32) -> i32 {
    return x;
}
fn id_str(x: &str) -> &str {
    return x;
}
fn main() {
    let int = id_i32(22);
    let string = id_str("zhangsan");
    println!("{}, {}", int, string);
}
```

**单态化的优点：**

* 泛型函数的调用是`静态分派`，在编译时就一一对应，既保有多态的灵活性，又没有任何效率的损失，和普通函数调用一样高效

**单态化的缺点：**

* 编译速度很慢，一个泛型函数编译器需要找到所有用到的不同类型，然后一个个编译
* 同时，这样编出来的二进制会比较大，因为泛型函数的二进制代码实际存在 N 份
* 因为单态化，代码以二进制分发(指Rust编译成二进制库)会损失泛型的信息。如果写了一个库，提供了如上的 id() 函数，使用这个库的开发者如果拿到的是二进制，那么这个二进制中必须带有原始的泛型函数，才能正确调用。但单态化之后，原本的泛型信息就被丢弃了



## 3.2.2 







# 4 trait

在Rust中，用trait实现特设多态。特设多态是指同一操作不同类型有不同的行为。其实通过定义 trait 以及为不同的类型实现这个 trait，就已经实现了特设多态。



## 4.1 什么是trait

trait是Rust中的接口(相当于Java的interface)，它定义了类型使用这个接口的行为。



以标准库中 `std::io::Write` 为例

```rust
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize> { ... }
    fn is_write_vectored(&self) -> bool { ... }
    fn write_all(&mut self, buf: &[u8]) -> Result<()> { ... }
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice<'_>]) -> Result<()> { ... }
    fn write_fmt(&mut self, fmt: Arguments<'_>) -> Result<()> { ... }
    fn by_ref(&mut self) -> &mut Self where Self: Sized { ... }
}
```

这个 trait 中定义了一系列方法。这些方法被称作`关联函数`，关联函数可以有缺省的实现。除了write 和 flush方法，其他都有缺省实现。**在实现trait时，有默认缺省实现的方法可以按需选择是否实现，没缺省实现的方法一定要实现。**



### 4.1.1 Self和self

在Write trait定义方法中，有两个特殊的关键字：Self 和 self。

* Self 代表当前的类型，比如 File 类型实现了 Write，那么实现过程中使用到的 Self 就指代 File
* self 在用作方法的第一个参数时，实际上是 `self: Self` 的简写，所以 &self 是 `self: &Self`， 而 &mut self 是 `self: &mut Self`



## 4.2 trait的作用

1. 可以把`数据结构`中的行为单独抽取出来，使其可以在多个类型之间共享
2. 也可以作为约束，在泛型编程中，限制`参数化类型`必须符合它规定的行为



**例子**

构建一个 BufBuilder 结构实现 Write trait，并实现了Write trait 的 write 和 flush 方法（如果没有实现 write 或者 flush，Rust 编译器会报错）

```rust
use std::fmt;
use std::io::Write;

struct BufBuilder {
    buf: Vec<u8>,
}

// 实现 BufBuilder的 new 函数
impl BufBuilder {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(1024),
        }
    }
}

// 实现 Debug trait，打印字符串
impl fmt::Debug for BufBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.buf))
    }
}

// 实现 Write trait
impl Write for BufBuilder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // 把 buf 添加到 BufBuilder 的尾部
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // 由于是在内存中操作，所以不需要 flush
        Ok(())
    }
}

fn main() {
    let mut buf = BufBuilder::new();
  
    // BufBuilder可以直接使用Write trait的方法
    buf.write_all(b"Hello world!").unwrap();
  
    println!("{:?}", buf);
}
```

**数据结构一旦实现了某个 trait，那么这个 trait 内部的方法都可以被该数据结构使用，比如main函数里buf调用了write_all() 。**



## 4.3 编写trait

**需求：**

写一个字符串解析器，可以把字符串的某部分解析成某个类型。



**分析：**

1. 可以定义如下trait：它有一个方法parse，这个方法接收一个字符串引用，返回 Self

```rust
pub trait Parse {
  fn parse(s: &str) -> Self;
}
```

这个 parse 方法是 trait 的静态方法，因为它的第一个参数和 self 无关，所以在调用时需要使用 `T::parse(str) `。

2. 接着，来为 u8 这个数据结构来实现 parse，比如：“123abc” 会被解析出整数 123，而 “abcd” 会被解析出 0

   > 需要引入一个Regex库使用正则表达式提取需要的内容，还需要使用 str::parse 函数 把一个包含数字的字符串转换成数字



### 4.3.1 方式1: 一般做法

单独为u8实现Parse trait

Cargo.toml添加依赖

```toml
[dependencies]
regex = "0.2"
```

代码

```rust
use regex::Regex;

pub trait Parse {
    fn parse(s: &str) -> Self;
}

// 为 u8 这个数据结构实现parse
impl Parse for u8 {
    fn parse(s: &str) -> Self {
        let re: Regex = Regex::new(r"^[0-9]+").unwrap();
      
        if let Some(captures) = re.captures(s) {
            // 取第一个 match，将其捕获的 数字 换成 u8
            captures
                .get(0)
                .map_or(0, |s| s.as_str().parse().unwrap_or(0))
        } else {
            // 返回 0 的目的是为处理不了的情况，返回一个缺省值
            0
        }
    }
}

#[test]
fn parse_should_work() {
    assert_eq!(u8::parse("123abcd"), 123);
    assert_eq!(u8::parse("1234abcd"), 0);
    assert_eq!(u8::parse("abcd"), 0);
}

fn main() {
    println!("result: {}", u8::parse("255 hello world"));
}
```



### 4.3.2 方式2: 泛型参数实现trait

在实现Parse trait 时，用泛型参数来实现 trait，要注意对泛型参数做一定的限制。

1. 不是任何类型都可以通过字符串解析出来。只能处理数字类型，并且这个类型还要能够被 str::parse 处理

   > str::parse 是一个泛型函数，它返回任何实现了 FromStr trait 的类型，所以这里对泛型参数的第一个限制是，它必须实现了 FromStr trait

2. 当无法正确解析字符串时，要返回一个缺省值表示无法处理。上面代码会返回 0，但在使用泛型参数后，无法返回 0，因为 0 不一定是某个符合泛型参数的类型中的一个值

   > 在 Rust 标准库中有 Default trait，绝大多数类型都实现了这个 trait，来为数据结构提供缺省值，所以泛型参数的另一个限制是 Default。



改进后代码如下：

```rust
use std::str::FromStr;
use regex::Regex;

pub trait Parse {
    fn parse(s: &str) -> Self;
}

// 约束 T 必须同时实现了 FromStr 和 Default
// 这样在使用的时候我们就可以用这两个 trait 的方法了
impl<T> Parse for T
where
    T: FromStr + Default,
{
    fn parse(s: &str) -> Self {
        let re: Regex = Regex::new(r"^[0-9]+(\.[0-9]+)?").unwrap();
        // 生成一个创建缺省值的闭包，这里主要是为了简化后续代码
        // Default::default() 返回的类型根据上下文能推导出来，是 Self
        // 而我们约定了 Self，也就是 T 需要实现 Default trait
        let d = || Default::default();
      
        if let Some(captures) = re.captures(s) {
            captures
                .get(0)
                .map_or(d(), |s| s.as_str().parse().unwrap_or(d()))
        } else {
            d()
        }
    }
}

#[test]
fn parse_should_work() {
    assert_eq!(u32::parse("123abcd"), 123);
    assert_eq!(u32::parse("123.45abcd"), 0);
    assert_eq!(f64::parse("123.45abcd"), 123.45);
    assert_eq!(f64::parse("abcd"), 0f64);
}

fn main() {
    println!("result: {}", u8::parse("255 hello world"));
}
```

优点：通过对带有约束的泛型参数实现 trait，一份代码就实现了 u32 / f64 等类型的 Parse trait。

缺点：当无法正确解析字符串时，返回了缺省值，其实也有可能是出错了。

> 这里返回缺省值的话，会跟解析 “0abcd” 这样的情况混淆，不知道解析出的 0，究竟是出错了，还是本该解析出 0。



### 4.3.3 方式3: 带关联类型的trait

更好的方式是 parse 函数返回一个 Result：

```rust
pub trait Parse {
    fn parse(s: &str) -> Result<Self, E>;
}
```

这里 Result 的 E 要返回的错误信息，在 trait 定义时并不确定，不同的实现者可以使用不同的错误类型，可以使用`关联类型`把这种灵活性留给 trait 的实现者



**带关联类型的 trait**

Rust 允许 trait 内部包含关联类型，实现时跟关联函数一样，`它也需要实现关联类型`。trait 方法里的参数或者返回值，都可以用关联类型来表述，而在实现有关联类型的 trait 时，只需要额外提供关联类型的具体类型即可。

为Parse trait添加关联类型，示例如下：

```rust
pub trait Parse {
    type Error; // 关联类型
    fn parse(s: &str) -> Result<Self, Self::Error>; // 返回关联类型Error
}
```

有了关联类型 Error，Parse trait 就可以在出错时返回合理的错误了，看修改后的代码

```rust
use std::str::FromStr;
use regex::Regex;

pub trait Parse {
    type Error; // 关联类型
  
    fn parse(s: &str) -> Result<Self, Self::Error> // 返回关联类型Error
    where
        Self: Sized;
}

impl<T> Parse for T
where
    T: FromStr + Default,
{
    // 实现关联类型 Error 为 String
    type Error = String;
  
    fn parse(s: &str) -> Result<Self, Self::Error> {
        let re: Regex = Regex::new(r"^[0-9]+(\.[0-9]+)?").unwrap();
      
        if let Some(captures) = re.captures(s) {
            // 当出错时我们返回 Err(String)
            captures
                .get(0)
                .map_or(Err("failed to capture".to_string()), |s| {
                    s.as_str()
                        .parse()
                        .map_err(|_err| "failed to parse captured string".to_string())
                })
        } else {
            Err("failed to parse string".to_string())
        }
    }
}

#[test]
fn parse_should_work() {
    assert_eq!(u32::parse("123abcd"), Ok(123));
    assert_eq!(
        u32::parse("123.45abcd"),
        Err("failed to parse captured string".into())
    );
    assert_eq!(f64::parse("123.45abcd"), Ok(123.45));
    assert!(f64::parse("abcd").is_err());
}

fn main() {
    println!("result: {:?}", u8::parse("255 hello world")); // result: Ok(255)
}
```

优点：我们允许用户把错误类型延迟到 trait 实现时才决定，这种带有关联类型的 trait 比普通 trait，更加灵活，抽象度更高。



## 4.4 支持泛型的trait

trait的定义可以支持泛型，以标准库里 [std::ops::Add ](https://doc.rust-lang.org/std/ops/trait.Add.html)这个用于提供加法运算的 trait 为例：

```rust
pub trait Add<Rhs = Self> {
    type Output;
    #[must_use]
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

这个 trait 有一个泛型参数 Rhs，代表加号右边的值，它被用在 add 方法的第二个参数位。这里 Rhs 默认是 Self，也就是用 Add trait时，如果不提供泛型参数，那么加号右值和左值都要是相同的类型。



**例1：使用这个 trait 来定义一个复数类型**。复数类型有实部和虚部，两个复数的实部相加，虚部相加，得到一个新的复数

```rust
use std::ops::Add;

#[derive(Debug)]
struct Complex {
    real: f64,
    imagine: f64,
}

impl Complex {
    pub fn new(real: f64, imagine: f64) -> Self {
        Self { real, imagine }
    }
}

// Complex 实现 Add trait
impl Add for Complex {
    type Output = Self;

    // 注意 add 第一个参数是 self，会移动所有权
    fn add(self, rhs: Self) -> Self::Output {
        let real = self.real + rhs.real;
        let imagine = self.imagine + rhs.imagine;
        Self::new(real, imagine)
    }
}

fn main() {
    let c1 = Complex::new(1.0, 1f64);
    let c2 = Complex::new(2 as f64, 3.0);
  
    println!("{:?}", c1 + c2); // Complex { real: 3.0, imagine: 4.0 }
  
    // c1、c2 已经被移动，所以下面这句无法编译
    // println!("{:?}", c1 + c2);
}
```

注意 add 的第一个参数是 self，它会移动所有权，所以调用完两个复数 c1 + c2 后，根据所有权规则，它们就无法使用了。

缺点：Add trait 对于实现了 Copy trait 的类型如 u32、f64 等结构来说，用起来很方便，但对于我们定义的 Complex 类型，执行一次加法，原有的值就无法使用。这缺点可以对 Complex 的引用实现 Add trait 来解决



例2：为 &Complex 也实现 Add，可以做 &c1 + &c2，这样所有权就不会移动了。

```rust
use std::ops::Add;

#[derive(Debug)]
struct Complex {
    real: f64,
    imagine: f64,
}

impl Complex {
    pub fn new(real: f64, imagine: f64) -> Self {
        Self { real, imagine }
    }
}

// 对 Complex 类型的实现
impl Add for Complex {
    type Output = Self;

    // 注意 add 第一个参数是 self，会移动所有权
    fn add(self, rhs: Self) -> Self::Output {
        let real = self.real + rhs.real;
        let imagine = self.imagine + rhs.imagine;
        Self::new(real, imagine)
    }
}

// 如果不想移动所有权，可以为 &Complex 实现 add，这样可以做 &c1 + &c2
impl Add for &Complex {
    // 注意返回值不应该是 Self 了，因为此时 Self 是 &Complex
    type Output = Complex;

    fn add(self, rhs: Self) -> Self::Output {
        let real = self.real + rhs.real;
        let imagine = self.imagine + rhs.imagine;
      
        Complex::new(real, imagine)
    }
}

fn main() {
    let c1 = Complex::new(1.0, 1f64);
    let c2 = Complex::new(2 as f64, 3.0);
    println!("{:?}", &c1 + &c2);
    println!("{:?}", c1 + c2);
}
```

此例子也只是使用了Add trait缺省的泛型



例3：**设计一个复数和一个实数直接相加，相加的结果是实部和实数相加，虚部不变。**此时泛型参数会传入具体的类型，通过使用 Add，为 Complex 实现了和 f64 相加的方法。所以泛型 trait 可以让我们在需要时，对同一种类型的同一个 trait，有多个实现。

```rust
use std::ops::Add;

#[derive(Debug)]
struct Complex {
    real: f64,
    imagine: f64,
}

impl Complex {
    pub fn new(real: f64, imagine: f64) -> Self {
        Self { real, imagine }
    }
}

// 对 Complex 类型的实现
impl Add for Complex {
    type Output = Self;

    // 注意 add 第一个参数是 self，会移动所有权
    fn add(self, rhs: Self) -> Self::Output {
        let real = self.real + rhs.real;
        let imagine = self.imagine + rhs.imagine;
        Self::new(real, imagine)
    }
}

// 如果不想移动所有权，可以为 &Complex 实现 add，这样可以做 &c1 + &c2
impl Add for &Complex {
    // 注意返回值不应该是 Self 了，因为此时 Self 是 &Complex
    type Output = Complex;

    fn add(self, rhs: Self) -> Self::Output {
        let real = self.real + rhs.real;
        let imagine = self.imagine + rhs.imagine;
        Complex::new(real, imagine)
    }
}

// 因为 Add<Rhs = Self> 是个泛型 trait，可以为 Complex 实现 Add<f64>
impl Add<f64> for &Complex {
    type Output = Complex;

    // rhs 现在是 f64 了
    fn add(self, rhs: f64) -> Self::Output {
        let real = self.real + rhs;
        Complex::new(real, self.imagine)
    }
}

fn main() {
    let c1 = Complex::new(1.0, 1f64);
    let c2 = Complex::new(2 as f64, 3.0);
    println!("{:?}", &c1 + &c2);
    println!("{:?}", &c1 + 5.0);
    println!("{:?}", c1 + c2);
}
```

Add trait 就是一个典型的特设多态，同样是加法操作，根据操作数据的不同进行不同的处理。



## 4.5 trait的继承

在 Rust 中，一个 trait 可以“继承”另一个 trait 的`关联类型和关联函数`。



**比如 `trait B: A` ，是指任何类型T，如果实现了trait B，它也必须实现trait A，即trait B 在定义时可以使用 trait A 中的关联类型和方法。**

> 比如 tokio 库中的 [AsyncWriteExt](https://docs.rs/tokio/1.10.0/tokio/io/trait.AsyncWriteExt.html)、futures 库中的 [StreamExt](https://docs.rs/futures/0.3.16/futures/stream/trait.StreamExt.html)。



**以 StreamExt 为例**

由于 StreamExt 中的方法都有缺省的实现，且所有实现了 Stream 的类型都实现了 StreamExt：

```rust
impl<T: ?Sized> StreamExt for T where T: Stream {}
```

所以如果你实现了 Stream，就可以直接使用 StreamExt 里的方法了。



trait 作为对不同数据结构中相同行为的一种抽象。除了基本 trait 之外

* 当行为和具体的数据关联时，比如字符串解析时定义的 Parse trait，我们引入了带有关联类型的 trait，把和行为有关的数据类型的定义，进一步延迟到 trait 实现的时候。
* 对于同一个类型的同一个 trait 行为，可以有不同的实现，比如我们之前大量使用的 From，此时可以用泛型 trait。



# 5 trait object

在Rust中，用trait object实现子类型多态

## 5.1 子类型多态

从严格意义上说，子类型多态是面向对象语言的专利。**如果一个对象 A 是对象 B 的子类，那么 A 的实例可以出现在任何期望 B 的实例的上下文中**，比如猫和狗都是动物，如果一个函数的接口要求传入一个动物，那么传入猫和狗都是允许的。



Rust 虽然没有父类和子类，但 trait 和实现 trait 的类型之间也是类似的关系，所以，Rust 也可以做子类型多态

```rust
struct Cat;
struct Dog;

trait Animal {
    fn name(&self) -> &'static str;
}

// Cat实现Animal
impl Animal for Cat {
    fn name(&self) -> &'static str {
        "Cat"
    }
}

// Dog实现Animal
impl Animal for Dog {
    fn name(&self) -> &'static str {
        "Dog"
    }
}

// impl Animal 是 T: Animal 的简写
fn name(animal: impl Animal) -> &'static str {
    // 会自动调用子类的name方法
    animal.name()
}

fn main() {
    let cat = Cat;
    println!("cat: {}", name(cat));
}
```

这里 impl Animal 是 T: Animal 的简写，所以 name 函数的定义和以下定义等价：

```rust
fn name<T: Animal>(animal: T) -> &'static str;
```

这种泛型函数会根据具体使用的类型被单态化，编译成多个实例，是静态分派。



**静态分派**效率很高，但有时，类型可能很难在编译时决定。

例子：比如要写一个格式化工具，可以定义一个 Formatter 接口，然后创建一系列实现：

```rust
pub trait Formatter {
    fn format(&self, input: &mut String) -> bool;
}

struct MarkdownFormatter;

impl Formatter for MarkdownFormatter {
    fn format(&self, input: &mut String) -> bool {
        input.push_str("\nformatted with Markdown formatter");
        true
    }
}

struct RustFormatter;

impl Formatter for RustFormatter {
    fn format(&self, input: &mut String) -> bool {
        input.push_str("\nformatted with Rust formatter");
        true
    }
}

struct HtmlFormatter;

impl Formatter for HtmlFormatter {
    fn format(&self, input: &mut String) -> bool {
        input.push_str("\nformatted with HTML formatter");
        true
    }
}
```

1. 首先，使用什么格式化方法，只有当打开文件，分析出文件内容之后才能确定，我们无法在编译期给定一个具体类型
2. 其次，一个文件可能有一到多个格式化工具，比如一个 Markdown 文件里有 Rust 代码，同时需要 MarkdownFormatter 和 RustFormatter 来格式化。



这里如果使用一个 Vec 来提供所有需要的格式化工具，那么，下面这个函数其 formatters 参数该如何确定类型

```rust
pub fn format(input: &mut String, formatters: Vec<???>) {
    for formatter in formatters {
        formatter.format(input);
    }
}
```

正常情况下，Vec<> 容器里的类型需要是一致的，但此处无法给定一个一致的类型，此时就需要用到动态分派。



## 5.2 动态分派/Trait Object

我们要有一种手段告诉编译器，此处需要并且仅需要任何实现了 Formatter 接口的数据类型。在 Rust 里，这种类型叫 Trait Object，表现为 `&dyn Trait_Name 或者 Box`。



于是，上述代码可以写成：

```rust
pub fn format(input: &mut String, formatters: Vec<&dyn Formatter>) {
    for formatter in formatters {
        formatter.format(input);
    }
}
```

这样可以在运行时，构造一个 Formatter 的列表，传递给 format 函数进行文件的格式化，这就是动态分派（dynamic dispatching）。



最终代码如下：

```rust
pub trait Formatter {
    fn format(&self, input: &mut String) -> bool;
}

struct MarkdownFormatter;
impl Formatter for MarkdownFormatter {
    fn format(&self, input: &mut String) -> bool {
        input.push_str("\nformatted with Markdown formatter");
        true
    }
}

struct RustFormatter;
impl Formatter for RustFormatter {
    fn format(&self, input: &mut String) -> bool {
        input.push_str("\nformatted with Rust formatter");
        true
    }
}

struct HtmlFormatter;
impl Formatter for HtmlFormatter {
    fn format(&self, input: &mut String) -> bool {
        input.push_str("\nformatted with HTML formatter");
        true
    }
}

pub fn format(input: &mut String, formatters: Vec<&dyn Formatter>) {
    for formatter in formatters {
        formatter.format(input);
    }
}

fn main() {
    let mut text = "Hello world!".to_string();
  
    let html: &dyn Formatter = &HtmlFormatter;
    let rust: &dyn Formatter = &RustFormatter;
  
    let formatters = vec![html, rust];
  
    format(&mut text, formatters);

    println!("text: {}", text);
}
```



## 5.3 Trait Object 的实现机理

当需要使用 Formatter trait 做动态分派时，可以像下面例子一样，将一个具体类型的引用，赋给 &Formatter ：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230214234134.png)





**HtmlFormatter 的引用赋值给 Formatter 后，会生成一个 Trait Object，在上图中可以看到，Trait Object 的底层逻辑就是`胖指针`。其中，一个指针指向数据本身，另一个则指向虚函数表（vtable）。**



vtable 是一张静态的表，Rust 在编译时会为使用了 trait object 的类型的 trait 实现生成一张表，放在可执行文件中（一般在 TEXT 或 RODATA 段）。如下图

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230214234246.png)



在这张表里，包含具体类型的一些信息，如 size、aligment 以及一系列函数指针：

* 这个接口支持的所有的方法，比如 format() ；
* 具体类型的 drop trait，当 Trait object 被释放，它用来释放其使用的所有资源。

这样，当在运行时执行 formatter.format() 时，formatter 就可以从 vtable 里找到对应的函数指针，执行具体的操作。



**所以，Rust 里的 Trait Object 只不过是我们熟知的 C++ / Java 中 vtable 的一个变体而已。**



事实上，Rust 也并不区分原生类型和组合类型，对 Rust 来说，所有类型的地位都是一致的。



**不过，你使用 trait object 时，要注意对象安全（object safety）。只有满足对象安全的 trait 才能使用 trait object。**



**怎么区分不是对象安全的trait**

如果 trait 所有的方法，返回值是 Self 或者携带泛型参数，那么这个 trait 就不能产生 trait object。

1. 不允许返回 Self，是因为 trait object 在产生时，原来的类型会被抹去，所以 Self 究竟是谁不知道。比如 Clone trait 只有一个方法 clone()，返回 Self，所以它就不能产生 trait object。
2. 不允许携带泛型参数，是因为 Rust 里带泛型的类型在编译时会做单态化，而 trait object 是运行时的产物，两者不能兼容。

比如 From trait，因为整个 trait 带了泛型，每个方法也自然包含泛型，就不能产生 trait object。如果一个 trait 只有部分方法返回 Self 或者使用了泛型参数，那么这部分方法在 trait object 中不能调用。



## 5.4 Trait Object的使用场景

* trait object 的好处：当在某个上下文中需要满足某个 trait 的类型，且这样的类型可能有很多，当前上下文无法确定会得到哪一个类型时，我们可以用 trait object 来统一处理行为。和泛型参数一样，trait object 也是一种延迟绑定，它让决策可以延迟到运行时，从而得到最大的灵活性。
* trait Object的坏处：trait object 把决策延迟到运行时，带来的后果是执行效率的打折。在 Rust 里，函数或者方法的执行就是一次跳转指令，而 trait object 方法的执行还多一步，它涉及额外的内存访问，才能得到要跳转的位置再进行跳转，执行的效率要低一些。如果要把 trait object 作为返回值返回，或者要在线程间传递 trait object，都免不了使用 `Box<dyn T>` 或者 `Arc<dyn T>`，会带来额外的堆分配的开销。



### 5.4.1 在函数的参数中使用

我们可以在函数的参数或者返回值中使用 trait object。



例如：构建一个 Executor trait，并对比`做静态分发的 impl Executor`、`做动态分发的 &dyn Executor `和 `Box<dyn Executor>` 这几种不同的参数的使用：

```rust
use std::{error::Error, process::Command};

// 起别名
pub type BoxedError = Box<dyn Error + Send + Sync>;

pub trait Executor {
    fn run(&self) -> Result<Option<i32>, BoxedError>;
}

pub struct Shell<'a, 'b> {
    cmd: &'a str,
    args: &'b [&'a str],
}

impl<'a, 'b> Shell<'a, 'b> {
    pub fn new(cmd: &'a str, args: &'b [&'a str]) -> Self {
        Self { cmd, args }
    }
}

impl<'a, 'b> Executor for Shell<'a, 'b> {
    fn run(&self) -> Result<Option<i32>, BoxedError> {
        let output = Command::new(self.cmd).args(self.args).output()?;
        Ok(output.status.code())
    }
}

// 使用泛型参数
pub fn execute_generics(cmd: &impl Executor) -> Result<Option<i32>, BoxedError> {
    cmd.run()
}

// 使用 trait object: &dyn T
pub fn execute_trait_object(cmd: &dyn Executor) -> Result<Option<i32>, BoxedError> {
    cmd.run()
}

// 使用 trait object: Box<dyn T>
pub fn execute_boxed_trait_object(cmd: Box<dyn Executor>) -> Result<Option<i32>, BoxedError> {
    cmd.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_shall_work() {
        let cmd = Shell::new("ls", &[]);
        let result = cmd.run().unwrap();
        assert_eq!(result, Some(0));
    }

    #[test]
    fn execute_shall_work() {
        let cmd = Shell::new("ls", &[]);

        let result = execute_generics(&cmd).unwrap();
        assert_eq!(result, Some(0));
        let result = execute_trait_object(&cmd).unwrap();
        assert_eq!(result, Some(0));
        let boxed = Box::new(cmd);
        let result = execute_boxed_trait_object(boxed).unwrap();
        assert_eq!(result, Some(0));
    }
}
```

* 这里为了简化代码，使用了 type 关键字创建了一个 BoxedError 类型，是 Box 的别名，它是 Error trait 的 trait object，除了要求类型实现了 Error trait 外，它还有额外的约束：类型必须满足 Send / Sync 这两个 trait
* impl Executor 使用的是泛型参数的简化版本
* &dyn Executor 和 `Box<dyn Executor>>` 是 trait object，前者在栈上，后者分配在堆上。值得注意的是，分配在堆上的 trait object 也可以作为返回值返回，比如示例中的 `Result<Option<i32>, BoxedError>` 里使用了 trait object



### 5.4.2 在函数返回值中使用

在返回值中使用 trait object，是 trait object 使用频率比较高的场景。



先来看一些实战中会遇到的例子：首先是 [async_trait](https://docs.rs/async-trait)，它是一种特殊的 trait，方法中包含 async fn。目前 [Rust 并不支持 trait 中使用 async fn](https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/)，一个变通的方法是使用 async_trait 宏。



如下定义的Fetch trait：

```rust
// Rust 的 async trait 还没有稳定，可以用 async_trait 宏
#[async_trait]
pub trait Fetch {
    type Error;
    async fn fetch(&self) -> Result<String, Self::Error>;
}
```

这里宏展开后，类似于：

```rust
pub trait Fetch {
    type Error;
    fn fetch<'a>(&'a self) -> 
        Result<Pin<Box<dyn Future<Output = String> + Send + 'a>>, Self::Error>;
}
```

它使用了 trait object 作为返回值。这样不管 fetch() 的实现，返回什么样的 Future 类型，都可以被 trait object 统一起来，调用者只需要按照正常 Future 的接口使用即可。



再看一个 [snow](https://github.com/mcginty/snow) 下的 [CryptoResolver](https://docs.rs/snow/0.8.0/snow/resolvers/trait.CryptoResolver.html) 的例子：

```rust
/// An object that resolves the providers of Noise crypto choices
pub trait CryptoResolver {
    // 随机数生成算法（Random）
    /// Provide an implementation of the Random trait or None if none available.
    fn resolve_rng(&self) -> Option<Box<dyn Random>>;

    // DH 算法（Dh）
    /// Provide an implementation of the Dh trait for the given DHChoice or None if unavailable.
    fn resolve_dh(&self, choice: &DHChoice) -> Option<Box<dyn Dh>>;

    // 哈希算法（Hash）
    /// Provide an implementation of the Hash trait for the given HashChoice or None if unavailable.
    fn resolve_hash(&self, choice: &HashChoice) -> Option<Box<dyn Hash>>;

    // 对称加密算法（Cipher）
    /// Provide an implementation of the Cipher trait for the given CipherChoice or None if unavailable.
    fn resolve_cipher(&self, choice: &CipherChoice) -> Option<Box<dyn Cipher>>;

    // 密钥封装算法（Kem）
    /// Provide an implementation of the Kem trait for the given KemChoice or None if unavailable
    #[cfg(feature = "hfs")]
    fn resolve_kem(&self, _choice: &KemChoice) -> Option<Box<dyn Kem>> {
        None
    }
}
```

这是一个处理 [Noise Protocol](https://zhuanlan.zhihu.com/p/96944134) 使用何种加密算法的一个 trait。这个 trait 的每个方法，都返回一个 trait object，每个 trait object 都提供加密算法中所需要的不同的能力。所有这些，都有一系列的具体的算法实现，通过 CryptoResolver trait，可以得到当前使用的某个具体算法的 trait object。在处理业务逻辑时，我们不用关心当前究竟使用了什么算法，就能根据这些 trait object 构筑相应的实现，比如下面的 generate_keypair：

```rust
pub fn generate_keypair(&self) -> Result<Keypair, Error> {
    // 拿到当前的随机数生成算法
    let mut rng = self.resolver.resolve_rng().ok_or(InitStage::GetRngImpl)?;
    // 拿到当前的 DH 算法
    let mut dh = self.resolver.resolve_dh(&self.params.dh).ok_or(InitStage::GetDhImpl)?;
    let mut private = vec![0u8; dh.priv_len()];
    let mut public = vec![0u8; dh.pub_len()];
    // 使用随机数生成器 和 DH 生成密钥对
    dh.generate(&mut *rng);

    private.copy_from_slice(dh.privkey());
    public.copy_from_slice(dh.pubkey());

    Ok(Keypair { private, public })
}
```



### 5.4.3 在数据结构中使用

继续以 snow 的代码为例，看 HandshakeState 这个用于处理 Noise Protocol 握手协议的数据结构，用到了哪些 trait object

```rust
pub struct HandshakeState {
    pub(crate) rng:              Box<dyn Random>,
    pub(crate) symmetricstate:   SymmetricState,
    pub(crate) cipherstates:     CipherStates,
    pub(crate) s:                Toggle<Box<dyn Dh>>,
    pub(crate) e:                Toggle<Box<dyn Dh>>,
    pub(crate) fixed_ephemeral:  bool,
    pub(crate) rs:               Toggle<[u8; MAXDHLEN]>,
    pub(crate) re:               Toggle<[u8; MAXDHLEN]>,
    pub(crate) initiator:        bool,
    pub(crate) params:           NoiseParams,
    pub(crate) psks:             [Option<[u8; PSKLEN]>; 10],
    #[cfg(feature = "hfs")]
    pub(crate) kem:              Option<Box<dyn Kem>>,
    #[cfg(feature = "hfs")]
    pub(crate) kem_re:           Option<[u8; MAXKEMPUBLEN]>,
    pub(crate) my_turn:          bool,
    pub(crate) message_patterns: MessagePatterns,
    pub(crate) pattern_position: usize,
}
```

你不需要了解 Noise protocol，也能够大概可以明白这里 Random、Dh 以及 Kem 三个 trait object 的作用：它们为握手期间使用的加密协议提供最大的灵活性。**如果这里不用 trait object，这个数据结构该怎么处理？**



可以用泛型参数，也就是说：

```rust
pub struct HandshakeState<R, D, K>
where
    R: Random,
    D: Dh,
    K: Kem
{
  ...
}
```

这是我们大部分时候处理这样的数据结构的选择。但是，过多的泛型参数会带来两个问题：

1. 首先，代码实现过程中，所有涉及的接口都变得非常臃肿，在使用 HandshakeState 的任何地方，都必须带着这几个泛型参数以及它们的约束
2. 其次，这些参数所有被使用到的情况，组合起来，会生成大量的代码

而使用 trait object，在牺牲一点性能的前提下，消除了这些泛型参数，实现的代码更干净清爽，且代码只会有一份实现。



**在数据结构中使用 trait object 还有一种很典型的场景是：闭包。**

因为在 Rust 中，闭包都是以匿名类型的方式出现，我们无法直接在数据结构中使用其类型，只能用泛型参数。而对闭包使用泛型参数后，如果捕获的数据太大，可能造成数据结构本身太大；但有时，我们并不在意一点点性能损失，更愿意让代码处理起来更方便。



例1：比如用于做 RBAC 的库 [oso](https://github.com/osohq/oso) 里的 AttributeGetter，它包含了一个 Fn

```rust
#[derive(Clone)]
pub struct AttributeGetter(
    Arc<dyn Fn(&Instance, &mut Host) -> crate::Result<PolarValue> + Send + Sync>,
);
```

例2：再比如做交互式 CLI 的 [dialoguer](https://github.com/mitsuhiko/dialoguer) 的 [Input](https://docs.rs/dialoguer/0.8.0/dialoguer/struct.Input.html)，它的 validator 就是一个 FnMut

```rust
pub struct Input<'a, T> {
    prompt: String,
    default: Option<T>,
    show_default: bool,
    initial_text: Option<String>,
    theme: &'a dyn Theme,
    permit_empty: bool,
    validator: Option<Box<dyn FnMut(&T) -> Option<String> + 'a>>,
    #[cfg(feature = "history")]
    history: Option<&'a mut dyn History<T>>,
}
```



# 参考

* [陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/420028)

* [泛型和特征](https://course.rs/basic/trait/intro.html)

* [Trait 和 Trait Object](https://rust-book.junmajinlong.com/ch11/00.html)

