# 1 所有权

`Rust` 中，每个值都能被一个变量所拥有，这个变量被称为值的 **所有者 或 拥有者**，即变量拥有值。所有权是为了限制堆上数据的多重引用，它决定了谁真正拥有数据，并控制着数据的生命周期。



## 1.1 所有权的规则

1. 一般情况下，一个值同一时刻只能有一个所有者，即不能有两个变量同时拥有同一个值

   > 例外：标准库提供了引用计数指针类型 `Rc` 和 `Arc`，它们允许值在某些限制下有多个所有者

2. 例如变量赋值、参数传递、函数返回等行为，一般旧的所有者会把值的所有权转移给新的所有者，以便保证单一所有者的约束

   > 例外：变量的类型如果实现了 `Copy` 特型，则是复制，不会转移所有权

3. 当变量离开作用域，变量会被丢弃，这个变量拥有的值也会被销毁，内存得到释放（也叫丢弃）

   > 丢弃的过程实际上内部是调用 `Drop`  特型种一个名为 `drop` 的函数来销毁数据释放内存，类似析构函数



## 1.2 所有者的判断

**例1:**

```rust
fn main() {
    let s = String::from("hello");
  
    println!("String capacity: {} len:{}", s.capacity(), s.len()); // String capacity: 5 len:5
}
```

由代码可知，变量 `s` 是栈中 **胖指针** 的所有者，而不是 **堆中实际数据 hello** 的所有者。分析如下

* 变量 `s` 是一个 `String` 类型，`String` 类型有一个可调整大小的缓冲区，缓冲区是在堆分配的，所以 `String` 类型的实际数据是存储在堆中，且它的大小是不确定的，可以将 `String` 视为 `Vec<u8>`

* 在栈中是使用了一个胖指针结构来表示这个 `String` 类型的数据，这个`胖指针中的指针`指向堆中的 `String` 实际数据。也就是，**变量 s 的值是那个胖指针，而不是堆中的实际数据**

  > 但是，由于胖指针是指向堆中数据的，多数时候为了简化理解和描述方式，也经常会说s是那个堆中实际数据的所有者



在内存中，`s` 的值如图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/rust-chengxusheji/%E6%89%80%E6%9C%89%E6%9D%83-%E5%80%9F%E7%94%A8-%E5%BC%95%E7%94%A8.drawio%20(1).png)



在 `main()` 函数的栈桢中，保存了 `s` 指针、容量（5）、长度（5），只有向量的缓冲区才分配在堆上，变量 `s` 拥有保存其元素的缓冲区，当变量 `s` 在函数末尾超出作用域时，程序会丢弃变量是，此时 `s` 拥有的缓冲区也会一起被丢弃。



**例2：**

```rust
fn main() {
    // 结构体
    struct Person {
        name: String,
        birth: i32,
    }

    let mut list = Vec::new(); // list是一个 Vec<Person>，即由结构体组成的向量

    list.push(Person {
        name: "zhangsan".to_string(),
        birth: 1990,
    });

    list.push(Person {
        name: "lisi".to_string(),
        birth: 1995,
    });

    // zhangsan born 1990
    // lisi born 1995
    for person in &list {
        println!("{} born {}", person.name, person.birth);
    }
}
```

由代码可知，`list` 拥有一个向量，向量拥有自己的元素，每个元素都是一个 `Person` 结构体，每个结构体拥有自己的字段，并且字符串字段拥有自己的文本。由此可见，所有者和拥有的那些值形成了一棵树，可以称为所有权树。当控制流离开 `list` 的作用域时，程序会丢弃 `list` 的值并将整棵所有权树一起丢掉。在内存中，`list` 的值如图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/rust-chengxusheji/%E6%89%80%E6%9C%89%E6%9D%83list%E4%BE%8B%E5%AD%90.drawio.png)



## 1.3 所有权的转移（move）

`Rust` 可以将值从一个所有者转移到另一个所有者，例如变量赋值、参数传递、函数返回等行为都会发生所有权的移动。



**例1：**

```rust
fn main() {
    let data = vec![1, 2, 3, 4];
    let data1 = data; // data的所有权转移给了data1，在这之后就不能访问data变量了，此时data变成了未初始化状态

    println!("sum of data1: {}", sum(data1)); // 调用方法传参时，data1的所有权转移给了sum函数的data变量
    println!("sum of data: {:?}", data); // 报错，因为data的所有权转移给了data1，不能再访问data
    println!("data1: {:?}", data1); // 报错，因为data1的所有权转移到了sum函数的data变量了，不能再访问data1
}

fn sum(data: Vec<u32>) -> u32 {
    data.iter().fold(0, |acc, x| acc + x)
}
```

由代码可知

* `Vector` 类型 有一个缓冲区，缓冲区是堆上的，实际数据是存在堆上的。`data` 赋值给 `data1` 时，`data` 的所有权转移给了 `data1`，在这之后就不能访问 `data` 变量了，此时 `data` 变成了未初始化状态
* 调用 `sum(data1)` 时， `data1` 的所有权转移给了 `sum` 函数的 `data` 变量



所有权的转移如下图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%89%80%E6%9C%89%E6%9D%83%E8%BD%AC%E7%A7%BB-1.png)





**例2：**再看一个所有权和转移的例子

```rust
struct Person {
    name: String,
    birth: i32,
}

let mut list = Vec::new(); 

list.push(Person {
    name: "lisi".to_string(),
    birth: 1990,
});
```

有以下几个地方都发生了移动

1. 从函数返回值

调用 `Vec::new()` 构造一个新向量并返回，返回的不是指向此向量的指针，而是向量本身：向量的所有权从 `Vec::new` 转移给了 `list`。同样，`to_string()` 调用返回的是一个新的 `String` 实例

2. 构造出新值

新 `Person` 结构体的 `name` 字段是用 `to_string()` 的返回值初始化的，该结构体拥有这个字符串的所有权

3. 将值传给函数

整个 `Person` 结构体（不是指向它的指针）被传给了向量的 `push` 方法，此方法会将结构体移动到向量的末尾。向量接管了 `Person` 的所有权，因此也间接接管了 `name` 这个 `String` 的所有权



**要记住移动的永远是值本身，而不是这些值拥有的堆存储**。对于向量和字符串，值本身就是指单独的“三字标头”（指针、容量、长度），幕后的大型元素数组和文本缓冲区仍然位于它们在堆中的位置。



**例3：**

`Rust` 中的 `Box` 类型是所有权的另一个例子。`Box<T> `是指向存储在堆上的 `T` 类型值的指针。 调用 `Box::new(v)` 分配一些堆空间，将值 `v` 移入其中，并返回一个指向该堆空间的 `Box`，因为 `Box` 拥有它所指向的空间，所以当丢弃 `Box` 时，也会释放此空间，如

```rust
{
    let point = Box::new((0.625, 0.5)); // 在栈分配了point，会在堆上为由两个f64值构成的元组分配空间，然后将其参数(0.625, 0.5)移进去，并返回指向该空间的指针
  
    let label: String = format!("{:?}", point); // 在此分配了label

    assert_eq!(label, "(0.625, 0.5)"); 
} // 在此全都被丢弃了

```

如图

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/rust-chengxusheji/Box%E6%89%80%E6%9C%89%E6%9D%83%E7%A7%BB%E5%8A%A8.drawio.png)





**所有权移动时要注意的地方**

1、如果将一个值转移给一个已经初始化的变量，那么先前的值会被丢弃，如

```rust
let mut s = "hello".to_string();
s = "world".to_string(); // 在这里丢弃了值 "hello"，因为s不需要拥有它了
```

```rust
let mut s = "hello".to_string();
let t = s; // 这里 hello的所有权转移 从s给了t，s会回到未初始化状态
s = "world".to_string(); // 这里s又拥有了world的所有权
```



2、禁止在循环中进行变量移动，如

```rust
let x = vec![10, 20, 30];
if c {
    f(x); // x被移动了
} else {
    g(x); // x被移动了
}

h(x); // 错误：因为前面的if else 一定会导致x被移动，这里的x已经是未初始化状态了
```

```rust
let x = vec![10, 20, 30];
while f() {
    g(x);  // 错误：因为第一次迭代x已经被移动了，当进行第二次迭代时，这里的x已经是未初始化状态了
}
```

应该改成

```rust
let x = vec![10, 20, 30];
while f() {
    g(x);  
    x = h(); // x在上面被移动了变成未初始化状态，但是这里x又被重新赋值，所有这个while循环是正确的
}
```



3、移动向量内容时，移动会令其来源会变成未初始化状态，目标会获得该值的所有权。但是并非值的每种拥有者都能变成未初始化状态，例如

```rust
// 一个由"101"到"105"组成的向量
let mut v = Vec::new();
for i in 101..106 {
    v.push(i.to_string());
}

let third = v[2]; // 错误，不能移动到Vec索引结构之外
let fifth = v[4]; // 这里也一样错误
```

解决方式：可以使用引用，而不是移动它，因为这里只是想访问该元素而已，如

```rust
// 一个由"101"到"105"组成的向量
let mut v = Vec::new();
for i in 101..106 {
    v.push(i.to_string());
}

// 方式1: 从向量的末尾弹出一个值
let fifth = v.pop().expect("vector empty!");
assert_eq!(fifth, "105");

// 方式2:将向量中指定索引处的值与最后一个值互换，并把前者移动出来
let second = v.swap_remove(1);
assert_eq!(second, "102");

// 方式3: 把要取出的值和另一个值互换
let third = std::mem::replace(&mut v[2], "substitute".to_string());
assert_eq!(third, "103");

assert_eq!(v, vec!["101", "104", "substitute"]);
```

上面每种方法都能将一个元素移出向量，但仍会让向量处于完全填充状态，只是向量可能会变小。



4、`for` 循环会获取元素的所有权，如

```rust
let v = vec![
    "hello".to_string(),
    "world".to_string(),
    "zhangsan".to_string(),
];

for mut s in v {
    s.push('!'); // s拥有每次循环的字符串，所以这里可以修改它
    println!("{}", s);
}

println!("{:?}", v); // 报错
```

当将向量直接传给循环（如 `for ... in v`）时，会将向量从`v` 中移动出去，让  `v` 变成未初始化状态。`for` 循环的内部机制会获取向量的所有权并将其分解为元素。在每次迭代中，循环会将另一个元素转移给变量 `s`。



# 2 所有权不会被转移的情况

以下 3 种方式，值的所有权不会发生转移

1. `clone`：即克隆数据（即深拷贝）

2. `copy`：如果一个数据结构实现了 `Copy` 特型，那么它就会使用 `Copy` 语义，而不是 `Move` 语义。此时赋值或者传参时，值会自动按位拷贝（即浅拷贝）

   > 实现了 `Copy` 特型的类型不会转移所有权，比如标准库中的整数、浮点数、字符这些简单类型，不受所有权转移的约束，它们会直接在栈中复制一份副本

3. `borrowing`：即 “借用” 数据，可以对值进行 “借用”，以获得值的引用



## 2.1 Clone Trait

> `trait` 称为特型，可以理解成接口

只有实现了 `Clone` 特型的类型才可以进行克隆，调用 `clone()` 方法可以拷贝变量的数据，克隆了一个副本，克隆是深拷贝，这样就不会使得原始变量的所有权发生转移，而导致原始变量变成未初始化状态。如

```rust
fn main() {
    let data = vec![1, 2, 3, 4];
  
    let data1 = data.clone(); // 克隆data，克隆之后，变量data仍然绑定原始数据，data的所有权不会转移到data1，此时堆会拷贝一份新的[1,2,3,4]数据，然后data1指向新的这份堆数据

    // 下面正常打印data
    println!("sum of data1: {}", sum(data1));
    println!("sum of data: {:?}", data);
}

fn sum(data: Vec<u32>) -> u32 {
    data.iter().fold(0, |acc, x| acc + x)
}
```

如图，堆上的数据也会复制一份



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/clone-1.png)



### 2.1.1 实现 Clone Trait

自定义类型时，在类型上面加上 `#[derive(Clone)]` 属性即可实现 `Clone` 特型，这样该类型有拥有了克隆的能力。如下 `Test` 类型的值就可以使用 `clone() `方法进行克隆

```rust
#[derive(Clone)] // 实现了Clone特型
struct Test {
  age: i32
};
```



## 2.2 Copy Trait

如果值对应的类型实现了 `Copy` 特型，当有赋值、传参等场景需要"移动"这个值时，值就会自动**按位拷贝（浅拷贝）**，而不是发生所有权的转移，如

```rust
fn main() {
    let a: i32 = 11;
    let b = a; // 这里是复制一份a，而不是所有权转移，i32类型实现了Copy特型

    println!("a={} b={}", a, b); // 这里还可以打印a，不会报错
}
```



如下面的错误例子，`data` 的类型 `Vec<i32>`，它没有实现 `Copy trait`，在赋值或者函数调用时无法 `Copy` 复制值，于是就按默认会进行所有权转移。在所有权转移之后，原先的变量 `data` 变成了未定义状态，无法访问

```rust
fn main() {
    let data = vec![1, 2, 3, 4];
    let data1 = data; // data的所有权转移给了data1，在这之后就不能访问data变量了

    println!("sum of data1: {}", sum(data1)); // data1的所有权转移给了sum函数的data变量
    println!("sum of data: {:?}", data); // 报错，因为data的所有权转移给了data1，不能再访问data
    println!("data1: {:?}", data1); // 报错，因为data1的所有权转移到了sum函数的data变量了，不能再访问data1
}

fn sum(data: Vec<u32>) -> u32 {
    data.iter().fold(0, |acc, x| acc + x)
}
```



### 2.2.1 实现 Copy Trait

如果自定义的类型（比如结构体）的所有字段本身都是 `Copy` 类型，则可以在类型上方加上 `#[derive(Copy)]`，即可为该类型实现 `Copy` 特型。如

```rust
#[derive(Copy)]
struct Test {
  age: i32
}
```

**注意**：实现了 `Copy` 的类型也要求实现 `Clone`，因为 `Copy` 特型是 `Clone` 特型的子特型



### 2.2.2 判断是否是 Copy 类型

1. 原生类型，如所有的机器整数类型、浮点数类型、char 类型、布尔类型，包括函数、不可变引用和裸指针实现了 `Copy`
2. 固定大小的数组和元组：如果其内部的数据结构实现了 `Copy`，那么它们也能通过  ``#[derive(Copy, Clone)]``实现 `Copy`
3. 默认情况下，`struct` 类型和 enum 类型不是 `Copy` 类型 
4. 如果用户自定义的结构体的所有字段本身都是 `Copy` 类型，那么可以通过将属性 `#[derive(Copy, Clone)]` 放置到此结构体上，此结构体就变成 `Copy` 类型
5. 任何实现了 `Drop` 特型的类型都不能是 `Copy` 类型
6. 任何在丢弃值时需要做一些特殊操作的类型都不能是 `Copy` 类型，比如
   * 可变引用没有实现 `Copy`
   * 非固定大小的数据结构，没有实现 `Copy`，如向量 `Vec` 需要释放自身元素，`String` 需要释放缓冲区
   * `File` 需要关闭自身文件句柄
   * `Box<T>` 拥有从堆中分配的引用目标
   * `MutexGuard` 需要解锁自身互斥锁



### 2.2.3 Clone 和 Copy 的区别

这里不考虑自己实现 `Copy` 特型和 `Clone` 特型自己实现了 `clone()` 方法的情况，都默认是 `Rust` 的实现

- `Copy` 时：只拷贝变量本身的值，如果这个变量指向了其它数据，则不会拷贝其指向的数据，即浅拷贝
- `Clone` 时：拷贝变量本身的值，如果这个变量指向了其它数据，则也会拷贝其指向的数据，即深拷贝，`Rust` 会对每个字段每个元素递归调用 `clone() `，直到最底部。

所以，使用 `Clone` 的默认实现时，`clone()` 操作的性能是较低的。但可以自己实现自己的克隆逻辑，也不一定总是会效率低。比如 `Rc` 类型，它的 `clone` 用于增加引用计数，同时只拷贝少量数据，它的 `clone` 效率并不低。



## 2.3 借用（Borrowing）

### 2.3.1 什么是借用

借用：可以对值进行借用，以获得值的引用，表示方式是加一个 `&符号`在值前面。如  `let b = &a; `，此时 `&a` 表示借用 `a`，这是一个借用动作，它的结果是得到一个引用类型，所以 `b` 是一个引用类型的变量（此处可以把 `&` 理解为 `C++` 的指针）

> 关于引用的知识可参考另一章节



1. 对一个值进行借用，值不会发生所有权的转移
2. 默认情况下，`Rust` 的借用都是只读的，一个值可以有多个只读引用，这种引用是非拥有型指针（即不是值的拥有者），有着受限的生命周期
3. 只读引用实现了 `Copy trait`，也就意味着引用的赋值、传参都会产生新的浅拷贝



例1：

```rust
fn main() {
    let a: i32 = 666;
    
    // 借用
    let b = &a; // 含义：a绑定的资源A借给b使用，b只有资源A的读权限，此时b是一个引用类型，&i32
    println!("a: {} b: {}", a, b); // a: 666 b: 666
    
    let c = b;
    // std::ptr::eq()来判断两个引用是否指向同一个地址，即判断所指向的数据是否是同一份数据
    println!("{}", std::ptr::eq(b, c)); // true 

    // 解指针，此时的 “*表达式” 类似C++的解指针，即拿到b存的地址指向的值
    println!("{}", *b); // 666
}
```

例2：

```rust
fn main() {
    let a = 1; // i32类型
    let b = 2;
    
    // &a和&b都是借用
    let rst = sum(&a, &b);
    println!("{} + {} = {}", a, b, rst); // 1 + 2 = 3
}

// &i32表示a是一个i32的引用类型
fn sum(a: &i32, b: &i32) -> i32 {
    a + b
}
```



### 2.3.2 借用的约束

对值的借用的约束：借用的生命周期不能超过值的生命周期

看一个正确的例子：

```rust
fn main() {
    let data = vec![1, 2, 3, 4];
    // data的生命周期是main函数结束，sum函数处于main的下一层调用栈中，所以sum调用结束后main函数还会继续执行，所以在 main() 函数中定义的 data 生命周期要比 sum() 中对 data 的引用要长，这样不会有任何问题
    println!("sum of data1: {}", sum(&data));
}

fn sum(data: &Vec<u32>) -> u32 {
    data.iter().fold(0, |acc, x| acc + x)
}
```

再看一个错误的例子：

```rust
// 错误用法，编译不通过
fn main() {
    // 生命周期更长的 main() 函数变量 r ，引用了生命周期更短的 local_ref() 函数里的局部变量a
    let r = local_ref();
    println!("r: {:p}", r);
}

fn local_ref<'a>() -> &'a i32 {
    let a = 42;
    &a // 报错，因为这里返回a的引用，a是局部变量，生命周期比调用方短
}
```



# 3 参考 

* [The Rust Programming Language](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
* [陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/415988)
* [Rust入门秘籍](https://rust-book.junmajinlong.com/ch3/07_reference_type.html)
* [Rust语言圣经(Rust Course)](https://course.rs/basic/ownership/borrowing.html)
* [理解 Rust 引用和借用](https://zhuanlan.zhihu.com/p/59998584)
* [Rust程序设计（第2版）](https://book.douban.com/subject/36547630/)
* [ Copy trait](https://doc.rust-lang.org/std/marker/trait.Copy.html)

