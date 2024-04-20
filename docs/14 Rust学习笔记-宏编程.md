# 1 Rust 的宏编程

`Rust` 提供了强大的宏编程能力，使其具有元编程能力。宏编程的本质就是把一棵语法树转换成另一颗语法树，主要作用是避免写大量结构重复的脚手架代码。在编译期间，在检查类型并生成任何机器码之前，每个宏调用都会被展开

> 元编程能力：如果一门编程语言把它在解析过程中产生的语法树（`AST`）暴露给开发者，允许开发者对语法树进行裁剪和嫁接，那么它就具备了元编程的能力，如 `Lisp` 语言



`Rust` 的宏分为两大类：声明宏 （`declarative macro`）和 过程宏（`procedural macro`）



## 1.1 声明宏

声明宏：主要是对代码模板进行替换，通过声明宏把重复的代码包装起来，然后在调用的地方展开成源码后，跟其他代码一起编译，这个过程不涉及语法树的操作

> 比如 `vec![]、println!、以及 info!` 都是声明宏



**什么情况可以使用声明宏?**

如果重复性的代码无法用函数来封装，那么可以选择用声明宏

> 比如 `Rust` 早期版本中的 `try!`，它是 `?` 操作符的前身



## 1.2 过程宏

过程宏：主要用于进行深度操作和改写代码语法树，更加灵活强大



**过程宏可以细分为3种**

1. 函数宏（`function-like macro`）：看起来像函数的宏，但是是在编译期进行处理的

   > 比如 `sqlx` 里的 [query](https://docs.rs/sqlx/0.5.10/src/sqlx/macros.rs.html#302-318) 宏，它内部展开出一个 [expand_query](https://github.com/launchbadge/sqlx/blob/335eed45455daf5b65b9e36d44d7f4343ba421e6/sqlx-macros/src/lib.rs#L27-L42) 函数宏

2. 属性宏（`attribute macro`）：可以在其他代码块上添加属性，为代码块提供更多功能

   > 比如 `rocket` 的 [get/put](https://docs.rs/rocket_codegen/0.4.10/src/rocket_codegen/lib.rs.html#329) 等路由属性

3. 派生宏（`derive macro`）：用 `derive` 属性添加新的功能，常用自动派生某些 `trait`，是平时使用最多且最复杂的宏

   > 比如 `Rust` 的 `serde` 库，我们的数据结构只需要添加 `#[derive(Serialize, Deserialize)]` 宏，就可以轻松序列化成 `JSON`、`YAML` 等好多种类型（或者从这些类型中反序列化）



**什么情况可以使用过程宏？**

* 派生宏：派生宏可以在特定的场景使用。一般来说，如果你定义的 `trait` 别人实现起来有固定的模式可循，那么可以考虑为其构建派生宏

  > 比如一个数据结构要提供 `Debug trait` 的能力，但为自己定义的每个数据结构实现 `Debug trait` 太过繁琐，而且代码所做的操作又都是一样的，这时就可以考虑使用派生宏来简化这个操作。
  >
  
* 函数宏和属性宏并没有特定的使用场景。例如 `sqlx` 用函数宏来处理 `SQL query`，在`tokio` 中使用属性宏 

  `#[tokio::main]` 来引入 `runtime`。它们可以帮助目标代码的实现逻辑变得更加简单，但一般除非特别必要，否则并不推荐写



# 2 声明宏

## 2.1 macro_rules!

**先看下 assert_eq! 宏**

 `assert_eq!` 宏的部分源码如下

```rust
macro_rules! assert_eq {
    ($left:expr, $right:expr $(,)?) => {
        match (&$left, &$right) { // 借用对 $left 和 $right 值的引用
            (left_val, right_val) => { // 创建新变量 left_val 和 right_val
                if !(*left_val == *right_val) {
                    let kind = $crate::panicking::AssertKind::Eq;
                  
                    $crate::panicking::assert_failed(kind, &*left_val, &*right_val, $crate::option::Option::None);
                }
            }
        }
    };
}
```

分析：

* 注意 `match` 中是借用对 `$left` 和 `$right` 值的引用，如果不是引用，直接传值，会使得该值被移动

* 创建了新变量 `left_val` 和 `right_val`

  > 因为这样只会计算一次 `$left` 和 `$right` 并存储它们的值，在接下去展开代码中才不会出现计算不一致的结果。如果没创建新变量，例如 `assert_eq!(letters.pop(), Some('z'))`，`Rust` 会将匹配的表达式插入模板中的多个位置，每个位置都会重新执行 `letters.pop()`，这样每次调用后它的值都会发生变化



**怎么定义声明宏**

声明宏可以用 `macro_rules!  ` 定义。`macro_rule!` 是内置于编译器的宏，`macro_rules!` 会使用模式匹配（类似 `match` 匹配），且可以提供多个匹配模式以及匹配后执行对应的代码块，宏的主体只是一系列规则，如

```rust
(pattern1) => (template1);
(pattern2) => (template2);
```

* 与 `match` 不同的是，宏里的值是一段 `Rust` 源代码(字面量)，模式用于跟这段源代码的结构相匹配，一旦匹配某个模式，传入宏的那段源代码将被模式关联的代码所替换，最终实现宏展开
* 定义宏时不用加感叹号，但调用宏时一定要标有感叹号
* 可以在模式 或 模板周围随意使用方括号 或 花括号 来代替圆括号，这对 `Rust` 没有任何影响，唯一的区别是花括号后面的分号通常是可选的。在调用宏时下面这些也是等效的。例如

```rust
// 以下3 种调用方式等价
assert_eq!(gcd(6, 10), 2);
assert_eq![gcd(6, 10), 2];
assert_eq!{gcd(6, 10), 2};

fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}
```

一般来说， 调用 `assert_eq!` 使用圆括号 `()`，调用 `vec!` 使用方括号 `[]`，调用 `macro_rules!` 时使用花括号 `{}`。



**宏展开**

`Rust` 展开 `assert_eq!` 宏调用的过程与对 `match` 表达式求值很像。下面讲下 `assert_eq!(gcd(6, 10), 2)` 的宏展开。

`Rust` 会首先将参数与模式进行匹配。`gcd(6, 10) , 2` 会匹配模式 `($left:expr, $right:expr $(,)?)`

* `gcd(6, 10)` 匹配 `$left: expr`：该模式片段表示 `Rust` 要匹配一个表达式（此时是 `gcd(6,10)`），并将其命名为 `$left`
* `,` 匹配 `,` ：逗号会匹配逗号
* `2` 匹配 `$right:expr`：`Rust` 会匹配表达式 `2` 并将其命名为 `$right`
* 模式中 `$(,)?` 表示匹配 `0` 次或 `1` 次逗号

注意：这个  `($left:expr, $right:expr $(,)?)` 模式中的两个代码片段都是 `expr` 类型的，表示它们期待表达式，`$left:expr` 就是一个 `expr` 类型的片段，不能直接写成 `$left`。



匹配完后，`Rust` 展开了相应的模板，如

```rust
{  // $left替换为 gcd(6,10) , $right替换为2。注意前面还有一个 & 引用符号
   // 类似于 match (&gcd(6,10), &2)
    match (&$left, &$right) { 
        (left_val, right_val) => {
            if !(*left_val == *right_val) { // 这里是解引用
                let kind = $crate::panicking::AssertKind::Eq;
                    $crate::panicking::assert_failed(kind, &*left_val, &*right_val, $crate::option::Option::None);
            }
        }
    }
}
```

`Rust` 会将 `$left` 和 `$right` 替换为它在匹配过程中找到的代码片段，即 `$left` 替换为 `gcd(6,10)` , `$right` 替换为 `2`。



## 2.2 构建 vec! 宏

来创建一个类似 `vec` 的 `my_vec` 宏，先用 `cargo new macrostest --lib` 创建一个新项目，然后新建 `example/rule.rs`，代码如下

> 该例子完整代码可参考[这里]( https://github.com/sinkhaha/rust-study-demo/blob/main/macrostest/examples/rule.rs)

```rust
#[macro_export]
macro_rules! my_vec {
    // 没带任何参数的 my_vec，我们创建一个空的 vec
    () => {
        std::vec::Vec::new() // 这里用了完整的命名空间
    };
  
    // 处理 my_vec![1, 2, 3, 4]
    ($($el:expr),*) => ({
        let mut v = std::vec::Vec::new();
        $(v.push($el);)*
        v
    });
  
    // 处理 my_vec![0; 10]
    ($el:expr; $n:expr) => {
        std::vec::from_elem($el, $n)
    }
}

fn main() {
    let mut v = my_vec![];
    v.push(1);
  
    // 调用时可以使用 [], (), {}
    let _v = my_vec!(1, 2, 3, 4);
    let _v = my_vec![1, 2, 3, 4];
    let v = my_vec! {1, 2, 3, 4};
    println!("{:?}", v);

    println!("{:?}", v);
    
    let v = my_vec![1; 10];
    println!("{:?}", v);
}
```

最后运行 ` cargo run --example rule ` 即可。用 `#[macro_export]` 将宏进行了导出，这样其它的包就可以将该宏引入到当前作用域中，然后才能使用。在使用标准库我们可以直接使用 `vec!` 宏，是因为 `Rust` 已经通过`std::prelude` 的方式自动引入了。



## 2.3 模式

以上 `my_vec!` 宏的代码写了 3 个匹配模式，下面说明下这 3 个模式的含义

1、第一个匹配条件是 `()`，表示如果没有传入任何参数，就创建一个新的 `Vec`

> 注意：由于宏要在调用的地方展开，我们无法预测调用者的环境是否已经做了相关的 `use`，所以代码最好带着完整的命名空间，如 `std::vec::Vec::new()`

2、第二个匹配条件 ` ( $( $el:expr ),* )`

* 从外往里看，整个`宏模式`被圆括号 `()` 包裹起来。里面是 `$()`，在声明宏中，条件捕获的参数使用 `$` 开头的标识符来声明，所以跟 `$()` 中模式相匹配的值(传入的 `Rust` 源代码)会被捕获，然后用于代码替换
* 条件捕获的每个参数都需要提供类型，这里 `expr` 是表达式类型，所以模式 `$el:expr` 会匹配任何 `Rust` 表达式，并把匹配到的表达式命名为 `$el`，所以捕获到的每一个表达式可以用 `$el `来访问
* `$(...),*` 说明可以匹配任意多个以逗号分隔的表达式。这里`$()` 之后的逗号说明在 `$()` 所匹配的代码的后面会有一个可选的逗号分隔符，在逗号之后的 `*` ，说明 `*` 之前的模式会被匹配 0次或多次（类似正则表达式）

3、第三个匹配条件 `($el:expr; $n:expr)`：说明传入用冒号分隔的两个表达式，那么会用 `from_element` 构建 `Vec`



**假如使用 `my_vec![1, 2, 3]` 来调用，此时模式解析和代码替换如下**

1、此时会匹配到 `($($el:expr),*)` 模式，`$(...),*` 表示可以匹配括号内的模式  `$x:expr`  0 次或 多次，且以逗号分隔，这里多次用到了重复的特性。此时 `expr` 是表达式类型，因此 `$el` 模式将被匹配三次，分别是 1、2、3。

一些常用的模式和含义如下：

| 模式     | 含义                          |
| -------- | ----------------------------- |
| $(...)*  | 匹配 0 次或 多次，没有分隔符  |
| $(...),* | 匹配 0 次或 多次，以逗号分隔  |
| $(...);* | 匹配 0 次或 多次，以分号分隔  |
|          |                               |
| $(...)+  | 匹配 1 次或 多次，没有分隔符  |
| $(...),+ | 匹配 1 次或 多次，以逗号分隔  |
| $(...);+ | 匹配 1 次或 多次，以分号分隔  |
|          |                               |
| $(...)?  | 匹配 0 次 或 1 次，没有分隔符 |

* 星号 * 与正则表达式中的星号 * 具有相同的含义，表示 0 或 更多
* 加号 + 表示至少匹配 1。次
* 问号 ? 表示匹配 0 次 或 1次



2、接着看匹配到对应模式后，要执行的代码替换，由于匹配的时候匹配到一个 `$(...)*` （可以不管分隔符），在执行的代码块中，也要相应地使用 `$(...)* `展开。所以 `$(v.push($el);)*` 相当于匹配出多少个 `$el` 就展开多少句 `push` 语句。当调用 `my_vec![1, 2, 3]` 时，下面这段生成的代码将替代传入的源代码：

```rust
{
    let mut v = std::vec::Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    v
}
```

如果是 `let my_v = my_vec![1, 2, 3]`，那生成的代码最后返回的值 `v` 将被赋予给变量 `my_v`，等同于 :

```rust
let my_v = {
   let mut v = std::vec::Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    v
}
```



## 2.4 模式匹配参数类型

在使用声明宏时，需要为模式匹配到参数明确类型，比如前面例子中的 `expr`，即表示表达式。`Rust` 中 `macro_rules!` 支持的片段类型如下

| 片段类型 | 匹配（带例子）                                               | 后面可以跟的语法标记         |
| -------- | ------------------------------------------------------------ | ---------------------------- |
| expr     | 表达式：2+2、"udon"、x.len()                                 | => ,  ;                      |
| stmt     | 表达式 或 声明，不包括任何尾随分号                           | => , ;                       |
| ty       | 类型：String、`Vec<u8>`、(&str, bool)                        | => , ; = \| { [ : > as where |
| path     | 路径：ferns、::std::sync::mpsc                               | => , ; = \| { [ : > as where |
| pat      | 模式：_,Some(ref x)                                          | => , = \| if  in             |
| item     | 语法项/标识符，比如一个变量名、函数、结构体、模块等：如 struct Point { x: f64, y: f64 } | 任意                         |
| block    | 代码块：如 { s += "ok\n"; true }                             | 任意                         |
| meta     | 属性的主体：inline，derive(Copy, Clone)                      | 任意                         |
| literal  | 字面量值：1024, "hello world"                                | 任意                         |
| lifetime | 生命周期：'a，'item，'static                                 | 任意                         |
| vis      | 可见性说明符：pub、pub(crate)                                | 任意                         |
| ident    | 标识符：std、Json                                            | 任意                         |
| tt       | 语法标记树: ; 、=>、{}                                       | 任意                         |

> 说明：例如 `expr`，说明是一个表达式就能匹配该类型类型， 表达式后面可以跟 `=>` 或 ` ,` 或 ` ;` 



## 2.5 构建 json! 宏

需求：开发一个用于构建 `JSON` 数据的宏，将一个 `JSON` 值作为参数并展开为 `Rust` 表达式的 `json!` 宏，如

```rust
let students = json!([
  {
    "name": "zhangsan",
    "class_of": 1990,
    "major": "singing"
  }
]);
```



编写一个宏时，第一步要弄清楚如何匹配 或 解析所期望的输入。`JSON` 数据有多种类型：对象、数组、数值等。可以先猜测每种 `JSON` 数据都有一条规则，代码如下：

```rust
macro_rules! json {
  (null) => { Json::Null };
  ([ $( $element:expr ),* ]) => { Json::Arrary(vec![ $($element),* ]) };
  ({...}) => { Json::Object(....) };
  (???) => { Json::Boolean(...) };
  (???) => { Json::Number(...) };
  (???) => { Json::String(...) };
}
```

先测试下第1种模式，此时是正常的

```rust
macro_rules! json {
  (null) => { Json::Null };
}

#[test]
fn json_null() {
  assert_eq!(json!(null), Json::Null); // 通过
}
```

接着测试下第2种匹配数组模式，此时不通过

```rust
macro_rules! json {
  ([ $( $element:expr ),* ]) => { Json::Arrary(vec![ $($element),* ]) };
}

#[test]
fn json_array() {
  let value = json!(
    [ // 无法匹配 $element:expr  的有效JSON
      {
        "age": 18
      }
    ]
  );
  let value1 = Json::Array( vec![ 
     Json::Object(
       Box::new( vec![("age".to_string(), Json::Number(18))].into_iter().collect() )
     )
  ]);
  assert_eq!(value, value1);
}
```

因为模式 `$( $element:expr ),*` 表示 以逗号分隔的 `Rust` 表达式列表，但是许多 `JSON` 值，尤其是对象，并不是有效的 `Rust` 表达式，所以它们无法匹配。正确的做法是需要用到语法标记树 `tt` 类型去匹配，因为每个 `JSON` 值都是一个语法标记树：数值、字符串、布尔值 和 `null` 是单个语法标记，对象和数组则是有括号的语法标记。所以正确写法如下

```rust
macro_rules! json {
  (null) => { Json::Null };
  ([ $( $element:tt ),* ]) => { Json::Arrary(...) };
  ({ $( $element:tt, $value:tt ),* ]}) => { Json::Object(....) };
  ($other: tt) => { ...// 返回 Number、String、Boolean };
}
```



我们需要将数组的每个元素从 `JSON` 格式转换成 `Rust` ，可以使用目前正在写的 `json!`  宏，对象类型也可以使用递归，如

```rust
// 数组
([ $( $element:tt ),* ]) => { Json::Arrary(vec![ $( json!($element) ),* ]) };

// 对象
({ $( $element:tt, $value:tt ),* ]}) => { Json::Object(
  Box::new( vec![ $(( $key.to_string(), json!($value) )),* ].into_iter().collect() )
)};
```

注意：编译器会对宏施加递归限制，默认最多递归 64 层。可以通过使用宏的 `crate` 顶部添加如下属性调整它：

```rust
#![recursion_limit = "256"]
```



这个宏还要支持布尔值、数字和字符串值，即需要支持 `json!(true)`，`json!(1.0)`，`json!("yes")` 将值转换为适当类型的 `Json` 值，这里可以使用 `From` 特型将各种类型的值转换为一种指定类型，例如

```rust
impl From<bool> for Json {
  fn from(b: bool) -> Json {
    Json::Boolean(b)
  }
}

impl From<i32> for Json {
  fn from(b: i32) -> Json {
    Json::Number(i as f64)
  }
}

impl From<String> for Json {
  fn from(b: String) -> Json {
    Json::String(s)
  }
}

impl<'a> From<&'a str> for Json {
  fn from(s: &'a str) -> Json {
    Json::String(s.to_string())
  }
}
......
```

其实 12 种数值类型都有非常相似的实现，没必要一个个去复制粘贴，可以写一个宏统一处理展开

```rust
macro_rules! impl_from_num_for_json {
    ( $( $t:ident )* ) => {
        $(
            impl From<$t> for Json {
                fn from(n: $t) -> Json {
                    Json::Number(n as f64)
                }
            }
        )*
    };
}


impl_from_num_for_json!(u8 i8 u16 i16 u32 i32 u64 i64 u128 i128
                        usize isize f32 f64);

```

接着可以使用 `Json::from(value)` 将任何受支持类型的 `value` 转换为 `Json` 值了，此时为

```rust
($other: tt) => {
  Json::from($other) // 处理布尔值、数值 和 字符串
}
```



所以最终这个 `json!` 宏的代码如下

```rust
macro_rules! json {
  (null) => { Json::Null };
  // 数组
  ([ $( $element:tt ),* ]) => {
    Json::Array(vec![ $( json!($element) ),* ]) 
  };
  // 对象
  ({ $( $element:tt, $value:tt ),* }) => {
    Json::Object(
      Box::new( vec![ $(( $key.to_string(), json!($value) )),* ].into_iter().collect() )
  )};  
  // 布尔值、数值 和 字符串等
  ($other: tt) => {
      Json::from($other) // 处理布尔值、数值 和 字符串
  };
}

```

> 这里用到了 `Json`、`Box`，记得要导入它们



## 2.6 作用域界定卫生宏

编写宏时一个非常棘手的问题是需要来自不同作用域的代码粘贴在一起。



`Rust` 中处理作用域的两种方式：一种方式用于局部变量和参数，另一种方式用于其他一切。



因为宏的展开会将代码展开，如果我们在宏中定义了自己的变量，一般来说宏里面的变量会跟宏的调用处的同名变量出现冲突。但实际上，宏却能正常工作，因为 `Rust` 会替你重命名宏内的变量，这个特型是首先在 `Scheme` 语言的宏中实现的，被称为卫生的，因此 `Rust` 被称为支持卫生宏的语言。`Rust` 中的卫生工作仅限于局部变量和参数，对于常量、类型、方法、模块、静态值和宏名称，`Rust` 是不卫生的。



假如把解析 `JSON` 对象的规则重写一些，宏内使用变量的方式

```rust
 ({ $( $element:tt, $value:tt ),* ]}) => {
     let mut fields = Box::new(HashMap::new());
     
     $( fields.insert($key.to_string(), json!($value)); )*
   
     Json::Object(fields)
   }  
};  
```

* 这里宏展开后的 `fields` 变量可能跟调用处有同名的 `fields`。不过 `Rust` 是卫生宏，所以这里能区分出跟调用处是不一样的 `fields` 变量。

* 如果宏确实需要引用调用者作用域内的变量，则调用者必须将变量的名称传给宏



## 2.7 导入宏和导出宏

由于宏会在编译早期展开，那会 `Rust` 还不知道项目的完整模块结构，因此编译器需要对宏的导入和导出进行特殊的支持。



在一个模块中可见的宏会自动在其子模块中可见。



**#[macro_use]**

 `#[macro_use]` 是一个属性，它的作用是在当前模块或 `crate` 中引入所有宏，它通常放在模块或 `crate` 的顶部。当你在一个模块上使用 `#[macro_use]` 时，你实际上是在告诉 `Rust` 编译器，你想要在当前模块及其子模块中使用指定 `crate` 中定义的所有宏，而不需要在每个宏的使用点前都显式地指定宏的路径。



一些使用场景：

1、导入标准库的宏，例如此时导入 `std` 库中的宏

```rust
#[macro_use]
extern crate std;
```

2、导入第三方库的宏，例如想要在 `crate` 的多个地方使用第三方库 `serde`提供的宏，如 `Serialize` 和 `Deserialize`，可以在 `crate` 根使用 `#[macro_use]` 来导入这些宏

```rust
#[macro_use]
extern crate serde;
```

3、导入当前 `crate` 定义的宏，如果在 `crate` 中定义了一些宏，并且希望在 `crate` 的不同模块中使用它们，可以在宏定义模块上使用 `#[macro_use]` 来使它们全局可用

```rust
// 在宏定义模块上
pub mod my_macros {
    // 宏定义...
}

#[macro_use]
pub mod my_macros;
```



**#[macro_export]**

 `#[macro_export]` 是一个属性宏，它的作用是：将定义在当前模块中的宏导出为公共宏，这样其他 `crate` 就可以直接使用这些宏而不需要再次声明 `#[macro_use]`，且可以像公共函数或结构体一样通过路径引用。



使用场景：

* 创建公共宏：例如当自定义一个宏给当前 `crate` 的其他部分 或 其他 `crate` 使用时

  > 这意味着这个宏可能会在其他模块中调用。因此导出的宏不应该依赖作用域内的任何内容

* 简化宏使用：使用 `#[macro_export]` 可以避免在每个使用宏的模块中都重复写 `#[macro_use]` 属性



**$crate**

`$crate` 是一个宏变量，它在宏的定义中使用，它在宏展开时被替换为当前 `crate` 的名称。



宏定义时应该对它用到的任何名称都使用绝对路径，当在 `Rust` 中定义一个宏时，`$crate` 可以用来引用宏调用所在的当前模块。这是因为宏展开是在编译时进行的，而模块结构在编译时是已知的。



例如上面实现的 `json!` 宏，可以写成 `$crate::Json` 而非 `Json` 的形式，这样即使没有导入 `Json` 也能正常工作。`HashMap` 可以被更改为 `::std::collections::Hashmap` 或 `$crate::macros::HashMap`

> 完整代码见[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/json-macro/src/macros.rs)

```rust
// macros.rs
pub use std::collections::HashMap;
pub use std::boxed::Box;
pub use std::string::ToString;

#[macro_export]
macro_rules! json {
   (null) => { $crate::Json::Null }; // 用成$crate::Json
  
   ([ $( $element:tt ),* ]) => {
      $crate::Json::Arrary(vec![ $( json!($element) ),* ]) 
   };
  
   ({ $( $element:tt, $value:tt ),* ]}) => {
         let mut fields = $crate::macros::Box::new($create::macros::HashMap::new());
     
         $( fields.insert($crate::macros::ToString::to_string($key), json!($value)); )*
   
         $crate::Json::Object(fields)
      }
   };  

   ($other: tt) => {
      $crate::Json::from($other) // 处理布尔值、数值 和 字符串
   };
}

```



## 2.8 内置宏

`Rust` 编译器提供了几个内置宏，它们在定义自己的宏时很有用，这些宏都不能使用 `macro_rule!` 来实现，它们是硬编码在 `rustc` 中的，例如

* `file!()`：文件名，它会展开为字符串字面量，即当前文件名

* `line!()`：行号，它会展开为 `u32` 字面量，以给出当前行号(从1开始计数)

* `column!()`：列号，它会展开为 u32 字面量，以给出当前列号(从1开始计数)

* `stringify!( ...tokens...)`：代码字符串，该宏会展开为包含给定语法标记的字符串字面量。`assert!` 宏会使用它来生成包含断言代码的错误信息。它的参数中的宏调用不会被展开，比如 `stringify!(line!())` 会展开为字符串 `"line!()"`
* `concat!(str0, str1, ...)`：串联，该宏会通过串联它的各个参数展开为单个字符串字面量



`Rust `还定义了用于查询构建环境的宏:

* `cfg!()`：配置，该宏会展开为布尔常量，如果当前正在构建的配置与圆括号中的条件匹配则为 `true`。如果在启用了调试断言的情况下进行编译，则`cfg!(debug_assertions)` 为 `true`

* `env!("VAR_NAME")`：环境变量，展开为字符串，即在编译期指定的环境变量的值，如果该变量不存在，则为编译错误，例如要获取 `crate` 的当前版本的字符串

  ```rust
  let version = env!("CARGO_PKG_VERSION");
  ```

* `option_env!("VAR_NAME")`：可选环境变量，和 `env!` 基本一样，不过该宏会返回 `Option<&'static str'>`，如果没有设置指定的变量，则返回 `None`



## 2.9 调试宏

**调试宏的几种方法**

1、使用 `rustc` 查看代码在展开所有宏后的样子。先使用`cargo build --verbose` 查看 `Cargo` 如果调用 `rustc` 。接着复制 `rustc` 命令并添加 `-Z unstable-options --pretty expanded` 选项，这样完全展开的代码将转储到终端，前提是当前代码没有语法错误

2、使用 `log_syntax!` 宏。它会让编译器输出所有经过编译器处理的标记。它只会在编译期将自己的参数打印到终端，可以将它用于 `println!` 式调试。使用时需要添加 `#![feature(log_syntax)]` 的特性标志。

3、使用 `trace_macros!` 。 在代码中插入 `trace_macros!(true)` 可以让编译器把所有宏调用记录到终端，每当 `Rust` 展开宏时，它都会打印宏的名称和各个参数，`trace_macros!(false)` 则关闭了跟踪。例如

```rust
#![feature(trace_macros)]

fn main() {
  trace_macros!(true);
  let numbers = vec![1, 2, 3];
  trace_macros!(false);
  println!("total:{}", numbers.iter().sum::<u64>());
}
```

因为`trace_macros!(false)` 关闭了跟踪，因此不会跟踪对 `println!()` 的调用



# 3 过程宏

过程宏要比声明宏要复杂很多，其实 3 种过程宏的本质都是一样的，都涉及要把输入的 `TokenStream` 处理成输出的 `TokenStream`。



注意：当创建过程宏时，它的定义必须要放入一个独立的`crate` 包中，且包的类型也是特殊的（`proc-macro`）。

> 过程宏放入独立包的原因在于它必须先被编译后才能使用。如果过程宏和使用它的代码在一个包，就必须先单独对过程宏的代码进行编译，然后再对我们的代码进行编译，但因为 `Rust`  的编译单元是包，因此无法做到这一点



## 3.1 创建函数宏

函数宏可以定义像函数那样调用的宏，从这个角度来看，它跟声明宏 `macro_rules!` 较为类似。



**创建过程**

> 该例子完整代码可参考[这里]( https://github.com/sinkhaha/rust-study-demo/blob/main/macrostest/examples/query.rs)

1、先用 `cargo new macrostest --lib` 创建项目，在 `Cargo.toml` 中添加 `proc-macro` 的声明，这样编译器才允许使用 `#[proc_macro]` 相关的宏

```rust
[lib]
proc-macro = true
```



2、接着 `src/lib.rs` 的代码如下

```rust
use proc_macro::TokenStream;

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    println!("{:#?}", input);
    "fn hello() { println!(\"Hello world!\"); }"
        .parse()
        .unwrap()
}
```

代码声明了它是一个 `proc_macro`，这是最基本的函数式的过程宏。这里打印了传入的 [TokenStream](https://doc.rust-lang.org/proc_macro/struct.TokenStream.html)，然后把一段包含在字符串中的代码解析成 `TokenStream` 返回，返回了一个 `hello()` 函数的 `TokenStream`

> 这里能使用字符串的 `parse()` 方法来获得 `TokenStream`，是因为 `TokenStream` 实现了 [FromStr trait](https://doc.rust-lang.org/std/str/trait.FromStr.html)。



3、该宏的调用方式类似于函数调用，可以通过  `query!(...)` 来调用，如

```rust
let sql = query!(SELECT * FROM users WHERE age > 10);
```



4、创建 `examples/query.rs` 编写如下测试代码

```rust
use macrostest::query;

fn main() {
    query!(SELECT * FROM users WHERE age > 10);
  
    // query!宏返回了一个 hello() 函数的 TokenStream，这里可以直接使用hellow函数，对 hello() 调用后输出Hello world!
    hello();
}
```



5、运行 `cargo run --example query`，看 `query` 宏对输入 `TokenStream` 的打印

```rust
TokenStream [
    Ident {
        ident: "SELECT",
        span: #0 bytes(47..53),
    },
    Punct {
        ch: '*',
        spacing: Alone,
        span: #0 bytes(54..55),
    },
    Ident {
        ident: "FROM",
        span: #0 bytes(56..60),
    },
    Ident {
        ident: "users",
        span: #0 bytes(61..66),
    },
    Ident {
        ident: "WHERE",
        span: #0 bytes(67..72),
    },
    Ident {
        ident: "age",
        span: #0 bytes(73..76),
    },
    Punct {
        ch: '>',
        spacing: Alone,
        span: #0 bytes(77..78),
    },
    Literal {
        kind: Integer,
        symbol: "10",
        suffix: None,
        span: #0 bytes(79..81),
    },
]
```

这里面 `TokenStream` 是一个 `Iterator`，里面包含一系列的 [TokenTree](https://doc.rust-lang.org/proc_macro/enum.TokenTree.html)：

```rust
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}
```

后面三个分别是 `Ident`（标识符）、`Punct`（标点符号）和 `Literal`（字面量）。这里的 `Group`（组）是因为如果代码中包含括号（比如`{} [] <> ()` ），那么内部的内容会被分析成一个 `Group`（组）。可以把例子中对 `query!` 的调用改成以下这个样子，再运行一下，此时 `TokenStream` 就包含了 `Group`：

```rust
query!(SELECT * FROM users u JOIN (SELECT * from profiles p) WHERE u.id = p.id and u.age > 10);
```



## 3.2 创建派生宏

### 3.2.1 需求

构建一个 `Builder` 派生宏，实现 [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop) 项目里 [06-optional-field需求](https://github.com/dtolnay/proc-macro-workshop/blob/master/builder/tests/06-optional-field.rs)

> `proc-macro-workshop` 是 `Rust` 大牛 `David Tolnay` 为帮助大家更好地学习宏编程构建的练习

```rust
use derive_builder::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        .build()
        .unwrap();
    assert!(command.current_dir.is_none());

    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        .current_dir("..".to_owned())
        .build()
        .unwrap();
    assert!(command.current_dir.is_some());
}
```

可以看到，要为 `Command` 这个结构提供 `Builder` 宏，让它支持 `builder()` 方法，返回了一个 `CommandBuilder` 结构，这个结构有若干个和 `Command` 内部每个域名字相同的方法，可以链式调用这些方法，最后 `build() `出一个 `Command` 结构。



### 3.2.2 不使用宏的实现方式

> 该例子完整代码可参考[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/macrostest/examples/manual-command.rs)

先 `cargo new macrostest --lib`创建项目，创建一个 `examples/manual-command.rs`，编写代码如下

```rust
#[allow(dead_code)]
#[derive(Debug)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: Option<String>,
}

#[derive(Debug, Default)]
pub struct CommandBuilder {
    executable: Option<String>,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    current_dir: Option<String>,
}

impl Command {
    pub fn builder() -> CommandBuilder {
        Default::default()
    }
}

impl CommandBuilder {
    pub fn executable(mut self, v: String) -> Self {
        self.executable = Some(v.to_owned());
        self
    }

    pub fn args(mut self, v: Vec<String>) -> Self {
        self.args = Some(v.to_owned());
        self
    }

    pub fn env(mut self, v: Vec<String>) -> Self {
        self.env = Some(v.to_owned());
        self
    }

    pub fn current_dir(mut self, v: String) -> Self {
        self.current_dir = Some(v.to_owned());
        self
    }

    pub fn build(mut self) -> Result<Command, &'static str> {
        Ok(Command {
            executable: self.executable.take().ok_or("executable must be set")?,
            args: self.args.take().ok_or("args must be set")?,
            env: self.env.take().ok_or("env must be set")?,
            current_dir: self.current_dir.take(),
        })
    }
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        .build()
        .unwrap();
    assert!(command.current_dir.is_none());

    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        .current_dir("..".to_owned())
        .build()
        .unwrap();
    assert!(command.current_dir.is_some());
    println!("{:?}", command);
} 
```

运行 `cargo run --example manual-command` 即可。这个代码基本就是照着 `main()` 中的使用方法写出来的，可以看到代码中很多重复的部分，尤其是 `CommandBuilder` 里的方法，这些可以用宏来自动生成。



### 3.2.3 使用宏的实现思路分析

**思路分析**：要用派生宏自动生成以上代码，首先要把输入的 `TokenStream` 抽取出来，也就是把在 `struct` 的定义内部，每个域的名字及其类型都抽出来，然后生成对应的方法代码。如果把代码看做是字符串的话，实际上就是要通过一个模板和对应的数据，生成我们想要的结果。



1、编写代码模板

> 可以用 [jinja](https://jinja.palletsprojects.com/en/3.0.x/) 写一个生成 `CommandBuilder` 结构的模板；在 `Rust` 有 [askma](https://github.com/djc/askama) 这个非常高效的库来处理 `jinja`

首先编写 `CommandBuilder` 模板如下，这里的 `fileds / builder_name` 是要传入的参数，每个 `field` 还需要 `name` 和 `ty` 两个属性，分别对应 `field` 的名字和类型。如

```rust
#[derive(Debug, Default)]
pub struct {{ builder_name }} {
    {% for field in fields %}
    {{ field.name }}: Option<{{ field.ty }}>,
    {% endfor %}
}
```

接着编写 `CommandBuilder` 结构生成方法的模板，如

```rust
impl {{ builder_name }} {
    {% for field in fields %}
    pub fn {{ field.name }}(mut self, v: impl Into<{{ field.ty }}>) -> {{ builder_name }} {
        self.{{ field.name }} = Some(v.into());
        self
    }
    {% endfor %}

    pub fn build(self) -> Result<{{ name }}, &'static str> {
        Ok({{ name }} {
            {% for field in fields %}
            {% if field.optional %}
            {{ field.name }}: self.{{ field.name }},
            {% else %}
            {{ field.name }}: self.{{ field.name }}.ok_or("Build failed: missing {{ field.name }}")?,
            {% endif %}
            {% endfor %}
        })
    }
}
```

对于原本是 ` Option<T>`类型的域，要避免生成 `Option<Option>`，所以需要把是否是 `Option` 单独抽取出来，如果是 `Option`，那么 `ty` 就是 `T`。所以，`field` 还需要一个属性 `optional`。



基于这个模板思路，可以构建一个数据结构来描述 `Field`：

```rust
#[derive(Debug, Default)]
struct Fd {
    name: String,
    ty: String,
    optional: bool,
}
```



2、当有了模板并定义好了为模板提供数据的结构，接着要处理的核心问题是：如何从 `TokenStream` 中抽取出来我们想要的信息。



可以先在前面[手工实现例子的 lib.rs ](https://github.com/sinkhaha/rust-study-demo/blob/main/macrostest/src/lib.rs)里添加一个 `derive macro`，把 `input` 打印出来进行分析：

```rust
// 这里使用 `proce_macro_derive` 这个宏去创建派生宏，这里把我们的派生宏命名为 RawBuilder。
#[proc_macro_derive(RawBuilder)]
pub fn derive_raw_builder(input: TokenStream) -> TokenStream {
    println!("{:#?}", input);
    TokenStream::default()
}
```

然后在 [examples/manual-command.rs](https://github.com/sinkhaha/rust-study-demo/blob/main/macrostest/examples/manual-command.rs) 中，修改 `Command` 结构，使其使用 `RawBuilder`（注意要 `use macrostest::RawBuilder`）：

```rust
use macrostest::RawBuilder;

#[allow(dead_code)]
#[derive(Debug, RawBuilder)]
pub struct Command {
    ...
    ...
}
```

最后运行 `cargo run --example manual-command` 后，会打印出 `TokenStream` 

```bash
TokenStream [
    Punct {
        ch: '#',
        spacing: Alone,
        span: #0 bytes(96..97),
    },
    Group {
        delimiter: Bracket,
        stream: TokenStream [
            Ident {
                ident: "allow",
                span: #0 bytes(98..103),
            },
            Group {
                delimiter: Parenthesis,
                stream: TokenStream [
                    Ident {
                        ident: "dead_code",
                        span: #0 bytes(104..113),
                    },
                ],
                span: #0 bytes(103..114),
            },
        ],
        span: #0 bytes(97..115),
    },
    Ident {
        ident: "pub",
        span: #0 bytes(191..194),
    },
    Ident {
        ident: "struct",
        span: #0 bytes(195..201),
    },
    Ident {
        ident: "Command",
        span: #0 bytes(202..209),
    },
    Group {
        delimiter: Brace,
        stream: TokenStream [
            Ident {
                ident: "executable",
                span: #0 bytes(216..226),
            },
            Punct {
                ch: ':',
                spacing: Alone,
                span: #0 bytes(226..227),
            },
            Ident {
                ident: "String",
                span: #0 bytes(228..234),
            },
            Punct {
                ch: ',',
                spacing: Alone,
                span: #0 bytes(234..235),
            },
            Ident {
                ident: "args",
                span: #0 bytes(240..244),
            },
            Punct {
                ch: ':',
                spacing: Alone,
                span: #0 bytes(244..245),
            },
            Ident {
                ident: "Vec",
                span: #0 bytes(246..249),
            },
            Punct {
                ch: '<',
                spacing: Alone,
                span: #0 bytes(249..250),
            },
            Ident {
                ident: "String",
                span: #0 bytes(250..256),
            },
            Punct {
                ch: '>',
                spacing: Joint,
                span: #0 bytes(256..257),
            },
            Punct {
                ch: ',',
                spacing: Alone,
                span: #0 bytes(257..258),
            },
            Ident {
                ident: "env",
                span: #0 bytes(263..266),
            },
            Punct {
                ch: ':',
                spacing: Alone,
                span: #0 bytes(266..267),
            },
            Ident {
                ident: "Vec",
                span: #0 bytes(268..271),
            },
            Punct {
                ch: '<',
                spacing: Alone,
                span: #0 bytes(271..272),
            },
            Ident {
                ident: "String",
                span: #0 bytes(272..278),
            },
            Punct {
                ch: '>',
                spacing: Joint,
                span: #0 bytes(278..279),
            },
            Punct {
                ch: ',',
                spacing: Alone,
                span: #0 bytes(279..280),
            },
            Ident {
                ident: "current_dir",
                span: #0 bytes(285..296),
            },
            Punct {
                ch: ':',
                spacing: Alone,
                span: #0 bytes(296..297),
            },
            Ident {
                ident: "Option",
                span: #0 bytes(298..304),
            },
            Punct {
                ch: '<',
                spacing: Alone,
                span: #0 bytes(304..305),
            },
            Ident {
                ident: "String",
                span: #0 bytes(305..311),
            },
            Punct {
                ch: '>',
                spacing: Joint,
                span: #0 bytes(311..312),
            },
            Punct {
                ch: ',',
                spacing: Alone,
                span: #0 bytes(312..313),
            },
        ],
        span: #0 bytes(210..315),
    },
]
```

从打印中可以看到：

* 首先有一个 `Group`，包含了 `#[allow(dead_code)]` 属性的信息。因为我们现在拿到的 `derive` 下的信息，所以所有不属于 `#[derive(...)]` 的属性，都会被放入 `TokenStream` 中
* 之后是 `pub / struct / Command` 三个` ident`
* 随后又是一个 `Group`，包含了每个 `field` 的信息。可以看到，`field` 之间用逗号这个 `Punct` 分隔，`field` 的名字和类型又是通过冒号这个 `Punct` 分隔。而类型，可能是一个 `Ident`，如 `String`，或者一系列 `Ident / Punct`，如 `Vec / < / String / >`



我们要做的就是，把这个 `TokenStream` 中的 `struct` 名字，以及每个 `field` 的名字和类型拿出来。如果类型是 `Option`，那么把 `T` 拿出来，把 `optional` 设置为 `true`。



### 3.2.4 编写派生宏实现需求

> 该例子完整代码可参考[这里](https://github.com/sinkhaha/rust-study-demo/tree/main/derivemacros)

1、首先创建 `cargo new derivemacros --lib` 项目，在 `Cargo.toml` 中引入依赖

```rust
[dependencies]
anyhow = "1"
askama = "0.11" # 处理 jinjia 模板，模板需要放在和 src 平行的 templates 目录下
```

2、接着，创建 `templates` 目录存放模板（`akama` 要求模板放在和 `src` 平行的 `templates` 目录下），然后创建`templates/builder.j2` 写入模板

```rust
impl {{ name }} {
    pub fn builder() -> {{ builder_name }} {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct {{ builder_name }} {
    {% for field in fields %}
        {{ field.name }}: Option<{{ field.ty }}>,
    {% endfor %}
}

impl {{ builder_name }} {
    {% for field in fields %}
    pub fn {{ field.name }}(mut self, v: impl Into<{{ field.ty }}>) -> {{ builder_name }} {
        self.{{ field.name }} = Some(v.into());
        self
    }
    {% endfor %}

    pub fn build(self) -> Result<{{ name }}, &'static str> {
        Ok({{ name }} {
            {% for field in fields %}
                {% if field.optional %}
                {{ field.name }}: self.{{ field.name }},
                {% else %}
                {{ field.name }}: self.{{ field.name }}.ok_or("Build failed: missing {{ field.name }}")?,
                {% endif %}
            {% endfor %}
        })
    }
}
```

3、然后创建 `src/raw_builder.rs`（记得在 `lib.rs` 中引入）；执行的逻辑就是把 `TokenStream` 中的 `struct` 名字，以及每个 `field` 的名字和类型拿出来，然后使用模板生成代码

```rust
use anyhow::Result;
use askama::Template;
use proc_macro::{Ident, TokenStream, TokenTree};
use std::collections::VecDeque;

/// 处理 jinja 模板的数据结构，builder.j2为模板，在模板中使用了 name / builder_name / fields
#[derive(Template)]
#[template(path = "builder.j2", escape = "none")]
pub struct BuilderContext {
    name: String,
    builder_name: String,
    fields: Vec<Fd>,
}

/// 描述 struct 的每个 field
#[derive(Debug, Default)]
struct Fd {
    name: String,
    ty: String,
    optional: bool,
}

impl Fd {
    /// name 和 field 都是通过冒号 Punct 切分出来的 TokenTree 切片
    pub fn new(name: &[TokenTree], ty: &[TokenTree]) -> Self {
        // 把类似 Ident("Option"), Punct('<'), Ident("String"), Punct('>) 的 ty
        // 收集成一个 String 列表，如 vec!["Option", "<", "String", ">"]
        let ty = ty
            .iter()
            .map(|v| match v {
                TokenTree::Ident(n) => n.to_string(),
                TokenTree::Punct(p) => p.as_char().to_string(),
                e => panic!("Expect ident, got {:?}", e),
            })
            .collect::<Vec<_>>();
      
        // 冒号前最后一个 TokenTree 是 field 的名字
        // 比如：executable: String,
        // 注意这里不应该用 name[0]，因为有可能是 pub executable: String
        // 甚至，带 attributes 的 field，
        // 比如：#[builder(hello = world)] pub executable: String
        match name.last() {
            Some(TokenTree::Ident(name)) => {
                // 如果 ty 第 0 项是 Option，那么从第二项取到倒数第一项
                // 取完后上面的例子中的 ty 会变成 ["String"]，optiona = true
                let (ty, optional) = if ty[0].as_str() == "Option" {
                    (&ty[2..ty.len() - 1], true)
                } else {
                    (&ty[..], false)
                };
                Self {
                    name: name.to_string(),
                    ty: ty.join(""), // 把 ty join 成字符串
                    optional,
                }
            }
            e => panic!("Expect ident, got {:?}", e),
        }
    }
}

impl BuilderContext {
    /// 从 TokenStream 中提取信息，构建 BuilderContext
    fn new(input: TokenStream) -> Self {
        let (name, input) = split(input);
        let fields = get_struct_fields(input);
        Self {
            builder_name: format!("{}Builder", name),
            name: name.to_string(),
            fields,
        }
    }

    /// 把模板渲染成字符串代码
    pub fn render(input: TokenStream) -> Result<String> {
        let template = Self::new(input);
        Ok(template.render()?)
    }
}

/// 把 TokenStream 分出 struct 的名字，和包含 fields 的 TokenStream
fn split(input: TokenStream) -> (Ident, TokenStream) {
    let mut input = input.into_iter().collect::<VecDeque<_>>();
    // 一直往后找，找到 struct 停下来
    while let Some(item) = input.pop_front() {
        if let TokenTree::Ident(v) = item {
            if v.to_string() == "struct" {
                break;
            }
        }
    }

    // struct 后面，应该是 struct name
    let ident;
    if let Some(TokenTree::Ident(v)) = input.pop_front() {
        ident = v;
    } else {
        panic!("Didn't find struct name");
    }

    // struct 后面可能还有若干 TokenTree，我们不管，一路找到第一个 Group
    let mut group = None;
    for item in input {
        if let TokenTree::Group(g) = item {
            group = Some(g);
            break;
        }
    }

    (ident, group.expect("Didn't find field group").stream())
}

/// 核心方法，从包含 fields 的 TokenStream 中切出来一个个 Fd，例如把一个 a=1,b=2 的字符串切成 [[a, 1], [b, 2]]
fn get_struct_fields(input: TokenStream) -> Vec<Fd> {
    let input = input.into_iter().collect::<Vec<_>>();
    input
        .split(|v| match v {
            // 先用 ',' 切出来一个个包含 field 所有信息的 &[TokenTree]
            TokenTree::Punct(p) => p.as_char() == ',',
            _ => false,
        })
        .map(|tokens| {
            tokens
                .split(|v| match v {
                    // 再用 ':' 把 &[TokenTree] 切成 [&[TokenTree], &[TokenTree]]
                    // 它们分别对应名字和类型
                    TokenTree::Punct(p) => p.as_char() == ':',
                    _ => false,
                })
                .collect::<Vec<_>>()
        })
        // 正常情况下，应该得到 [&[TokenTree], &[TokenTree]]，对于切出来长度不为 2 的统统过滤掉
        .filter(|tokens| tokens.len() == 2)
        // 使用 Fd::new 创建出每个 Fd
        .map(|tokens| Fd::new(tokens[0], &tokens[1]))
        .collect()
}
```

4、完成了把  `TokenStream` 转换成 `BuilderContext` 的代码，接下来就是在我们的 `RawBuilder` 宏中使用这个结构以及它的 `render` 方法，把 `lib.rs` 中的代码修改如下

```rust
mod raw_builder;

use proc_macro::TokenStream;
use raw_builder::BuilderContext;

#[proc_macro_derive(RawBuilder)]
pub fn derive_raw_builder(input: TokenStream) -> TokenStream {
    println!("input的值是 {:#?}", input);
    BuilderContext::render(input).unwrap().parse().unwrap()
}
```

5、最后 `examples/command.rs` 代码如下

```rust
use macros::RawBuilder;

#[allow(dead_code)]
#[derive(Debug, RawBuilder)] // 使用RawBuilder宏
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        .build()
        .unwrap();
    assert!(command.current_dir.is_none());

    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        .current_dir("..".to_owned())
        .build()
        .unwrap();
    assert!(command.current_dir.is_some());
    println!("{:?}", command);
}
```

6、最后运行 `cargo run --example command` 即可



## 3.3 创建属性宏

**属性过程宏跟派生宏类似，区别是**

1. 属性宏允许我们定义自己的属性
2. 派生宏只能用于结构体和枚举，而类属性宏可以用于其它类型项，例如函数



假设我们在开发一个 `web` 框架，当用户通过 `HTTP GET` 请求访问 `/` 根路径时，使用 `index` 函数为其提供服务:

```rust
#[route(GET, "/")]
fn index() {
}
```

这里的 `#[route]` 属性就是一个过程宏，它的定义函数大概如下：

```rust
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
}
```

与 派生宏不同，属性宏的定义函数有两个参数：

- 第一个是用于说明属性包含的内容：`Get, "/"` 部分
- 第二个是属性所标注的类型项，在这里是 `fn index() {...}`，注意，函数体也被包含其中

除此之外，类属性宏跟 派生宏的工作方式并无区别：创建一个包，类型是 `proc-macro`，接着实现一个函数用于生成想要的代码。



# 3 用 syn/quote 库构建派生宏

上面用最原始的方式构建了一个 `RawBuilder` 派生宏，本质就是从 `TokenStream` 中抽取需要的数据，然后生成包含目标代码的字符串，最后再把字符串转换成 `TokenStream`。



下面使用 [syn](https://github.com/dtolnay/syn) / [quote](https://github.com/dtolnay/quote) 来构建一个同样的 `Builder ` 派生宏。这两个库就是 `Rust` 宏生态下处理 `TokenStream` 的解析以及代码生成很好用的库。



## 3.1 syn crate 介绍

[syn](https://github.com/dtolnay/syn) 是一个对 `TokenStream` 解析的库，它提供了丰富的数据结构，对语法树中遇到的各种 `Rust` 语法都有支持。

> 比如一个 `Struct` 结构，在 `TokenStream` 中就是一系列 `TokenTree`，而通过 `syn` 解析后，`struct` 的各种属性以及它的各个字段，都有明确的类型。这样可以很方便地通过模式匹配来选择合适的类型进行对应的处理。



**DeriveInput类型**

`syn` 提供了对 `derive macro` 的特殊支持：[DeriveInput](https://docs.rs/syn/latest/syn/struct.DeriveInput.html)类型：

```rust
pub struct DeriveInput {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub data: Data,
}
```

通过 `DeriveInput` 类型可以很方便地解析派生宏。比如：

```rust
#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    ...
}
```

只需要使用 `parse_macro_input!(input as DeriveInput)`即可，不必和 `TokenStream` 打交道，而是使用解析出来的 `DeriveInput`，这里直接访问 `DeriveInput` 的 `ident` 域就能拿出来 `struct` 的名字



**Parse trait**

`parse_macro_input` [源码 ](https://docs.rs/syn/latest/src/syn/parse_macro_input.rs.html#108-128)如下

```rust
macro_rules! parse_macro_input {
    ($tokenstream:ident as $ty:ty) => {
        match $crate::parse_macro_input::parse::<$ty>($tokenstream) {
            $crate::__private::Ok(data) => data,
            $crate::__private::Err(err) => {
                return $crate::__private::TokenStream::from(err.to_compile_error());
            }
        }
    };
    ($tokenstream:ident with $parser:path) => {
        match $crate::parse::Parser::parse($parser, $tokenstream) {
            $crate::__private::Ok(data) => data,
            $crate::__private::Err(err) => {
                return $crate::__private::TokenStream::from(err.to_compile_error());
            }
        }
    };
    ($tokenstream:ident) => {
        $crate::parse_macro_input!($tokenstream as _)
    };
}
```

从代码可以看到，当调用 `parse_macro_input!(input as DeriveInput)`，实际上它执行了 `$crate::parse_macro_input::parse::(input)`，这个 `parse` 函数从何而来，继续看[代码](https://docs.rs/syn/latest/src/syn/parse_macro_input.rs.html#138-152)

```rust
pub fn parse<T: ParseMacroInput>(token_stream: TokenStream) -> Result<T> {
    T::parse.parse(token_stream)
}

pub trait ParseMacroInput: Sized {
    fn parse(input: ParseStream) -> Result<Self>;
}

impl<T: Parse> ParseMacroInput for T {
    fn parse(input: ParseStream) -> Result<Self> {
        <T as Parse>::parse(input)
    }
}
```

从代码可以看到，任何实现了 `ParseMacroInput trait` 的类型 `T`，都支持 `parse()` 函数。进一步的，**任何 `T`，只要实现了 `Parse trait`，就自动实现了 `ParseMacroInput trait`。**这个 [Parse trait](https://docs.rs/syn/latest/syn/parse/trait.Parse.html) 定义如下

```rust
pub trait Parse: Sized {
    fn parse(input: ParseStream<'_>) -> Result<Self>;
}
```

`syn` 下面几乎所有的数据结构都实现了 `Parse trait`，包括 `DeriveInput`。所以，如果想自己构建一个数据结构，可以通过 `parse_macro_input!  `宏从 `TokenStream` 里读取内容，并写入这个数据结构，**最好的方式是为我们的数据结构实现 `Parse trait`。**

> 关于 `Parse trait` 的更多使用，可以看看 [DeriveInput 对 Parse 的实现](https://docs.rs/syn/latest/src/syn/derive.rs.html#96-162)。也可以看一下 [sqlx](https://github.com/launchbadge/sqlx) 下的 `query!` 宏[内部对 Parse trait 的实现](https://github.com/launchbadge/sqlx/blob/335eed45455daf5b65b9e36d44d7f4343ba421e6/sqlx-macros/src/query/input.rs#L36-L110)。



## 3.2 quote crate 介绍

[quote](https://github.com/dtolnay/quote) 用于将 `Rust` 语法树数据结构转化为源代码。



前面在生成 `TokenStream` 时，使用的是最原始的把包含代码的字符串转换成 `TokenStream` 的方法。这种方法虽然可以通过使用模板很好地工作，但在构建代码的过程中，我们操作的数据结构已经失去了语义。



`quote` 让我们可以像编写正常的 Rust 代码一样，保留所有的语义，然后把它们转换成 `TokenStream`。它提供了一个`quote! 宏`，会替换代码中所有的` #(...)`，生成 `TokenStream`。比如写一个 `hello() `方法，如下

```rust
quote! {
    fn hello() {
        println!("Hello world!");
    }
}
```

`quote!` 做替换的方式和 `macro_rules!` 非常类似，也支持重复匹配。



## 3.3 用 syn/quote 重写 Builder 派生宏

大致思路就是先从 `TokenStream` 抽取需要的数据，再通过模板，把抽取出来的数据转换成目标代码（`TokenStream`）

> 完整代码参考[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/synquotederivemacros/src/builder.rs)

**1、创建项目`cargo new synquotederivemacros --lib`，`Cargo.toml` 添加依赖**

```toml
[package]
name = "synquotederivemacros"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[lib]
proc-macro = true

[dependencies]
anyhow = "1"
askama = "0.11" # 处理 jinjia 模板，模板需要放在和 src 平行的 templates 目录下
darling = "0.13" # 可以很方便的处理宏里面 attributes
proc-macro2 = "1" # proc-macro 的封装
quote = "1" # 用于生成代码的 TokenStream
syn = { version = "1", features = ["extra-traits"] } # 用于解析 TokenStream，使用 extra-traits 可以用于 Debug
```

* `syn crate` 默认所有数据结构都不带一些基本的 `trait`，比如 `Debug`，所以如果想打印数据结构的话，需要使用 `extra-traits feature`

* 由于 `syn/quote` 生成的 `TokenStream` 是 [proc-macro2](https://github.com/dtolnay/proc-macro2) 的类型，所以还需要使用 `proc-macro2` 库，它是对 `proc-macro` 的简单封装，使用起来更方便，而且可以让过程宏可以单元测试。



**2、先看看 DeriveInput 都输出什么**

在 `src/lib.rs` 中，先添加新的 `Builder` 派生宏：

```rust
use syn::{parse_macro_input, DeriveInput};
use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    // 通过 parse_macro_input!，得到了一个 DeriveInput 结构的数据
    let input = parse_macro_input!(input as DeriveInput);
    println!("{:#?}", input); // 打印 DeriveInput 结构的数据
    TokenStream::default()
}

fn main() {
    
}
```

在 `examples/command.rs` 中，先为 `Command` 引入 `Builder` 宏：

```rust
use synquotederivemacros::{Builder};

#[allow(dead_code)]
#[derive(Debug, Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: Option<String>,
}

```

然后运行` cargo run --example command`，就可以看到 `DeriveInput` 的输出：

* 对于 `struct name`，可以直接从 `ident` 中获取
* 对于 `fields`，需要从 `data` 内部的 ``DataStruct { fields }` 中取。目前，我们只关心每个 `field` 的 `ident` 和 `ty`。



**3、定义自己的用于处理 `derive` 宏的数据结构**

定义一个数据结构来获取构建 `TokenStream` 用到的信息，如下 `Fd` 和 `BuilderContext`：

```rust
struct Fd {
    name: Ident,
    ty: Type,
    optional: bool,
}

pub struct BuilderContext {
    name: Ident,
    fields: Vec<Fd>,
}
```



**4、把 DeriveInput 转换成自己的数据结构 BuilderContext**

写两个 `Fromtrait` 的实现，分别把 `Field` 转换成 `Fd`，把 `DeriveInput` 转换成 `BuilderContext`：

```rust
/// 把一个 Field 转换成 Fd
impl From<Field> for Fd {
    fn from(f: Field) -> Self {
        let (optional, ty) = get_option_inner(f.ty);
        Self {
            // 此时，我们拿到的是 NamedFields，所以 ident 必然存在
            name: f.ident.unwrap(),
            optional,
            ty,
        }
    }
}

/// 把 DeriveInput 转换成 BuilderContext
impl From<DeriveInput> for BuilderContext {
    fn from(input: DeriveInput) -> Self {
        let name = input.ident;

        let fields = if let Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) = input.data
        {
            named
        } else {
            panic!("Unsupported data type");
        };

        let fds = fields.into_iter().map(Fd::from).collect();
        Self { name, fields: fds }
    }
}

// 如果是 T = Option<Inner>，返回 (true, Inner)；否则返回 (false, T)
fn get_option_inner(ty: Type) -> (bool, Type) {
    todo!()
}
```

注意在从 `input` 中获取 `fields` 时，用了一个嵌套很深的模式匹配，如果没有强大的模式匹配的支持，获取 `FieldsNamed` 会是非常冗长的代码

```rust
if let Data::Struct(DataStruct {
    fields: Fields::Named(FieldsNamed { named, .. }),
    ..
}) = input.data
{
    named
}
```

在处理 `Option`类型时，我们用了一个还不存在的函数 `get_option_inner()`，这样一个函数是为了实现，如果是 `T = Option`，就返回` (true, Inner)`，否则返回` (false, T)`。



**5、使用 quote 生成代码**

写一个生成代码的 `render()` 方法：

```rust
impl BuilderContext {
    pub fn render(&self) -> TokenStream {
        let name = &self.name;
        // 生成 XXXBuilder 的 ident
        let builder_name = Ident::new(&format!("{}Builder", name), name.span());

        let optionized_fields = self.gen_optionized_fields();
        let methods = self.gen_methods();
        let assigns = self.gen_assigns();

        quote! {
            /// Builder 结构
            #[derive(Debug, Default)]
            struct #builder_name {
                #(#optionized_fields,)*
            }

            /// Builder 结构每个字段赋值的方法，以及 build() 方法
            impl #builder_name {
                #(#methods)*

                pub fn build(mut self) -> Result<#name, &'static str> {
                    Ok(#name {
                        #(#assigns,)*
                    })
                }
            }

            /// 为使用 Builder 的原结构提供 builder() 方法，生成 Builder 结构
            impl #name {
                fn builder() -> #builder_name {
                    Default::default()
                }
            }
        }
    }

    // 为 XXXBuilder 生成 Option<T> 字段
    // 比如：executable: String -> executable: Option<String>
    fn gen_optionized_fields(&self) -> Vec<TokenStream> {
        todo!();
    }

    // 为 XXXBuilder 生成处理函数
    // 比如：methods: fn executable(mut self, v: impl Into<String>) -> Self { self.executable = Some(v); self }
    fn gen_methods(&self) -> Vec<TokenStream> {
        todo!();
    }

    // 为 XXXBuilder 生成相应的赋值语句，把 XXXBuilder 每个字段赋值给 XXX 的字段
    // 比如：#field_name: self.#field_name.take().ok_or(" xxx need to be set!")
    fn gen_assigns(&self) -> Vec<TokenStream> {
        todo!();
    }
}
```

到目前为止，完整的从 `TokenStream` 到 `TokenStream` 转换的骨架已经完成，剩下的只是实现细节而已。



**6、完整实现**

创建 `src/builder.rs` 文件（记得在 `src/lib.rs` 里引入），然后写入代码

```rust
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, GenericArgument, Path, Type,
    TypePath,
};

/// 我们需要的描述一个字段的所有信息
struct Fd {
    name: Ident,
    ty: Type,
    optional: bool,
}

/// 我们需要的描述一个 struct 的所有信息
pub struct BuilderContext {
    name: Ident,
    fields: Vec<Fd>,
}

/// 把一个 Field 转换成 Fd
impl From<Field> for Fd {
    fn from(f: Field) -> Self {
        let (optional, ty) = get_option_inner(&f.ty);
        Self {
            // 此时，我们拿到的是 NamedFields，所以 ident 必然存在
            name: f.ident.unwrap(),
            optional,
            ty: ty.to_owned(),
        }
    }
}

/// 把 DeriveInput 转换成 BuilderContext
impl From<DeriveInput> for BuilderContext {
    fn from(input: DeriveInput) -> Self {
        let name = input.ident;

        let fields = if let Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) = input.data
        {
            named
        } else {
            panic!("Unsupported data type");
        };

        let fds = fields.into_iter().map(Fd::from).collect();
        Self { name, fields: fds }
    }
}

impl BuilderContext {
    pub fn render(&self) -> TokenStream {
        let name = &self.name;
        // 生成 XXXBuilder 的 ident
        let builder_name = Ident::new(&format!("{}Builder", name), name.span());

        let optionized_fields = self.gen_optionized_fields();
        let methods = self.gen_methods();
        let assigns = self.gen_assigns();

        quote! {
            /// Builder 结构
            #[derive(Debug, Default)]
            struct #builder_name {
                #(#optionized_fields,)*
            }

            /// Builder 结构每个字段赋值的方法，以及 build() 方法
            impl #builder_name {
                #(#methods)*

                pub fn build(mut self) -> Result<#name, &'static str> {
                    Ok(#name {
                        #(#assigns,)*
                    })
                }
            }

            /// 为使用 Builder 的原结构提供 builder() 方法，生成 Builder 结构
            impl #name {
                fn builder() -> #builder_name {
                    Default::default()
                }
            }
        }
    }

    // 为 XXXBuilder 生成 Option<T> 字段
    // 比如：executable: String -> executable: Option<String>
    fn gen_optionized_fields(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|Fd { name, ty, .. }| quote! { #name: std::option::Option<#ty> })
            .collect()
    }

    // 为 XXXBuilder 生成处理函数
    // 比如：methods: fn executable(mut self, v: impl Into<String>) -> Self { self.executable = Some(v); self }
    fn gen_methods(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|Fd { name, ty, .. }| {
                quote! {
                    pub fn #name(mut self, v: impl Into<#ty>) -> Self {
                        self.#name = Some(v.into());
                        self
                    }
                }
            })
            .collect()
    }

    // 为 XXXBuilder 生成相应的赋值语句，把 XXXBuilder 每个字段赋值给 XXX 的字段
    // 比如：#field_name: self.#field_name.take().ok_or(" xxx need to be set!")
    fn gen_assigns(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|Fd { name, optional, .. }| {
                if *optional {
                    return quote! {
                        #name: self.#name.take()
                    };
                }

                quote! {
                    #name: self.#name.take().ok_or(concat!(stringify!(#name), " needs to be set!"))?
                }
            })
            .collect()
    }
}

// 如果是 T = Option<Inner>，返回 (true, Inner)；否则返回 (false, T)
fn get_option_inner(ty: &Type) -> (bool, &Type) {
    // 首先模式匹配出 segments
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(v) = segments.iter().next() {
            if v.ident == "Option" {
                // 如果 PathSegment 第一个是 Option，那么它内部应该是 AngleBracketed，比如 <T>
                // 获取其第一个值，如果是 GenericArgument::Type，则返回
                let t = match &v.arguments {
                    syn::PathArguments::AngleBracketed(a) => match a.args.iter().next() {
                        Some(GenericArgument::Type(t)) => t,
                        _ => panic!("Not sure what to do with other GenericArgument"),
                    },
                    _ => panic!("Not sure what to do with other PathArguments"),
                };
                return (true, t);
            }
        }
    }
    return (false, ty);
}
```

接着更新 `src/lib.rs` 里定义的 `derive_builde`r，直接从 `DeriveInput` 中生成一个 `BuilderContext`，然后 `render()`。

> 注意 `quote` 得到的是 `proc_macro2::TokenStream`，所以需要调用一下 `into() `转换成 `proc_macro::TokenStream`

```rust
#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    builder::BuilderContext::from(input).render().into()
}
```

然后在 `examples/command.rs` 中，更新 `Command` 的 `derive` 宏：

```rust
use macros::Builder;

#[allow(dead_code)]
#[derive(Debug, Builder)]
pub struct Command {
    ...
}
```

最后运行 `cargo run --example command` 后可以得到结果



## 3.4 支持 attributes 的派生宏

很多时候，派生宏可能还需要一些额外的 `attributes` 来提供更多信息，更好地指导代码的生成。

> 比如 `serde`，可以在数据结构中加入 `#[serde(xxx)] attributes`，控制 `serde` 序列化 / 反序列化的行为



在 `proc-macro-workshop` 里 [Builder 宏的第 7 个练习](https://github.com/dtolnay/proc-macro-workshop/blob/master/builder/tests/07-repeated-field.rs)中，有如下需求：

```rust
#[derive(Builder)]
pub struct Command {
    executable: String,
    #[builder(each = "arg")]
    args: Vec<String>,
    #[builder(each = "env")]
    env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .arg("build".to_owned())
        .arg("--release".to_owned())
        .build()
        .unwrap();

    assert_eq!(command.executable, "cargo");
    assert_eq!(command.args, vec!["build", "--release"]);
}
```

这里，如果字段定义了 `builder attributes`，并且提供了 `each` 参数，那么用户不断调用 `arg` 来依次添加参数，这样使用起来就直观多了。



**思路**

想要支持这样的功能，首先要能够解析 `attributes`，然后要能够根据 `each attribute` 的内容生成对应的代码，比如

```rust
pub fn arg(mut self, v: String) -> Self {
    let mut data = self.args.take().unwrap_or_default();
    data.push(v);
    self.args = Some(data);
    self
}
```

`syn` 提供的 `DeriveInput` 并没有对 `attributes` 额外处理，所有的 `attributes` 被包裹在一个 `TokenTree::Group` 中。可以使用 [darling](https://github.com/teddriggs/darling)库，来为 `Builder` 宏添加对 `attributes` 的支持。



**实现**

> 完整代码可参考[这里](https://github.com/sinkhaha/rust-study-demo/blob/main/synquotederivemacros/src/builder_with_attr.rs)

1、我们还是使用刚才的项目，在 `Cargo.toml` 中，加入对 `darling` 的引用：

```toml
[dependencies]
darling = "0.13"
```

2、在 `src/lib.rs` 中，再创建一个 `BuilderWithAttrs` 的派生宏：

```rust
#[proc_macro_derive(BuilderWithAttr, attributes(builder))]
pub fn derive_builder_with_attr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    builder_with_attr::BuilderContext::from(input)
        .render()
        .into()
}
```

和之前不同的是，这里多了一个 `attributes(builder) ` 属性，这是告诉编译器，请允许代码中出现的

` #[builder(...)]`，它是我这个宏认识并要处理的。



3、再创建一个 `examples/command_with_attr.rs`

```rust
use synquotederivemacros::BuilderWithAttr;

#[allow(dead_code)]
#[derive(Debug, BuilderWithAttr)]
pub struct Command {
    executable: String,
    #[builder(each = "arg")]
    args: Vec<String>,
    #[builder(each = "env", default="vec![]")]
    env: Vec<String>,
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .arg("build".to_owned())
        .arg("--release".to_owned())
        .build()
        .unwrap();

    assert_eq!(command.executable, "cargo");
    assert_eq!(command.args, vec!["build", "--release"]);
    println!("{:?}", command);
}
```

这里不仅希望支持 `each` 属性，还支持 `default` （如果用户没有为这个域提供数据，就使用 `default `对应的代码来初始化）



4、然后，在 `src/builder_with_attr.rs` 中，添加用于捕获 `attributes` 的数据结构：

```rust
use darling::FromField;

#[derive(Debug, Default, FromField)]
#[darling(default, attributes(builder))]
struct Opts {
    each: Option<String>,
    default: Option<String>,
}
```

因为捕获的是 `field` 级别的 `attributes`，所以这个数据结构需要实现 [FromField trait](https://docs.rs/darling/latest/darling/trait.FromField.html)（通过 `FromTrait` 派生宏），并且告诉 `darling` 要从哪个 `attributes` 中捕获（这里是从 `builder` 中捕获）。不过先需要修改一下 `Fd`，让它包括 `Opts`，并且在 `From` 的实现中初始化 `opts`：

```rust

/// 我们需要的描述一个字段的所有信息
struct Fd {
    name: Ident,
    ty: Type,
    optional: bool,
    opts: Opts,
}

/// 把一个 Field 转换成 Fd
impl From<Field> for Fd {
    fn from(f: Field) -> Self {
        let (optional, ty) = get_option_inner(&f.ty);
        // 从 Field 中读取 attributes 生成 Opts，如果没有使用缺省值
        let opts = Opts::from_field(&f).unwrap_or_default();
        Self {
            opts,
            // 此时，我们拿到的是 NamedFields，所以 ident 必然存在
            name: f.ident.unwrap(),
            optional,
            ty: ty.to_owned(),
        }
    }
}
```

现在 `Fd` 就包含 `Opts` 的信息了，可以利用这个信息来生成 `methods` 和 `assigns`。



接下来先看 `gen_methods` 怎么修改。如果 `Fd` 定义了 `each attribute`，且它是个 `Vec` 的话，我们就生成不一样的代码，否则的话，像之前那样生成代码：

```rust
// 为 XXXBuilder 生成处理函数
// 比如：methods: fn executable(mut self, v: impl Into<String>) -> Self { self.executable = Some(v); self }
fn gen_methods(&self) -> Vec<TokenStream> {
    self.fields
        .iter()
        .map(|f| {
            let name = &f.name;
            let ty = &f.ty;
            // 如果不是 Option 类型，且定义了 each attribute
            if !f.optional && f.opts.each.is_some() {
                let each = Ident::new(f.opts.each.as_deref().unwrap(), name.span());
                let (is_vec, ty) = get_vec_inner(ty);
                if is_vec {
                    return quote! {
                        pub fn #each(mut self, v: impl Into<#ty>) -> Self {
                            let mut data = self.#name.take().unwrap_or_default();
                            data.push(v.into());
                            self.#name = Some(data);
                            self
                        }
                    };
                }
            }
            quote! {
                pub fn #name(mut self, v: impl Into<#ty>) -> Self {
                    self.#name = Some(v.into());
                    self
                }
            }
        })
        .collect()
}
```

这里重构了一下 `get_option_inner()` 的代码，因为 `get_vec_inner() ` 和它有相同的逻辑

```rust
// 如果是 T = Option<Inner>，返回 (true, Inner)；否则返回 (false, T)
fn get_option_inner(ty: &Type) -> (bool, &Type) {
    get_type_inner(ty, "Option")
}

// 如果是 T = Vec<Inner>，返回 (true, Inner)；否则返回 (false, T)
fn get_vec_inner(ty: &Type) -> (bool, &Type) {
    get_type_inner(ty, "Vec")
}

fn get_type_inner<'a>(ty: &'a Type, name: &str) -> (bool, &'a Type) {
    // 首先模式匹配出 segments
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(v) = segments.iter().next() {
            if v.ident == name {
                // 如果 PathSegment 第一个是 Option/Vec 等类型，那么它内部应该是 AngleBracketed，比如 <T>
                // 获取其第一个值，如果是 GenericArgument::Type，则返回
                let t = match &v.arguments {
                    syn::PathArguments::AngleBracketed(a) => match a.args.iter().next() {
                        Some(GenericArgument::Type(t)) => t,
                        _ => panic!("Not sure what to do with other GenericArgument"),
                    },
                    _ => panic!("Not sure what to do with other PathArguments"),
                };
                return (true, t);
            }
        }
    }
    return (false, ty);
}
```

最后为 `gen_assigns()` 提供对 `default attribute` 的支持：

```rust
fn gen_assigns(&self) -> Vec<TokenStream> {
    self.fields
        .iter()
        .map(|Fd { name, optional, opts, .. }| {
            if *optional {
                return quote! {
                    #name: self.#name.take()
                };
            }

            // 如果定义了 default，那么把 default 里的字符串转换成 TokenStream
            // 使用 unwrap_or_else 在没有值的时候，使用缺省的结果
            if let Some(default) = opts.default.as_ref() {
                let ast: TokenStream = default.parse().unwrap();
                return quote! {
                    #name: self.#name.take().unwrap_or_else(|| #ast)
                };
            }

            quote! {
                #name: self.#name.take().ok_or(concat!(stringify!(#name), " needs to be set!"))?
            }
        })
        .collect()
}
```

5、最后运行 `cargo run --example command_with_attr  ` 就会得到结果



# 4 参考

* [陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/481355)
* [Rust程序设计（第2版）](https://book.douban.com/subject/36547630/)
* [Macro宏编程](https://course.rs/advance/macro.html)

