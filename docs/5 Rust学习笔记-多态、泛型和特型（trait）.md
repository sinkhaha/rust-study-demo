

# 1 多态

## 1.1 类型系统

类型系统是对类型进行定义、检查和处理的系统。



强类型和弱类型：按定义后类型之间是否可以隐式转换划分

* 强类型语言：指不可以隐式转换（如： `Rust`、`Java`）

* 弱类型语言：指可以隐式转换（如：`JavaScript` ）


静态类型系统和动态类型系统：按类型的检查时机划分

* 静态类型系统：指在编译期进行类型检查，可进一步细分为显式静态和隐式静态（如：`Rust / Java` 是显式静态语言，`Haskell` 是隐式静态语言）
* 动态类型系统：指在运行期间进行类型检查（如：`JavaScript`）



## 1.2 多态

在类型系统中，多态是指在使用相同的接口时，不同类型的对象会采用不同的实现

* 动态类型系统：多态通过鸭子类型实现
* 静态类型系统：多态可以通过参数多态、特设多态和子类型多态实现



## 1.3 静态类型系统多态的 3 种形式

1. 参数多态：指实现的操作与具体的类型无关，类型是一个`满足某些约束`的参数，如泛型

   > `Rust` 中，通过泛型来实现参数多态

2. 特设多态：指同一操作不同类型有不同的行为，如重载

   > `Rust` 中，通过特型（`trait`）实现特设多态

3. 子类型多态：指同一对象可能属于多种类型，在运行时子类型可以被当成父类型使用，如继承和重写

   > `Rust` 中，通过特型对象 （`trait object`）来实现子类型多态




# 2 泛型

在 `Rust` 中，泛型是多态的一种表示形式，用泛型实现参数多态



## 2.1 数据结构的泛型

数据结构的泛型：是指把数据结构中重复的参数抽出来；在使用泛型类型时，根据不同的参数，会得到不同的具体类型。



### 2.1.1 在结构体中使用泛型

如下结构体：

```rust
// 在结构体中使用泛型
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

1. 结构体 `struct Point<T>{}` 使用了泛型参数 `T`，在使用泛型参数之前需要进行声明 `Point<T>`，接着就可以在结构体的字段类型中使用 `T` 替代具体的类型

2. 再看一下为结构体添加方法的 `impl<T> Point<T> ` ，其中`impl<T>` 是泛型参数的声明，只有提前声明了，才可以在 `Point<T>` 中使用；此时的 `Point<T>` 不再是泛型声明，而是一个完整的结构体类型



### 2.1.2 在枚举中使用泛型

以 `Cow（Clone-on-Write）` 枚举为例，`Cow` 是 `Rust` 中一个很重要的数据结构；在返回数据时，可以用 `Borrowed` 返回一个借用的数据（只读），也可以用 `Owned` 返回一个拥有所有权的数据（可写）

```rust
pub enum Cow<'a, B: ?Sized + 'a> where B: ToOwned,
{
    // 借用的数据
    Borrowed(&'a B),
    // 拥有的数据
    Owned(<B as ToOwned>::Owned),
}
```

这里泛型参数 `B` 有 3 个约束： 

1. `B: ?Sized + 'a` 约束可以拆成 `B: 'a` 和 `B: ?Sized ` 两个约束

   * `B: 'a`  约束表示 `B` 的生命周期是 `'a`，当 `Cow` 内部的类型 `B` 生命周期为 `'a` 时，Cow 自己的生命周期也是 `'a`
   * `B: ?Sized` 约束表示B 可以是任意大小的类型。`Rust` 默认的泛型参数都需要是固定大小（`Sized`） 的，加了个 `?` 问号，即 `?Sized` 代表用任意大小的类型

2. 子特型 `ToOwned` 约束：用 `where B: ToOwned` 表示，说明 `B` 是 `ToOwned` 特型的子特型，ToOwned` 可以把借用的数据克隆出一个拥有所有权的数据

   

注意：在 `Rust` 里，**生命周期标注也是泛型的一部分**，一个生命周期 `'a` 代表任意的生命周期，和 `T` 代表任意类型是一样的。



 `Cow` 里 `Owned` 方法中 `<B as ToQwned>::Owned` 的含义：它对 `B` 做了一个强制类型转换，转成 `ToOwned trait`，然后访问 `ToOwned trait` 内部的 `Owned` 类型。

> 在 `Rust` 里，子类型可以强制转换成父类型，`B` 符合 `ToOwned` 约束，所以 `B` 是 `ToOwned trait` 的子类型，因而 `B` 可以安全地强制转换成 `ToOwned`



**例子**

上面 `Cow` 枚举，泛型参数的约束都发生在开头的定义中，很多时候也可以在不同的实现方法时逐步添加约束，如下

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



### 2.1.3 泛型的约束

泛型的约束：指对泛型参数做一定的规则限制。



**`Rust` 中 2 种表示泛型参数的约束的方式（两种方式可以同时使用）**

* 类似函数参数的类型声明，用冒号 `:` 来表示约束，多个约束之间用加号 `+` 表示，如 `T: Trait_Name` 表示 `T` 要满足名字为 `Trait_Name` 的这个 `trait`，即 `T` 是 `Trait_Name` 的子特型
* 也可以使用 `where` 语句：在函数的返回值类型后面、大括号前面使用，语法为  `where T: Trait_Name`，即 `T` 是 `Trait_Name` 的子特型



例如看上面的 `Cow` 枚举的定义

```rust
pub enum Cow<'a, B: ?Sized + 'a> where B: ToOwned,
{
    // 借用的数据
    Borrowed(&'a B),
    // 拥有的数据
    Owned(<B as ToOwned>::Owned),
}
```

* `B: ?Sized + 'a` ：表示泛型参数 `B` 是任意大小的类型（`?sized`），且生命周期是 `'a`
* `where B: ToOwned` ：表示 `B` 是 `ToOwned trait` 的子特型，`ToOwned` 的作用是可以把借用的数据克隆出一个拥有所有权的数据



## 2.2 函数的泛型 和 静态分派

函数的泛型：是指在声明一个函数时，参数或返回值的类型可以由泛型参数声明。



### 2.2.1 单态化（静态分派）

对于泛型函数，`Rust` 会进行单态化处理（静态分派），单态化就是在编译时，把所有用到的泛型函数的泛型参数展开，生成若干个函数。



**例子：**

`id()` 是一个泛型函数，`id<T>` 声明了泛型参数，它接收一个带有泛型类型 `T` 的参数 `x`，并返回一个泛型类型，

```rust
fn id<T>(x: T) -> T {
    return x;
}

fn main() {
    let int = id(10); //  也可以这样调用 id::<i32>(20);
    let string = id("zhangsan");
    println!("{}, {}", int, string);
}
```

经过 `Rust` 的单态化处理，上面泛型函数在编译后，会得到一个处理后的多个版本：

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
* 静态分派编出来的二进制会比较大，因为泛型函数的二进制代码实际存在 `N` 份
* 因为单态化，代码以二进制分发（指 `Rust` 编译成二进制库）会损失泛型的信息。如果写了一个库，提供了如上的 `id()` 函数，使用这个库的开发者如果拿到的是二进制，那么这个二进制中必须带有原始的泛型函数，才能正确调用。但单态化之后，原本的泛型信息就被丢弃了



### 2.2.2 impl Trait

先看下面代码，由多个泛型类型组合而成的结果，看起来可能会很凌乱，如

```rust
use std::iter;
use std::vec:IntoIter;

fn cyclical_zip(v: Vec<u8>, u: Vec<u8>) -> 
    iter::Cycle<iter::Chain<IntoIter<u8>, IntoIter<u8>>> {
      v.into_iter().chain(u.into_iter()).cycle()
}
```



**优化方式一**：可以用特型对象替代这个“丑陋的” 返回类型，如

```rust
// 可以用特性对象替代这个“丑陋的” 返回类型
fn cyclical_zip(v: Vec<u8>, u: Vec<u8>) -> Box<dyn Iterator<Item=u8>> {
  Box::new(v.into_iter().chain(u.into_iter())).cycle()
} 
```

缺点：这种方式就要在每次调用这个函数时承受动态分发和不可避免的堆分配开销，实际它只是为了简写类型而已，没什么实际效用。



**优化方式二：**`impl Trait ` 特性

`Rust` 中有一个名为 `impl Trait` 的特性，该特性允许“擦除”返回值的类型，仅指定它实现的一个 或 多个特型，而无须进行动态派发 或 堆分配。可以改成以下写法 `impl Iterator<Item=u8> `，如

```rust
// 改成了这种写法，声明了它会返回某种 u8 迭代器
fn cyclical_zip(v: Vec<u8>, u: Vec<u8>) -> impl Iterator<Item=u8> {
  v.into_iter().chain(u.into_iter()).cycle()
}
```



注意：`impl Trait` 也是一种静态分派形式，因此编译器必须在编译期就知道从该函数返回的类型，以便在栈上分配正确的空间数量并正确访问该类型的字段和方法。

* 所以 `Rust` 不支持**特型方法**使用 `impl Trait` 作为返回值
* 只有**自由函数和关联具体类型**的函数才能使用 `impl Trait` 作为返回值

例如有如下 `trait`

```rust
trait Shape {
  fn new() -> Self;
  fn area(&self) -> f64;
}
```

假设在为几种类型实现了 `Shape` 后，此时希望根据某个运行期的值使用不同的 `Shape`，实际以 `impl Shape` 作为返回类型并不能实现这一目标，如

```rust
fn make_shape(shape: &str) -> impl Shape { // 特型方法返回 impl Shape
  match shape {
    "circle" => Circle::new(),
    "triangle" => Triangle::new(), // 错误：不兼容的类型
    "shape" => Rectangle::new(),
  }
}
```

因为在这个 `make_shape` 方法中， 这个类型可能是 `Circle`、`Triangle`、`Rectangle`，它们占用的空间大小可能不同，并有着不同的`area()`方法实现，所以不能使用 `impl Shape` 作为返回值。



`impl Trait` 也可以用在带有泛型参数的函数中，如

```rust
fn print<T: Display>(val: T) {
  println!("{}", val);
}
```

它与使用 `impl Trait` 的版本完全相同

```rust
fn print(val: impl Display) {
    println!("{}", val);
}
```

但是使用泛型时允许函数的调用者指定泛型参数的类型，比如` print::<i32>(42)`，而如果使用 `impl Trait` 则不能这么做。



### 2.2.3 泛型函数的类型参数

泛型函数可以有多个类型参数，如

```rust
fn run_queryM<M: Mapper + Serialize, R: Reducer + Serialize>(data: &DataSet, map: M, reduce: R) -> Results {
    ...
}
```

这种语法可以用 `where` 关键字来表示，类型参数 `M` 和 `R`  仍然放在前面声明里，但是限界移到了单独的行，如

```rust
fn run_queryM<M, R>(data: &DataSet, map: M, reduce: R) -> Results 
    where M: Mapper + Serialize, R: Reducer + Serialize 
{...}
```



泛型函数可以同时具有生命周期参数和类型参数，生命周期参数要排在前面，如

```rust
fn nearest<'t, 'c, P>(target: &'t P, candidates: &'c [P]) -> &'c P 
    where P: MeasureDistance
{ ... }
```



泛型函数也可以接收常量参数，如

```rust
// 需要一个泛型参数N，类型时unsize
fn dot_procuct<const N: unsize>(a: [f64; N], b: [f64; N]) -> f64 {
  let mut sum = 0;
  for i in 0..N {
    sum += a[i] * b[i];
  }
  sum
}
```



注意：

* 单独的方法也可以是泛型的，即使它并没有定义在泛型类型上

* 类型别名也是可以泛型的





# 3 特型（trait）

在 `Rust` 中，特型是一种多态，用特型实现特设多态。特设多态是指同一操作不同类型有不同的行为。其实通过定义 `trait` 以及为不同的类型实现这个 `trait`，就已经实现了特设多态。



## 3.1 什么是特型

特型（`trait` ）可以简单理解为 `Rust` 中的接口，它定义了类型使用这个接口的行为。



大多数情况，特型代表着一种能力，即一个类型能做什么，例如

* 实现了 `std::fmt::Debug` 的值能用带有 `{:?}` 格式说明符的 `println!()` 打印
* 实现了 `std::clone::Clone` 的值能在内存中克隆自身

注意：特型本身必须在作用域内，否则，它的所有方法都是不可见的，此时要使用 `use` 导入特型

> 有一些可以不导入，比如 `Clone` ，因为默认它们始终在作用域中，它们是标准库预到如的一部分，`Rust` 会把这些名称自动到如每个模块中



## 3.2 定义和实现特型

**定义特型**

使用 `“trait 特型名”` 即可定义一个特型，例如

```rust
trait Visible {
  /// 在给定的画布上渲染此对象
  fn draw(&self, canvas: &mut Canvas);
  
  /// 单击(x, y) 时是选中此对象，就返回true
  fn hit_test(&self, x: i32, y: i32) -> bool;
}
```



**实现特型**

使用 `“impl 特型名 for 某个类型”` 即可为某个类型实现特型，注意`impl` 块中的一切方法实现都必须是属于此特型

```rust
impl Visible for Broom {
   ... // 实现draw和hit_test方法，这些方法里可以直接调用辅助方法
}
```

如果要添加一个辅助方法来给 `Broom` 的 `draw()` 使用，必须在单独的 `impl` 块中定义它，不能写在特型的实现里，这些辅助方法，可以在上面特型的各个 `impl` 块中使用，例如添加一个名为 `broomstick_range` 的辅助方法

```rust
// 单独的imple块，没有实现 Visible
impl Broom {
  fn broomstick_range(&self) -> Range<i32> {
    self.y - self.height -1 .. self.y
  }
}
```



**特型的作用**

1. 可以把`数据结构中的行为`单独抽取出来定义成一个特型，使其可以在多个类型之间共享。`Rust` 允许在**任意类型**上实现特型，但特型 或 类型之间必须至少有一个是在当前 `crate` 中新建的，避免调错方法。所以想为任意类型添加一个方法，都可以用特型来完成
2. 特型也可以作为约束，在泛型编程中，限制`参数化类型`必须符合它规定的行为，如 `W: Write`，表示这个类型 `W` 是 `Write` 的子特型



例1：如为 `char` 类型添加一个 `is_emoji` 的方法

```rust
// 为能实现方法新建一个特型
trait IsEmoji {
  fn is_emoji(&self) -> bool;
}

// 为内置的字符类型实现IsEmoji特型
impl IsEmoji for char {
  fn is_emoji(&self) -> bool {
    ...
  }
}

assert_eq!('$'.is_emoji(), false); // 只有IsEmoji在当前作用域，这个is_emoji方法才可见
```

像上面这个特殊特型（`IsEmoji`）的唯一目的是向现有类型 `char` 中添加一个方法，这称为扩展类型。利用这个扩展类型，还可以使用一个泛型的`impl` 块来一次性向整个类型家族添加扩展类型，如定义一个能让你把 `HTML`写入值里的特型 `WriteHtml`

```rust
use std::io::{Self, Write};

/// 定义一个能让你把HTML写入值里的特型
trait WriteHtml {
  fn write_html(&mut self, html: &HtmlDocument) -> io::Result<()>;
}
```

为所有写入器实现特型，比如为所有 `Rust` 写入器添加一个方法

```rust
impl<W: Write> WriteHtml for W { // 意思是对于每个实现了Write的类型W，这里有一个适用于WriteHtml实现
  fn write_html(&mut self, html: &HtmlDocument) -> io::Result<()> {
    ...
  }
}
```

`serde` 序列化库就提供了一个很好的例子，它定义了一个特型 `Serialize`，为该库支持的每中数据类型都提供了实现，为一些类型添加了 `.serialize()` 方法，比如 `bool`、`i8`、`i6`、`i32`、`Vec`、`HashMap` 等类型。



### 3.2.1 关联函数

特型里定义一系列方法，这些方法都称为关联函数，关联函数可以有缺省的实现。当实现这个特型时，有缺省实现的方法我们可以按需选择是否实现，如果实现了调用时就会调用我们自己实现的方法，没实现就调用特型自己默认的实现。但是特型中没有缺省实现的方法一定要自己实现。



例如以标准库中 `std::io::Write` 特型为例，除了 `write` 和 `flush` 方法，其他都有缺省实现

```rust
// 标准库中 Write 的定义
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>; // 没有缺省实现
    fn flush(&mut self) -> Result<()>; // 没有缺省实现
    ...
    fn write_all(&mut self, buf: &[u8]) -> Result<()> { 
      // 有缺省实现
    }
    fn by_ref(&mut self) -> &mut Self
       where Self: Sized { ... }
    ...
}

pub struct Sink; // 空结构体

// 为 结构体 实现 Write 特型
impl Write for Sink {
  // 必须实现write
  fn write(&mut self, buf: &[u8]) -> Result<usize>{
    Ok(buf.len())
  }
  // 必须实现flush
  fn flush(&mut self) -> Result<()> {
    Ok(())
  }
}

let mut out = Sink;
out.write_all(b"hello world \n")?; // 这里可以直接调用 Write 默认的write_all
```

数据结构一旦实现了某个特型，那么这个特型内部的方法都可以被该数据结构使用，例如这里调用了 `Write` 的 `write_all`。



**类型关联函数（静态方法）**

大多数面向对象的语言，接口不能包含静态方法或构造函数。但是特型可以包含类型关联函数，这是 `Rust` 对静态方法的模拟，例如

```rust
trait StringSet {
  /// 返回一个新建的空集合
  fn new() -> Self;
  
  /// 返回一个包含 strings 中所有字符串的集合
  fn from_slice(strings: &[&str]) -> bool;
  
  /// 判断这个集合中是否包含特定的 string
  fn contains(&self, string: &str) -> bool;
}
```

每个实现了 `StringSet` 特型的类型都要实现上面 3 个关联函数。前面的 `new()` 和 `from_slice()` 不接受 `self` 参数，它们类似构造函数的角色，可以使用  `::`  双冒号语法调用它们，就像调用任何其他类型的关联函数一样，如用 `StringSet::new()` 进行调用 。



注意：特型对象不支持类型关联函数。如果想使用 `&dyn StringSet` 特型对象，就必须修改此特型，为每个`未通过引用接受 self 参数的关联函数`加上类型限界 `where Self: Sized ` ，例如

```rust
trait StringSet {
  /// 返回一个新建的空集合
  fn new() -> Self;
      where Self: Sized // 加上了限界
  
  /// 返回一个包含 strings 中所有字符串的集合
  fn from_slice(strings: &[&str]) -> bool;
      where Self: Sized // 加上了限界
  
  /// 判断这个集合中是否包含特定的 string
  fn contains(&self, string: &str) -> bool;
}
```

这个限界告诉 `Rust`，特型对象不需要支持特定的关联函数。通过添加这些限界，就能把 `StringSet` 作为特型对象使用了。虽然特型对象仍不支持关联函数 `new` 或 `from_slice`，但是还是可以创建它们并用调用其他方法的，如 `contains()` 。



### 3.2.2 关联常量

与结构体和枚举一样，特型也可以关联常量，它们语法相同，例如

```rust
trait Greet {
  const GREETING: & 'static str = "Hello";
  fn greet(&self) -> String;
}
```

不过关联常量在特型中有特殊的功能，可以声明它们，但不为其定义值，之后，特型的实现者可以定义这些值。例如

```rust
trait Float {
  const ZERO: Self; // 只声明，不赋值
  const ONE: Self；
}

// 实现时再赋值
impl Float for f32 {
  const ZERO: f32 = 0.0;
  const ONE: f32 = 1.0;
}

impl Float for f64 {
  const ZERO: f64 = 0.0;
  const ONE: f64 = 1.0;
}
```

注意：关联常量不能与特型对象一起使用，因为为了在编译期选择正确的值，编译器会依赖相关实现的类型信息。



### 3.2.3 关联类型

以迭代器为例，迭代器是用于遍历某种值序列的对象。`Rust` 中一个标准的 `Iterator` 特型，定义如下

```rust
pub trait Iterator {
  type Item; // 关联类型
  
  fn next(&mut self) -> Option<Self::Item>; // 返回值使用了关联类型
  ...
}
```

* `type Item` 是一个关联类型，表示实现了 `Iterator` 的每种类型都必须指定它所生成的条目的类型

* 如 `next()` 方法，在其返回值中使用了关联类型

  > 它会返回一个 `Option<Self::Item>` 或者 是序列中的下一个值 `Some(item)`，或者当没有更多值可供访问时返回 `None`。该类型不能写成无修饰的 `Item`，因为这里的 `Item` 是每个迭代器类型下的一个特性，而不是一个独立的类型

实现 `Iterator` 的方式如下：

```rust
impl Iterator for Args { // 标准库中 std::env 模块的代码
  type Item = String; // 关联类型是具体的 String 类型
  
  fn next(&mut self) -> Option<String> {
    ...
  }
  ...
}
```

具有关联类型的特型（如 `Iterator`）与特型对象是兼容的，但前提是要把所有关联类型都明确写出来。



### 3.2.4  Self 和 self

以 `Write` 特型为例子，它的定义方法中，有两个特殊的关键字：`Self` 和 `self`

* `Self` 代表当前的类型，比如 `MyFile` 类型实现了 `Write`，那么实现过程中使用到的 `Self` 就指代 `MyFile`，这样的好处是有多个实现时，每个实现都能识别 `Self` 就是自己的类型
* `self` 作为方法的第一个参数，实际上 `self` 是 `self: Self` 的简写，表示当前对象。同理， `&self` 是 `self: &Self`， 而 `&mut self` 是 `self: &mut Self`



例如：特型可以用关键字 `Self` 作为类型

```rust
// 定义一个特型
pub trait Spliceable {
  fn splice(&self, other: &Self) -> Self; // Self作为返回值
}
// 有两个实现
impl Soliceable for CherryTree {
    fn splice(&self, other: &Self) -> Self { // 这里返回值Self是CherryTree
        ...
    } 
}
impl Soliceable for Mammoth {
   fn splice(&self, other: &Self) -> Self { // 这里返回值Self是Mammoth
        ...
   }  
}
```

这里 `Self` 作为返回值类型 ，说明 `x.splice()` 的类型与 `x` 的类型相同。



注意：使用了 `Self` 类型的特型与特型对象不兼容。如

```rust
// 错误，特型 Spliceable 不能用作特型对象
fn splice_anything(left: &dyn Spliceable, right: &dyn Spliceable) {
   let combo = left.splice(right); // 注意这里left是特型对象，这里是错误的
}
```



## 3.3 子特型（继承）

在 `Rust` 中，一个 `trait` 可以“继承”另一个 `trait` 的关联类型 和 关联函数。比如 `trait B: A` ，是指任何类型 `T`，如果实现了特型  `B`，它也必须实现特型 `A`，即特型 `B` 在定义时可以使用特型 `A` 中的关联类型和方法。

> 比如 `tokio` 库中的 [AsyncWriteExt](https://docs.rs/tokio/1.10.0/tokio/io/trait.AsyncWriteExt.html)



例如，特型 `Creature` 是 `Visible` 的子特型

```rust
// Creature 类型实现了Visible特型
// 可以说成Creature是Visible的子类型，而Visible是Creature的超特型
trait Creature: Visible {
   fn position(&self) -> (i32, i32);
   fn facing(&self) -> Direction;
}
```

每个实现了 `Creature` 的类型也必须实现 `Visible` 特型；如果你实现了 `Creature`，就可以直接使用 `Visible` 中的方法

```rust
// Broom 实现了 Creature 特型，所以 Broom 也得实现 Visible 特型
impl Creature for Broom {
    ...
}

// Broom 实现 Visible 特型
impl Visible for Broom {
    ...  
}
```

注意：子特型不会继承其超特型的关联项，如果想调用超特型的方法，那么依然要保证每个特型都在作用域内。



实际 `Rust` 的子特型只是对 `Self` 类型界限的简写。上面子特型实现 `Visible` 也可以写成，结果是等价的。

```rust
trait Creature where Self: Visible {
   ...
}
```



## 3.4 泛型特型

特型的定义可以支持泛型，标准库中  [std::ops::Add ](https://doc.rust-lang.org/std/ops/trait.Add.html) 是用于提供加法运算的特型，它就使用了泛型特型，定义如下

```rust
// 用于标记支持 + 加号 的类型的特型
pub trait Add<Rhs = Self> {
    type Output; // 在应用了 + 运算符后的结果类型
  
    // 实现 + 运算符的方法
    #[must_use]
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

* `Add` 是一个泛型特型，泛型参数 `RHS` 是右操作数的缩写（`right-hand side`），它被应用到 `add` 方法的第二个参数位。这里 `Rhs` 默认是 `Self`，也就是用 `Add trait` 时，如果不提供泛型参数，那么乘号右值和左值都要是相同的类型。

  > 泛型参数的实例是 `Add<f64>`、`Add<String>`、`Add<Size>` 等都是不同的特型

* 单一类型可以同时实现 `Add<f64>`、`Add<i32>` 。每个实现都有自己关联的 `Output` 类型
* 在 `Rust` 中，表达式 `lhs + rhs` 是 `Add::add(lhs, rhs) ` 的简写形式



**例1**：使用这个 `trait` 来定义一个复数类型。复数类型有实部和虚部，两个复数的实部相加，虚部相加，得到一个新的复数

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

注意 `add` 的第一个参数是 `self`，它会移动所有权，所以调用完两个复数 `c1 + c2` 后，根据所有权规则，它们就无法使用了。

缺点：`Add trait` 对于实现了 `Copy trait` 的类型如 `u32`、`f64` 等结构来说，用起来很方便，但对于我们定义的 `Complex` 类型，执行一次加法，原有的值就无法使用。这缺点可以对 `Complex` 的引用实现 `Add trait` 来解决



**例2**：为 `&Complex` 也实现 `Add`，可以做 `&c1 + &c2`，这样所有权就不会移动了

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

此例子也只是使用了 `Add trait` 缺省的泛型



**例3：设计一个复数和一个实数直接相加，相加的结果是实部和实数相加，虚部不变。**此时泛型参数会传入具体的类型，通过使用 `Add`，为 `Complex` 实现了和 `f64` 相加的方法。所以泛型 `trait` 可以让我们在需要时，对同一种类型的同一个 `trait`，有多个实现，如

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

`Add trait` 就是一个典型的特设多态，同样是加法操作，根据操作数据的不同进行不同的处理。



## 3.5 编写 trait 的例子

**需求：**

写一个字符串解析器，可以把字符串的某部分解析成某个类型。



**分析：**

1. 可以定义如下 `trait`：它有一个方法 `parse`，这个方法接收一个字符串引用，返回 `Self`

```rust
pub trait Parse {
  fn parse(s: &str) -> Self;
}
```

这个 `parse` 方法是静态方法，因为它的第一个参数和 `self` 无关，所以在调用时需要使用 `T::parse(str) `。

2. 接下来为 `u8` 这个数据结构来实现 `parse`，比如：`“123abc”` 会被解析出整数 `123`，而 `“abcd”` 会被解析出 `0`

   > 需要引入一个`Regex` 库使用正则表达式提取需要的内容，还需要使用 `str::parse` 函数 把一个包含数字的字符串转换成数字



### 3.5.1 方式1: 一般做法

单独为 `u8` 这个类型实现 `Parse trait`

1. `Cargo.toml` 添加依赖

```toml
[dependencies]
regex = "0.2"
```

2. 实现代码

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



### 3.5.2 方式2: 泛型参数实现 trait

在实现 `Parse trait` 时，可以用泛型参数来实现 `trait`，要注意对泛型参数做一定的限制

1. 不是任何类型都可以通过字符串解析出来。只能处理数字类型，并且这个类型还要能够被 `str::parse` 处理

   > `str::parse` 是一个泛型函数，它返回任何实现了 `FromStr trait` 的类型，所以这里对泛型参数的第一个限制是它必须实现了 `FromStr trait`

2. 当无法正确解析字符串时，要返回一个缺省值表示无法处理。上面代码会返回 0，但在使用泛型参数后，无法返回 0，因为 0 不一定是某个符合泛型参数的类型中的一个值

   > 在 `Rust` 标准库中有 `Default trait`，绝大多数类型都实现了这个 `trait`，来为数据结构提供缺省值，所以泛型参数的另一个限制是 `Default`



使用泛型的实现代码如下：

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

* 优点：通过对带有约束的泛型参数实现 `trait`，同一份代码就实现了 `u32 / f64` 等类型的 `Parse trait`

* 缺点：当无法正确解析字符串时，返回了缺省值，其实也有可能是出错了。

  > 这里返回缺省值的话，会跟解析 `“0abcd”` 这样的情况混淆，不知道解析出的 0，究竟是出错了，还是本该解析出 0



### 3.5.3 方式3: 带关联类型的 trait

更好的方式是 `parse` 函数返回一个 `Result`：

```rust
pub trait Parse {
    fn parse(s: &str) -> Result<Self, E>;
}
```

这里 `Result` 的 `E` 要返回的错误信息，在 `trait` 定义时并不确定，不同的实现者可以使用不同的错误类型，可以使用`关联类型`把这种灵活性留给 `trait` 的实现者



**带关联类型的 trait**

`Rust` 允许 `trait` 内部包含关联类型，实现时跟关联函数一样，它也需要实现关联类型。`trait` 方法里的参数或者返回值，都可以用关联类型来表述，而在实现有关联类型的 `trait` 时，只需要额外提供关联类型的具体类型即可。



为 `Parse trait` 添加关联类型，示例如下：

```rust
pub trait Parse {
    type Error; // 关联类型
    fn parse(s: &str) -> Result<Self, Self::Error>; // 返回关联类型Error
}
```

有了关联类型 `Error`，`Parse trait` 就可以在出错时返回合理的错误了，看修改后的代码

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

优点：我们允许用户把错误类型延迟到 `trait` 实现时才决定，这种带有关联类型的 `trait` 比普通 `trait`，更加灵活，抽象度更高。



# 4 特型对象（trait object ）

在 `Rust` 中，可以用泛型 和 特型对象（`trait object`）实现子类型多态



## 4.1 子类型多态

从严格意义上说，子类型多态是面向对象语言的专利。**如果一个对象 `A` 是对象 `B` 的子类，那么 `A` 的实例可以出现在任何期望 `B` 的实例的上下文中**，比如猫和狗都是动物，如果一个函数的接口要求传入一个动物，那么传入猫和狗都是允许的。



`Rust` 虽然没有父类和子类，但 `trait` 和实现 `trait` 的类型之间也是类似的关系，所以，`Rust` 也可以做子类型多态，例如

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

 `impl Animal` 是 `T: Animal` 的简写，所以也可以写成如下样子

```rust
fn name<T: Animal>(animal: T) -> &'static str;
```

这种泛型函数会根据具体使用的类型被单态化，编译成多个实例，是静态分派。静态分派效率很高，但有时类型可能很难在编译时决定。例如要写一个格式化工具，可以定义一个 `Formatter` 接口，然后创建一系列实现：

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

* 首先，使用什么格式化方法，只有当打开文件，分析出文件内容之后才能确定，我们无法在编译期给定一个具体类型

* 其次，一个文件可能有一到多个格式化工具，比如一个 `Markdown` 文件里有 `Rust` 代码，同时需要 `MarkdownFormatter` 和 `RustFormatter` 来格式化。

这里如果使用一个 `Vec` 来提供所有需要的格式化工具，那么，下面这个函数的 `formatters` 参数该如何确定类型，如

```rust
pub fn format(input: &mut String, formatters: Vec<???>) {
    for formatter in formatters {
        formatter.format(input);
    }
}
```

正常情况下，`Vec<T>` 容器里的类型需要是一致的，但此处无法给定一个一致的类型，此时就需要用到动态分派。



## 4.2 特型对象 和 动态分派

我们要有一种手段告诉编译器，此处仅需要任何实现了 `Formatter` 接口的数据类型。在 `Rust` 中，这种类型叫 特型对象（`Trait Object`），表示为 `&dyn T 或者 Box<dyn T>`，前者在栈上，后者分配在堆上。



特型对象是指向实现了给定特型的某个值的指针。例如 类型 `&dyn std::io::Write` 和 `Box<dyn std::io::Write> ` 都是特型对象，它们都是指向了 `Write` 特型的某个值的指针。



于是，上述格式化的代码可以写成：

```rust
pub fn format(input: &mut String, formatters: Vec<&dyn Formatter>) {
    for formatter in formatters {
        formatter.format(input);
    }
}
```

这样可以在运行时，构造一个 `Formatter` 的列表，传递给 `format` 函数进行文件的格式化，这就是动态分派。



最终完整代码如下：

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

// 接受一个特型对象参数
pub fn format(input: &mut String, formatters: Vec<&dyn Formatter>) {
    for formatter in formatters {
        formatter.format(input);
    }
}

fn main() {
    let mut text = "Hello world!".to_string();
  
    let html: &dyn Formatter = &HtmlFormatter; 
    let rust: &dyn Formatter = &RustFormatter;
  
    let formatters = vec![html, rust]; // 传入一个列表
  
    format(&mut text, formatters);

    println!("text: {}", text);
}
```

`Rust` 在需要时，会自动将普通引用转换为特型对象。



注意：`Rust` 中不允许 `dyn Write` 类型的变量，如

```rust
use std::io::Write;

let mut buf: Vec<u8> = vec![];
let writer: dyn Write = buf; // 错误，Write的大小不是常量
```

这里代码是错误的，因为变量的大小必须是编译期已知的，而那些实现了 `Write` 的类型可以是任意大小的。`Rust` 中 `dyn` 类型是无固定大小类型，它是特型对象的引用目标。但是如下代码是正确的

```rust
use std::io::Write;

let mut buf: Vec<u8> = vec![];
let writer: &mut dyn Write = buf; // 正确
```

此时 `writer` 叫特型对象，因为对特型类型的引用叫做特型对象。与任何引用一样，特型对象指向某个值，它具有生命周期，并且可以是可变或共享的。不同的是，`Rust` 无法在编译期间知道引用目标的类型



**使用特型对象还是泛型代码？**

当需要一些混合类型的集合时，可以用特型对象。但泛型相比特型对象有 3 个优势

1. 速度快
2. 并不是每个特型都能支持特型对象
3. 泛型很容易同时指定具有多个特型的泛型参数限界 



## 4.3 特型对象的实现机理

当需要使用 `Formatter trait` 做动态分派时，可以像下面例子一样，将一个具体类型的引用，赋给 `&Formatter` ：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230214234134.png)





`HtmlFormatter` 的引用赋值给 `Formatter` 后，会生成一个 `Trait Object`，在上图中可以看到，特型对象的底层逻辑就是一个`胖指针`，其中一个指针指向数据本身，另一个指针则指向虚函数表（`vtable`）。



`vtable` 是一张静态的表，`Rust` 在编译时会为使用了 `trait object` 的类型的 `trait` 实现生成一张表，放在可执行文件中（一般在 `TEXT` 或 `RODATA` 段）。如下图

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/20230214234246.png)



在这张表里，包含具体类型的一些信息，如 `size`、`aligment` 以及一系列函数指针：

* 这个接口支持的所有的方法，比如 `format() `
* 具体类型的 `drop trait`，当 `Trait object` 被释放，它用来释放其使用的所有资源

这样当在运行时执行 `formatter.format() `时，`formatter` 就可以从 `vtable` 里找到对应的函数指针，执行具体的操作。



所以，`Rust` 里的 `Trait Object` 只不过是 `C++` 中 `vtable` 的一个变体而已。事实上，`Rust` 也并不区分原生类型和组合类型，对 `Rust` 来说，所有类型的地位都是一致的。



**注意：使用 `trait object` 时，要注意对象安全（`object safety`）。只有满足对象安全的 `trait` 才能使用 `trait object`。**



**怎么区分不是对象安全的 `trait`？**

如果 `trait` 所有的方法，返回值是 `Self`  或者携带 `泛型参数`，那么这个`trait` 就不能产生 `trait object`

1. 不允许返回 `Self`，是因为 `trait object` 在产生时，原来的类型会被抹去，所以 `Self` 不知道究竟是谁

   > 比如 `Clone trait` 只有一个方法 `clone()`，返回 `Self`，所以它就不能产生 `trait object`。

2. 不允许携带泛型参数，是因为 `Rust` 里带泛型的类型在编译时会做单态化，而 `trait object` 是运行时的产物，两者不能兼容

   > 比如 `From trait`，因为整个 `trait` 带了泛型，每个方法也自然包含泛型，就不能产生 `trait object`。

3. 如果一个 `trait` 只有部分方法返回 `Self` 或者使用了泛型参数，那么这部分方法在 `trait object` 中不能调用



## 4.4 特型对象的使用场景

特型对象有以下优缺点

* 优点：当在某个上下文中需要满足某个 `trait` 的类型，且这样的类型可能有很多，当前上下文无法确定会得到哪一个类型时，我们可以用 `trait object` 来统一处理行为。和泛型参数一样，`trait object` 也是一种延迟绑定，它让决策可以延迟到运行时，从而得到最大的灵活性
* 缺点：`trait object` 把决策延迟到运行时，带来的后果是执行效率的打折。在 `Rust` 里，函数或者方法的执行就是一次跳转指令，而 `trait object` 方法的执行还多一步，它涉及额外的内存访问，才能得到要跳转的位置再进行跳转，执行的效率要低一些。如果要把 `trait object` 作为返回值返回，或者要在线程间传递 `trait object`，都免不了使用 `Box<dyn T>` 或者 `Arc<dyn T>`，会带来额外的堆分配的开销



### 4.4.1 在函数的参数中使用

可以在函数的参数中使用 `trait object`。



例如：构建一个 `Executor trait`，并对比做静态分发的 `impl Executor`、做动态分发的 `&dyn Executor `和 `Box<dyn Executor>` 这几种不同的参数的使用，如

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

* 这里为了简化代码，使用了 `type` 关键字创建了一个 `BoxedError` 类型，是 `Box` 的别名，它是 `Error trait` 的 `trait object`，除了要求类型实现了 `Error trait` 外，它还有额外的约束：类型必须满足 `Send / Sync `这两个特型
* `impl Executor` 使用的是泛型参数的简化版本
* `&dyn Executor` 和 `Box<dyn Executor>>` 是 `trait object`，前者在栈上，后者分配在堆上。值得注意的是，分配在堆上的 `trait object` 也可以作为返回值返回，比如示例中的 `Result<Option<i32>, BoxedError>` 里使用了 `trait object`



### 4.4.2 在函数返回值中使用

在返回值中使用 `trait object`，是 `trait object` 使用频率比较高的场景。



先来看一些实战中会遇到的例子：首先是 [async_trait](https://docs.rs/async-trait)，它是一种特殊的 `trait`，方法中包含 `async fn`。以前[Rust 并不支持 trait 中使用 async fn](https://smallcultfollowing.com/babysteps/blog/2019/10/26/async-fn-in-traits-are-hard/)，一个变通的方法是使用 `async_trait` 宏

> 但是在最近的 [Rust 1.75.0](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html) 版本中，已经支持稳定的 `async fn`



如下定义的 `Fetch trait`：

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

它使用了 `trait object` 作为返回值。这样不管 `fetch()` 的实现，返回什么样的 `Future` 类型，都可以被 `trait object` 统一起来，调用者只需要按照正常 `Future` 的接口使用即可。



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

这是一个处理 [Noise Protocol](https://zhuanlan.zhihu.com/p/96944134) 使用何种加密算法的一个 `trait`。这个 `trait` 的每个方法，都返回一个 `trait object`，每个 `trait object `都提供加密算法中所需要的不同的能力。所有这些，都有一系列的具体的算法实现，通过 `CryptoResolver trait`，可以得到当前使用的某个具体算法的 `trait object`。在处理业务逻辑时，我们不用关心当前究竟使用了什么算法，就能根据这些 `trait object` 构筑相应的实现，比如下面的 `generate_keypair`：

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



### 4.4.3 在数据结构中使用

继续以 `snow` 的代码为例，看 `HandshakeState` 这个用于处理 `Noise Protocol` 握手协议的数据结构，用到了哪些 `trait object`，如

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

你不需要了解 `Noise protocol`，也能够大概可以明白这里 `Random`、`Dh` 以及 `Kem` 三个 `trait object` 的作用：它们为握手期间使用的加密协议提供最大的灵活性。**如果这里不用 `trait object`，这个数据结构该怎么处理？**可以用泛型参数，也就是说：

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

1. 首先，代码实现过程中，所有涉及的接口都变得非常臃肿，在使用 `HandshakeState` 的任何地方，都必须带着这几个泛型参数以及它们的约束
2. 其次，这些参数所有被使用到的情况，组合起来，会生成大量的代码

而使用 `trait object`，在牺牲一点性能的前提下，消除了这些泛型参数，实现的代码更干净清爽，且代码只会有一份实现。



**在数据结构中使用 `trait object` 还有一种很典型的场景是：闭包。**

因为在 `Rust` 中，闭包都是以匿名类型的方式出现，我们无法直接在数据结构中使用其类型，只能用泛型参数。而对闭包使用泛型参数后，如果捕获的数据太大，可能造成数据结构本身太大；但有时，我们并不在意一点点性能损失，更愿意让代码处理起来更方便。



例1：比如用于做 `RBAC` 的库 [oso](https://github.com/osohq/oso) 里的 `AttributeGetter`，它包含了一个 `Fn`

```rust
#[derive(Clone)]
pub struct AttributeGetter(
    Arc<dyn Fn(&Instance, &mut Host) -> crate::Result<PolarValue> + Send + Sync>,
);
```

例2：再比如做交互式 `CLI` 的 [dialoguer](https://github.com/mitsuhiko/dialoguer) 的 [Input](https://docs.rs/dialoguer/0.8.0/dialoguer/struct.Input.html)，它的 `validator` 就是一个 `FnMut`

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



# 5 参考

* [Rust程序设计（第2版）](https://book.douban.com/subject/36547630/)
* [陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/420028)
* [泛型和特征](https://course.rs/basic/trait/intro.html)
* [Trait 和 Trait Object](https://rust-book.junmajinlong.com/ch11/00.html)


