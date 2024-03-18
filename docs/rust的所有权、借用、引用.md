# 所有权

所有权是为了限制堆上数据的多重引用，决定了谁真正拥有数据，并控制着数据的生命周期。



## 所有权的规则

1. 一个值只能被一个变量所拥有，这个变量被称为值的`所有者 或 拥有者`，即变量拥有值

2. 一般情况下，一个值同一时刻只能有一个所有者，即不能有两个变量同时拥有同一个值

> * 所以变量赋值、参数传递、函数返回等行为，默认旧的所有者会把值的所有权转移给新的所有者，以便保证单一所有者的约束
> * 例外情况：标准库提供了引用计数指针类型Rc和Arc，它们允许值在某些限制下有多个所有者

3. 当变量离开作用域，变量会被丢弃，这个变量拥有的值也会被销毁，内存得到释放（也叫丢弃）

> 丢弃的过程实际上内部是调用Drop特型种一个名为drop的函数来销毁数据释放内存，类似析构函数



## 所有者的判断

**例1:**

```rust
fn main() {
    let s = String::from("hello");
  
    println!("String capacity: {} len:{}", s.capacity(), s.len()); // String capacity: 5 len:5
}
```

代码中，`变量s` 是栈中`胖指针` 的所有者，而不是`堆中实际数据 hello `的所有者。分析如下

* 变量s是一个String类型，String类型有一个可调整大小的缓冲区，缓冲区是在堆分配的，所以String类型的实际数据是存储在堆中，且它的大小是不确定的，可以将 String 视为 `Vec<u8>`
* 在栈中是使用了一个胖指针结构来表示这个String类型的数据，这个`胖指针中的指针`指向堆中的String实际数据。也就是，**变量s的值是那个胖指针，而不是堆中的实际数据**

> 但是，由于胖指针是指向堆中数据的，多数时候为了简化理解和描述方式，也经常会说s是那个堆中实际数据的所有者



在内存中，s的值如图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/rust-chengxusheji/%E6%89%80%E6%9C%89%E6%9D%83-%E5%80%9F%E7%94%A8-%E5%BC%95%E7%94%A8.drawio%20(1).png)



在main()函数的栈桢中，保存了 s指针、容量(5)、长度(5)，只有向量的缓冲区才分配在堆上，变量s拥有保存其元素的缓冲区，当变量s在函数末尾超出作用域时，程序会丢弃变量是，此时s拥有的缓冲区也会一起被丢弃。



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

由代码可知，list拥有一个向量，向量拥有自己的元素，每个元素都是一个Person结构体，每个结构体拥有自己的字段，并且字符串字段拥有自己的文本。由此可见，所有者和拥有的那些值形成了一棵树。当控制流离开list的作用域时，程序会丢弃list的值并将整棵所有权树一起丢掉。



在内存中，list的值如图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/rust-chengxusheji/%E6%89%80%E6%9C%89%E6%9D%83list%E4%BE%8B%E5%AD%90.drawio.png)



## 所有权的转移(move)

Rust 可以将值从一个所有者转移到另一个所有者，例如变量赋值、参数传递、函数返回等行为都会发生所有权的移动。如

```rust
// 错误用法
fn main() {
    let data = vec![1, 2, 3, 4];
    let data1 = data; // data的所有权转移给了data1，在这之后就不能访问data变量了，此时data变成了未初始化状态

    println!("sum of data1: {}", sum(data1)); // data1的所有权转移给了sum函数的data变量
    println!("sum of data: {:?}", data); // 报错，因为data的所有权转移给了data1，不能再访问data
    println!("data1: {:?}", data1); // 报错，因为data1的所有权转移到了sum函数的data变量了，不能再访问data1
}

fn sum(data: Vec<u32>) -> u32 {
    data.iter().fold(0, |acc, x| acc + x)
}
```

由代码可知

* data的所有权转移给了data1，在这之后就不能访问data变量了，此时data变成了未初始化状态

* data1的所有权转移给了sum函数的data变量



所有权的转移如下图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E6%89%80%E6%9C%89%E6%9D%83%E8%BD%AC%E7%A7%BB-1.png)





**更多移动的操作**

如果将一个值转移给一个已经初始化的变量，那么先前的值会被丢弃，如

```rust
let mut s = "hello".to_string();
s = "world".to_string(); // 在这里丢弃了值 "hello"，因为s不需要拥有它了
```

再看如下代码

```rust
let mut s = "hello".to_string();
let t = s; // 这里 hello的所有权转移 从s给了t，s会回到未初始化状态
s = "world".to_string(); // 这里s又拥有了world的所有权
```

再看一个例子，分析一下所有权和转移

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

1、从函数返回值

调用 Vec::new() 构造一个新向量并返回，返回的不是指向此向量的指针，而是向量本身：向量的所有权从 Vec::new 转移给了list。同样，to_string()调用返回的是一个新的String实例

2、构造出新值

新Person结构体的name字段是用 to_string() 的返回值初始化的。该结构体拥有这个字符串的所有权。

3、将值传给函数

整个Person结构体（不是指向它的指针）被传给了向量的push方法，此方法会将结构体移动到向量的末尾。向量接管了Person的所有权，因此也间接接管了name这个String的所有权



要牢记移动的永远是**值本身**，而不是这些值拥有的堆存储：对于向量和字符串，值本身 就是指单独的“三字标头”，幕后的大型元素数组和文本缓冲区仍然位于它们在堆中的位置



Rust 中的Box类型是所有权的另一个例子。`Box<T>`是指向存储在堆上的T类型值的指针。 调用 Box::new(v) 分配一些堆空间，将值v移入其中，并返回一个指向该堆空间的Box，因为Box拥有它所指向的空间，所以当丢弃Box时，也会释放此空间，如

```rust
{
    let point = Box::new((0.625, 0.5)); // 在栈分配了point，会在堆上为由两个f64值构成的元组分配空间，然后将其参数(0.625, 0.5)移进去，并返回指向该空间的指针
  
    let label: String = format!("{:?}", point); // 在此分配了label

    assert_eq!(label, "(0.625, 0.5)"); 
} // 在此全都被丢弃了

```



**一些错误的移动实践**

禁止在循环中进行变量移动，如

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

正确的例子

```rust
let x = vec![10, 20, 30];
while f() {
    g(x);  
    x = h(); // x在上面被移动了变成未初始化状态，但是这里x又被重新赋值，所有这个while循环是正确的
}
```

移动与索引内容，移动时源会把值的所有权转移给目标，源会变回未初始化状态。但是并非指的每种拥有者都能变成未初始化状态，例如

```rust
// 一个由"101"到"105"组成的向量
let mut v = Vec::new();
for i in 101..106 {
    v.push(i.to_string());
}

let third = v[2]; // 错误，不能移动到Vec索引结构之外
let fifth = v[4]; // 这里也一样错误
```

如果以上问题要被解决，rust需要记住向量的第3个元素和第5个元素是未初始化状态，并跟踪该信息直到向量被丢弃，这样需要每个向量都携带额外的信息来指示哪些元素是活动的，哪些元素是未活动的，这样显然不是rust应该做的事，向量不应该携带额外信息或状态，所以rust会直接报错。

解决方式是：可以使用引用，而不是移动它，因为我们只是想访问该元素，如下

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



看如下例子

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

当我们将向量直接传给循环（如 for ... in v）时，会将向量从v中移动出去，让v变成未初始化状态。for循环的内部机制会获取向量的所有权并分解为元素。在每次迭代中，循环会将另一个元素转移给变量s，



# 所有权不会被转移的情况

以下3种方式，一个值的所有权不会发生转移

1. clone：即克隆数据（即深拷贝）

2. copy：如果一个数据结构实现了 Copy 特型，那么它就会使用 Copy 语义。这样，当赋值或者传参时，值会自动按位拷贝（即浅拷贝）

   > 实现了 Copy 特型的类型不会转移所有权，比如标准库中的整数、浮点数、字符这些简单类型，不受所有权转移的约束，它们会直接在栈中复制一份副本

3. borrowing：即“借用”数据，可以对值进行“借用”，以获得值的引用



## Clone Trait

> trait 称为特型，可以理解成接口

只有实现了Clone特型的类型才可以进行克隆，调用`clone()`方法可以拷贝变量的数据，克隆了一个副本，clone是深拷贝，这样就不会使得原始变量的所有权发生转移，而导致原始变量变成未初始化状态。如

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

如图：堆上的数据也会复制一份



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/clone-1.png)



**实现Clone Trait**

自定义类型时，在类型上面加上`#[derive(Clone)]`属性即可实现Clone特型，这样该类型有拥有了克隆的能力。如下Test类型的值就可以使用clone()方法进行克隆

```rust
#[derive(Clone)] // 实现了Clone特型
struct Test {
  age: i32
};
```



## Copy Trait

如果值对应的类型实现了Copy特型，当要移动一个值时，值会自动**按位拷贝（浅拷贝）**，而不是发生所有权的转移，如

```rust
fn main() {
    let a: i32 = 11;
    let b = a; // 这里是复制一份a，而不是所有权转移，i32类型实现了Copy特型

    println!("a={} b={}", a, b); // 这里还可以打印a，不会报错
}
```



如下面的错误例子，data 的类型 `Vec<i32>`，它没有实现 Copy trait，在赋值或者函数调用时无法 Copy 复制值，于是就按默认会进行所有权转移。在所有权转移之后，原先的变量 data 变成了未定义状态，无法访问

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



**rust哪些数据结构实现了Copy 特型**

1. 原生类型，如所有的机器整数类型、浮点数类型、char类型、bool类型，包括函数、不可变引用和裸指针实现了 Copy
2. 固定大小的数组和元组：如果其内部的数据结构实现了 Copy，那么它们也能通过 ``#[derive(Copy, Clone)]``实现Copy
3. 默认情况下，struct类型和enum类型不是Copy类型 。如果用户自定义的结构体的所有字段本身都是Copy类型，那么可以通过将属性 `#[derive(Copy, Clone)]` 放置到此结构体上，此结构体就变成Copy类型



根据经验，任何在丢弃值时需要做一些特殊操作的类型都不能是Copy类型，比如

1. 可变引用没有实现 Copy
2. 非固定大小的数据结构，没有实现 Copy，如向量Vec需要释放自身元素，String需要释放缓冲区
3. File需要关闭自身文件句柄
4. `Box<T>` 拥有从堆中分配的引用目标
5. MutexGuard需要解锁自身互斥锁

> 也可参考 [官方文档介绍Copy trait](https://doc.rust-lang.org/std/marker/trait.Copy.html)



**实现Copy Trait**

我们的自定义类型可以在类型上方加上 `#[derive(Copy)]`，即可为该类型实现Copy特型。如

```rust
#[derive(Copy, Clone)]
struct Test {
  age: i32
};
```

注意：实现了Copy的类型也要求实现Clone



## Clone和Copy的区别

当copy和clone都是rust的默认实现时，不考虑自己实现Copy特型和Clone特型的情况：

- Copy时：只拷贝变量本身的值，如果这个变量指向了其它数据，则不会拷贝其指向的数据，即浅拷贝
- Clone时：拷贝变量本身的值，如果这个变量指向了其它数据，则也会拷贝其指向的数据，即深拷贝，Rust会对每个字段每个元素递归调用clone()，直到最底部。

所以，使用Clone的默认实现时，clone()操作的性能是较低的。但可以自己实现自己的克隆逻辑，也不一定总是会效率低。比如Rc类型，它的clone用于增加引用计数，同时只拷贝少量数据，它的clone效率并不低。



## Borrowing借用

### 用法

借用：可以对值进行借用，以获得值的引用，表示方式是加一个 `&符号`在值前面

> 如`let b = &a; `，此时`&a`表示借用a，这是一个借用动作，它的结果是得到一个引用类型，所以b是引用类型；

> 此处可以把&理解为C++的指针



对一个值进行借用，值不会发生所有权的转移。



**默认情况下，Rust 的借用都是只读的，一个值可以有多个只读引用，这种引用是非拥有型指针（即不是值的拥有者），有着受限的生命周期**



**只读引用实现了 Copy trait，也就意味着引用的赋值、传参都会产生新的浅拷贝。**

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



### 借用的约束

对值的引用的约束：借用的生命周期不能超过值的生命周期

```rust
// 正确用法
fn main() {
    let data = vec![1, 2, 3, 4];
    // data的生命周期是main函数结束，sum函数处于main的下一层调用栈中，所以sum调用结束后main函数还会继续执行，所以在 main() 函数中定义的 data 生命周期要比 sum() 中对 data 的引用要长，这样不会有任何问题
    println!("sum of data1: {}", sum(&data));
}

fn sum(data: &Vec<u32>) -> u32 {
    data.iter().fold(0, |acc, x| acc + x)
}
```



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



# 引用类型 和 引用

## 引用

**引用：其实就是指针，是指向特定类型数据的一个指针或一个胖指针(有额外元数据的指针)**，**它的值是内存地址**。**引用可以指向内存中任何地方的值，不仅仅是栈上的。**

> 例如`&123`表示的是一个指向数据值123的一个指针。打印一个`i32`变量，结果是这个变量的值；同理，打印一个引用，结果就是引用的值：它表示指向的变量的内存地址。

> 如果要打印一个引用本身的地址，就要对引用再加上一层引用，如打印引用(&a)的地址，要打印`&&a`才行。



可以分为

* 拥有型指针：如`Box<T>`、String值和Vec值内部的指针，当拥有者被丢弃时，它的引用目标也会随之消失

* 非拥有型指针：也叫引用，这种指针对引用目标的生命周期没有影响。所以要注意**任何引用的生命周期不能超出其指向的值**



rust把创建对某个值的引用的操作称为 **借用** 那个值 （凡是借用，终须归还）



## 对值的引用

引用能让你在不影响其所有权的情况下访问值，引用分为两种

* 共享引用：允许读取但是不能修改其引用目标，可以同时拥有任意数量的对特定值的共享引用。共享引用是Copy类型。表达式 &e 会产生对 e值的共享引用，如果 e 的类型为T，那么 &e 的类型就是 &T
* 可变引用：允许读取和修改值。只能存在一个对特定值的可变引用。可变引用不是Copy类型。表达式 &mut e 会产生对 e值的可变引用，如果 e的类型为T，可以将其类型写成&mut T





* 只要存在对一个值的共享引用，即使是它的拥有者也不能修改它，该值会被锁定。
* 如果有某个值的可变引用，那么它就会独占对该值的访问权，在可变引用消失前，即使拥有者也根本无法使用该值





* 按值传递：当通过将`值的所有权`转移给函数的方式将这个值传给函数，可以说成是按值传递了它

* 按引用传递：当将`值的引用`传递给函数，可以说是按引用传递了它



例子

```rust
use std::collections::HashMap;

type Table = HashMap<String, Vec<String>>;

fn main() {
    // 起一个table别名，它实际是HashMap

    let mut table = Table::new();

    table.insert(
        "zhangsan".to_string(),
        vec!["11".to_string(), "111".to_string()],
    );

    table.insert(
        "lisi".to_string(),
        vec!["22".to_string(), "222".to_string()],
    );

    table.insert(
        "wangwu".to_string(),
        vec!["33".to_string(), "333".to_string()],
    );

    show(&table);

    sort_works(&mut table);
}

// Table传的是引用
fn show(table: &Table) {
    // artist是&String类型，works是&Vec<String>类型
    for (artist, works) in table {
        print!("works by {}", artist);

        // work是&String类型
        for work in works {
            println!(" {}", work);
        }
    }
}

fn sort_works(table: &mut Table) {
    // _artist是&String类型，works是&mut Vec<String>类型
    for (_artist, works) in table {
        works.sort(); // 这里要排序，不能传共享引用，因为共享引用不允许修改
    }
}
```



### 共享与可变

在共享引用的整个生命周期中，它引用的目标会保持只读状态，即不能对`引用目标`赋值 或 将值移动到别处。

如下代码就会出现悬空指针，因为r的生命周期内发生了移动向量的残做

```rust
 let v = vec![4, 8, 19, 27, 34, 10];
 let r = &v;
 let aside = v; // 把向量移给aside，v是未初始化状态
 r[0]; // 错误，这里v是未初始化状态，r成了悬空指针
```

TODO 差一个图



**共享引用是只读访问**

1. 共享引用借用的值是只读的
2. 在共享引用的整个生命周期，无论是它的引用目标，还是可从该引用目标间接访问的任何值，都不能被任何代码改变
3. 这种结构中不能存在对任何内容的有效可变引用，其拥有者应该保持只读状态

```rust
fn main() {
    let mut x = 10;
    let r1 = &x;
    let r2 = &x; // 符合第1点，正确：允许多个共享借用

    x += 10; //符合第2点，错误，不能赋值给x，因为它已经被借出

    let m = &mut x; // 符合第3点，错误，不能把x借入为可变引用，因为它覆盖在已借出的不可变引用的生命周期内

    println!("{}, {}, {}", r1, r2, m); // 这些引用在这里使用的，所以它们的生命周期至少要存续这么长
}
```



**可变引用是独占访问**

1. 可变引用借用的值 只能通过该引用访问
2. 在可变引用的整个生命周期中，无论是它的引用目标，还是该引用目标间接访问的任何目标，都没有任何其他路径可访问
3. 对可变引用来说，唯一能和自己的生命周期重叠的引用就是从可变引用本身借出的引用

```rust
 let mut y = 20;
 let m1 = &mut y;
 let m2 = &mut y; // 错误，不能多次借入可变引用
 let z = y; // 错误，不能使用 y, 因为它涵盖的已借出的可变引用的生命周期内
 println!("{}, {}, {}", m1, m2, z);

```





可以从共享引用中重新借入共享引用，如 

```rust
let mut w = (107, 109);
let r = &w;
let r0 = &r.0; // 正确，把共享引用重新借入为共享引用
let m1 = &mut r.1; // 错误，不能把共享引用重新借入为可变引用
println!("{}", r0);
```





可以从可变引用中重新借入可变引用，如

```rust
fn main() {
    let mut v = (107, 109);
    let m = &mut v;
    let m0 = &mut m.0; // 正确，从可变引用借入可变引用
    *m0 = 137;
    let r1 = &m.1; // 正确，从可变引用借入共享引用，并且不能和 m0 重叠
    v.1; // 错误，禁止通过其他路径访问
    print!("{}", r1);
}
```





## 引用类型

**引用类型：是一种数据类型，它所保存的值是一个引用。引用类型可以分为 引用类型 和 可变引用类型。**

* `&T`：表示类型T的引用类型， 是一个对于 T 的「不可变引用」（immutable reference）或者「常量引用」（const reference），也叫共享引用，意味着可能存在对同一个值的其它引用，也许是在别的线程或是当前线程的调用栈中

* `&mut T`：表示类型T的可变引用类型， 一般称为对类型为 T的数据的「可变引用」（mutable reference），也叫独占引用，意味着在同一时刻，同一个值不可能存在别的引用



## 类型T的引用类型

类型T的引用类型：可以用`&T`表示，也可以用`ref`表示

> 注意：此时的“&”符是应用在类型声明上的，表示的是引用类型
>



#### 用法1：用&T表示

`&符号`用在`类型声明`上，表示的是引用类型

例1：

```rust
// &String类型，表示String的引用类型
// &i32类型，表示i32的引用类型
// &&i32类型，表示“&i32引用”的引用类型
// &123表示的是123这个值的引用
fn main() {
    let a = String::from("hello");
    
    // 这里只关注&用在类型声明上的情况，即&String。而&用在表达式上的，即&a表示的是借用，它得到的是一个引用类型
    let b: &String = &a; // b是&String类型，表示String的引用类型；

    let c = &123; // c是&i32类型，表示i32的引用类型，即&123是123这个值的引用；注意：此时&123是借用，得到的是一个引用类型
    let d = &c; // d是&&i32类型，表示“&i32引用”的引用类型

    // a = hello, b = hello, c = 123, d=123
    println!("a = {}, b = {}, c = {}, d={}", a, b, c, d);
}
```

例2：

```rust
fn main() {
    let a = 1; // i32类型
    let b = 2;
    let rst = sum(&a, &b);
    println!("{} + {} = {}", a, b, rst); // 1 + 2 = 3
}

// &i32表示a是一个i32的引用类型
fn sum(a: &i32, b: &i32) -> i32 {
    a + b
}
```



#### 用法2：用ref表示

1. ref用在变量绑定上，也是指引用类型
2. 在模式匹配时，用ref关键字也是表示引用类型



例1：

```rust
fn main() {
    // 引用类型声明时可以不赋值；
    // 表示a是&i32类型；也可以直接声明为let ref a;
    let ref a: i32; 
    a = &1; // 因为a是引用类型，所以只能赋予&1，&1得到的是一个i32类型的引用类型
    println!("{} ", a); // 1
  
    // 引用类型在声明时就赋值
    let ref b = 2; // 表示b是&i32类型
    println!("{} ", b); // 2
  
    let c = &2; // c也是&i32类型，引用类型
    println!("{} ", c); // 2
}  
```

> `let ref a`表示声明了一个引用类型，它只能绑定到某次借用动作上，&1即借用1。



例子2：

```rust
fn main() {
    let s = Some(String::from("Hello!"));
    match s {
        Some(ref t) => println!("t = {}", t), // ref引用类型，此时s的所有权不会转移给t
        _ => {}
    }
    println!("s = {}", s.unwrap()); // 依然可以访问s
}
```



## 类型T的可变引用类型

类型T的可变引用类型：可以用`&mut T`表示，也可以用`ref mut`表示。



#### 用法1：用&mut T表示

**如果想要通过引用去修改源数据，需要使用`&mut v`来创建可修改源数据v的可变引用**。

> 因为直接使用`&`创建出来的引用是只读的，所以只能通过该引用去读取其指向的数据，但是不能通过引用去修改指向的数据。



例1：

> 注意：想要通过`&mut`引用去修改源数据，要求原变量是可变的。

```rust
// 不合法
fn main() {
  let n = 33;
  let n_ref = &mut n;  // 编译错误，因为n不是可变的，n需要加上mut
}

// 合法
fn main(){
  // mut表示n是可变的
  let mut n: i32 = 66; // i32类型
  
  // &mut n 得到的是一个i32类型的可变的引用类型
  // n_ref是&mut i32类型，因为n是mut可变的，注意变量n_ref本身是不可以修改的，&mut表示n_ref指向的内容是可以修改的
  let n_ref = &mut n;
  
  *n_ref = 88; // 修改n的值为88，此处*表示的是解指针
  println!("{}", n); // 88
}
```

例2：

```rust
// 表示x是可变的引用类型
fn foo(x: &mut i32) {
    *x = 2; // 修改为2
}

fn main() {
    // a是可变的
    let mut a: i32 = 1;
  
    // 传了1个可变的引用类型进去
    foo(&mut a);
  
    println!("{}", a); // 2
}
```

例3：

```rust
// 不合法
fn main() {
    let s = Some(String::from("Hello!"));
    match s {
        Some(t) => println!("t = {}", t), // 所有权转移到了t
        _ => {}
    }
    println!("s = {}", s.unwrap()); // 编译出错，s的所有权转移到t了，不能再访问s
}

// 合法
// 可做如下修改:方式1
fn main() {
    let s = Some(String::from("Hello!"));
    match s {
        Some(ref t) => println!("t = {}", t), // ref引用类型，此时s的所有权不会转移给t
        _ => {}
    }
    println!("s = {}", s.unwrap()); // 依然可以访问s
}
  
// 方式2
fn main() {
    let s = Some(String::from("Hello!"));
    
    // 使用&s，这里是借用，所以当传到Some(t)里后，t的值和&s一样，所以不会使得s的所有权转移
    match &s {
        Some(t) => println!("t = {}", t),
        _ => {}
    }
    println!("s = {}", s.unwrap()); // 依然可以访问
}
```

例4：

```rust
fn main() {
    let data = vec![1, 2, 3, 4]; // data是Vec<i32>类型
    let data1 = &data; // data1是&Vec<i32>类型
    let data2 = &data;
    
    // 此时打印data1，data1是一个引用，打印结果就是引用的值，引用存的是内存的地址，它指向了变量的内存地址
    // addr of value: 0x7ff7beb99848 0x7ff7beb99848
    println!(
        "addr of value: {:p} {:p}",
        data1, data2
    );
}
```



#### 用法2：用ref mut表示

例1：

```rust
fn main() {
    // 表示a是可变的
    let mut a: i32 = 1;

    // c是一个引用类型，变量c本身是不可修改的，加了mut表示但c指向的内容是可以修改的
    let ref mut c = a;
    *c = 3; // 修改c指向的内容

    println!("{}", a); // 3
}
```



## 引用的2个限制

**为了保证内存安全，Rust 对可变引用的使用也做了严格的约束：**

1. 在一个作用域内，仅允许一个活跃的可变引用

   > 所谓活跃，就是真正被使用来修改数据的可变引用，如果只是定义了，却没有使用或者当作只读引用使用，不算活跃。

2. 在一个作用域内，活跃的可变引用（写）和只读引用（读）是互斥的，不能同时存在

   > 简单来说，就是要注意交叉使用 可变引用 和 只读引用



**例1：**

以下代码会报错，此时存在多个可变引用

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &mut s;
    let r2 = &mut s; // 报错，多s创建了两个引用

    println!("{}, {}", r1, r2);
}
```

**例2：**

以下代码也会报错，此时存在可变引用和只读引用

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    let r3 = &mut s; // 注意是可变引用

    // 这里会报错，因为r1和r2是只读引用，r3的声明在输出r1和r2前面，r3可能会改变s，有可能涉及到s内存的重新分配，这是不安全的
    println!("{}, {}, and {}", r1, r2, r3); 
}

// 可改成以下正确写法
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    println!("{}, {}", r1, r2); // 先输出了可读引用

    let r3 = &mut s;
    println!("{}", r3);
}
```



## 引用例子分析

分析以下例子各输出什么

```rust
fn main() {
    let data = vec![1, 2, 3, 4]; // Vec<u32>类型是动态大小，存储在堆中

    let data1 = &data;
    let data2 = &data;
    // 值的地址是什么？引用的地址又是什么？

    // 以下3个输出的都是 堆的地址
    // &data就是"data的胖指针ptr的值"，该指针指向堆的地址，所以&data就是堆的地址
    // data1和data2都指向了&data，所以也是堆地址
    println!("data引用的地址: {:p}", &data); // 0x7ff7b1bf6628
    println!("data1的值: {:p}", data1); // 0x7ff7b1bf6628
    println!("data2的值: {:p}", data2); // 0x7ff7b1bf6628

    // data1是一个引用类型，&data1就是输出`引用data1`的地址
    println!("`引用data1`的地址: {:p}", &data1); // 0x7ff7b4968640
    println!("`引用data2`的地址: {:p}", &data2); // 0x7ff7b4968648

    println!("`data引用`的引用: {:p}", &&data); // 0x7ff7b49687f0

    println!("sum of data1: {}", sum(data1)); // 10

    // 堆上各个数据的地址
    // [0x7f9581705e70, 0x7f9581705e74, 0x7f9581705e78, 0x7f9581705e7c]
    println!(
        "每个元素的地址 [{:p}, {:p}, {:p}, {:p}]",
        &data[0], &data[1], &data[2], &data[3]
    );
}

fn sum(data3: &Vec<u32>) -> u32 {
    // data3指向了&data，所以也是堆地址
    println!("data3的值 {:p}", data3); // 0x7ff7b9eed628
    println!("`引用data3的地址` {:p}", &data3); // 0x7ff7b9eed430
    // data3引用解指针，其实就是data3的值，也就是堆地址
    println!("data3引用解指针，堆地址 {:p}", *&data3); // 0x7ff7b9eed628

    data3.iter().fold(0, |acc, x| acc + x)
}

```

分析如下图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/%E5%BC%95%E7%94%A8%E7%9A%84%E5%A0%86%E6%A0%88%E5%9C%B0%E5%9D%80.png)

* data的值是[1,2,3,4]

* data的ptr指针存的是堆的地址 0x7ff7b9eed628，指向堆

* data1、data2和data3都是data的引用(用&data表示)，它们的ptr指针指向data的ptr指针，存的也是堆的地址0x7ff7b9eed628

  > &data就是data的引用，它的值就是data的地址，即为"data的胖指针ptr的值"，该指针指向堆的地址，所以&data就是堆的地址；
  >
  > 这里data有很多只读引用指向它，但堆上的数据依旧只有 data 一个所有者，所以值的任意多个引用并不会影响所有权的唯一性



## 使用引用



TODO



## 引用安全（涉及生命周期）

**借用局部变量**

不能借用对局部变量的引用，并将其移出变量的作用域，因为引用的生命周期不能超过其引用的值

```rust
{
  let r;
  
  {
    let x = 1;
    r = &x;
  } // x的生命周期在这个括号结束就结束了。r的生命周期不能超出x本身，因为它是x的引用，当超出了x的作用域，r将是一个悬空指针
  
  assert_eq!(*r, 1); // 错误，r是局部变量x的引用，*r 试图读取 x 所占用的内存。把这句放到上面大括号内就正常
}
```

rust会尝试为程序中的每个引用类型分配一个生命周期，以表达根据其使用方式应施加的约束。

生命周期是rust在编译期虚构的产物。



**将引用作为函数参数**

看一下在函数中将一个引用存储在全局变量(静态变量)中，如

```rust
fn main() {
    static mut STASH: &i32 = &128; // 必须初始化

    fn f(p: &'static i32) {
        unsafe {
            STASH = p;
        }
    }
}
```

* 静态变量STASH必须初始化
* 可变静态变量本质不是线程安全的，只能在unsafe块中访问可变静态变量
* 注意f函数 p的生命周期是'static ，如果不加这个限制，p的生命周期默认是 'a，因为STASH静态变量会存在程序的整个执行过程中，它的生命周期成为 'static生命周期，它的生命周期比 'a长，所以也要接受有 'static 生命周期的引用，不然STASH就是一个悬空指针



**把引用传给函数**

要注意 “引用的生命周期” 是否小于等于 调用函数时的生命周期，如下 &y 的生命周期为调用f函数时的生命周期

```rust
fn main() {
    fn f(p: &i32) {
        println!("{}", p);
    }

    let y = 10;
    f(&y);
}
```

解析以上代码的生命周期，把生命周期限制显示的标注出来

```rust
fn main() {
    // 函数f和p的生命周期都是 'a
    fn f<'a>(p: &'a i32) {
        println!("{}", p);
    }

    let y = 10;
    f(&y); // &y的生命周期即为调用f函数时的生命周期，rust会自己推断出 &y 的生命周期符合f函数的生命周期限制
}
```



**返回引用**

函数通常会接收某个数据结构的引用，然后返回对该结构的某个部分的引用。如下函数返回对切片中最小元素的引用

```rust
// v应该至少有一个元素
fn smallest(v: &[i32]) -> &i32 {
        let mut s = &v[0];
        for r in &v[1..] {
            if *r < *s {
                s = r;
            }
        }
        s // 返回值为s
}


```

smallest函数的生命周期标识为，可以知道smallest的参数和返回值的生命周期必须相同，都是 'a

```rust
fn smallest<'a>(v: &'a [i32]) -> &'a i32 {}
```



```rust
// 调用如下
let s;
{
   let parabola = [9, 4, 1, 0];
   s = smallest(&parabola); // 因为 smallest返回值的生命周期 跟 &parabola 的生命周期时相同的，所以只在大括号内，这里赋给了s，s的生命周期超过smallest返回值的生命周期，所以下面解引用s会报错
}
assert_eq!(*s, 0); // 这里会报错，因为指向了已经被丢弃的数组的元素
```



**包含引用的结构体**

rust要求包含引用的类型都要接受显示生命周期参数



```rust
fn main() {
    struct S {
        r: &i32,
    }

    let s;

    {
        let x = 10;
        s = S { r: &x };
    }

    assert_eq!(*s.r, 10);
}
```

以上代码存在2处错误

1. 结构体 S 里的r是引用类型，这里要标注生命周期
2. 最后一行 *r 是 x的引用，此时x已经被丢弃了，访问会报错



针对第1点，有两种修改方式

第一种：为引用类型标注生命周期，可以用全局生命周期，修改如下

```rust
 struct S {
    r: &'static i32
 }
```

因为每当一个引用类型出现在另一个类型的定中时，必须写出它的生命周期。这样r只能引用贯穿整个生命周期的i32值，这种限制太严格，所以也可以用另一种写法

第二种：给类型指定一个生命周期的参数 'a 并将其应用在r上

```rust
struct S<'a> {
  r: &'a i32
}
```

这样S类型也有一个生命周期，这样创建的每个S类型的值都会获得一个全新的生命周期 'a，它会受到该值的使用方式的限制。所以存储在 r 中的任何引用的生命周期一定等于或大于 'a 生命周期。 

表达式s = `S { r: &x };` 创建了一个新的S值，并存储到变量s中，变量s的生命周期为 'a，

当将 &x 存储在r字段中时，就会将 'a 完全限制在了x的生命周期内部。可以修改为如下

```rust
fn main() {
    struct S<'a> {
        r: &'a i32,
    }

    let s;

    {
        let x = 10;
        s = S { r: &x };
        assert_eq!(*s.r, 10);
    }
}
```



**不同的生命周期参数**



错误代码

```rust
fn main() {
    // 注意这里x和y引用都是相同的生命周期
    struct S<'a> {
        x: &'a i32,
        y: &'a i32,
    }

    let x = 10;
    let r;
    {
        let y = 20;
        {
            let s = S { x: &x, y: &y }; // 这里&y会报错 borrowed value does not live long enough
            r = s.x;
        }
    }

    println!("{}", r);
}
```

上面代码不会出现悬空指针。对 y 的引用会保留在 s 中，它会在 y 之前超出作用域。对x的引用最终会出现在 r 中，它的生命周期不会超出x。



错误原因推理如下：

1. S 两个字段有相同的生命周期

2. 赋值 r = s.x 时，就要求 'a 覆盖r 的生命周期

3. 用 &y 初始化 s.y 时，这就要求 'a 不能长于y 的生命周期

显然上面第2和第3点不能同时成立，所以不存在这样的一个 'a 生命周期。修正如下，让S里的每个引用都有各自的生命周期就可以了，如

```rust
fn main() {
    // x 和 y 必须不同的生命周期
    struct S<'a, 'b> {
        x: &'a i32,
        y: &'b i32,
    }

    let x = 10;
    let r;
    {
        let y = 20;
        {
            let s = S { x: &x, y: &y };
            r = s.x;
        }
    }

    println!("{}", r);
}
```



**省略生命周期参数**

rust默认会为需要生命周期的每个地方分配不同的生命周期，如下代码

```rust
fn main() {
    struct S<'a, 'b> {
        x: &'a i32,
        y: &'b i32,
    }

    fn sum_r_xy(r: &i32, s: S) -> i32 {
        r + s.x + s.y
    }
}
```

这里sum_r_xy函数的签名的生命周期实际，如下

```rust
 fn sum_r_xy<'a, 'b, 'c>(r: &'a i32, s: S<'b, 'c>) -> i32 {
     r + s.x + s.y
 }
```



* 如果函数的参数只有一个生命周期，那么rust会假设返回值具有同样的生命周期
* 如果函数的参数有多个生命周期，那么rust就会要求你明确指定返回值的生命周期
* 如果函数是某个类型的方法，并且具有引用类型的selft参数，那么rust会假定返回值的生命周期 与 self参数的生命周期相同（self指调用方法的对象）





# 解引用

解引用：表示解除引用，即**通过引用获取到该引用所指向的原始值**。可以用*表示，也可以用 “&绑定变量”表示

## 用法1：用*表示

解引用：在引用前面加一个星号，如`*a`（其中a是一个引用）

```rust
fn main() {  
   let a = &666; // a是&i32类型，是一个引用类型
  
   // *a 表示解引用
   let b = *a; // b是i32类型

   println!("a: {} b: {}", a, b); // a: 666 b: 666 
}
```

rust会自动解多层嵌套引用，如

```rust
fn main() {
    let a: &i32 = &123;
    let b: &&i32 = &a;
    let c: &&&i32 = &b;

    // 解多层嵌套引用
    println!("a = {}, b = {}, c = {}", a, b, c); // a = 123, b = 123, c = 123
  
    println!("*a = {}, **b = {}, ***c = {}", *a, **b, ***c); // *a = 123, **b = 123, ***c = 123
}
```



## 用法2：用”&绑定变量“

解引用：也可以在声明变量时，在变量前加上&，即“&绑定变量”，如`let &b = a;`

```rust
fn main() {
    let a = &666; // a是&i32类型，是一个引用类型
    // 解引用
    let &b: i32 = a; // 在&变量绑定上表示解地址，此时b是i32类型
    println!("a: {} b: {}", a, b); // a: 666 b: 666 
}  
```



## 自动解引用

在某些情况下，Rust会自动进行解引用

1. 使用`.操作符`时(包括取属性值和方法调用)，会隐式地尽可能`解除或创建`多层引用

   > Rust会自动分析func()的参数，并在需要时自动创建或自动解除引用。
   >
   > 例如以`abc.func()`有可能会自动转换为`&abc.func()`，反之，`&abc.func()`也有可能会自动转换为`abc.func()`

2. 使用比较操作符时，若比较的两边是`相同类型的引用`，则会自动解除引用到它们的值然后比较

   > 例如有引用类型的变量n，那么`n > &30`和`*n > 30`的效果是一样的



**例子**

例1：使用 ".操作符" 自动解引用的例子

```rust
// 使用 ".操作符" 自动解引用的例子
struct Person {
  first_name: String,
  last_name: String,
  age: u8
}

fn main() {
  let pascal = Person {
    first_name: "san".to_string(),
    last_name: "zhang".to_string(),
    age: 28
  };

  let r = &pascal; //r是&Person类型

  // r.first_name自动解引用，不然得这样子写 (*r).first_name
  println!("Hello, {}!", r.first_name);
}
```

例2：.操作符自动解引用的例子

```rust

fn main() {
    let n = &123;
    // .操作符自动解引用，等价于 *n > 30
    if n > &30 {
        println!("{}", n); // 123
    }
    
    // 正常写法，自己手动解引用
    if *n > 30 {
        println!("{}", n); // 123
    }
}
```

例3：使用 ".操作符" 自动创建引用的例子

```rust
// 使用 ".操作符" 自动创建引用的例子
fn main() {
    let mut numbers = [3, 1, 2];
    // 数组的sort()方法需要一个&mut self，.操作符会隐式地对左边的操作符借用一个引用
    // 此时 .sort()等价于 (&mut numbers).sort();
    numbers.sort();

    println!("{:?}", numbers);
}
```



# 总结

## 一些含义

```rust
// 含义：a绑定到字符串资源A上，拥有资源A的所有权
let a = "xxx".to_string();　　

// 含义：a绑定到字符串资源A上，拥有资源A的所有权，同时a还可绑定到新的资源上面去（更新绑定的能力，但新旧资源类型要同）；
let mut a = "xxx".to_string();　

// 含义：a绑定的资源A转移给b，b拥有这个资源A
let b = a;

// 含义：a绑定的资源A借给b使用，b只有资源A的读权限
let b = &a;

// 含义：a绑定的资源A借给b使用，b有资源A的读写权限
let b = &mut a;

// 含义：a绑定的资源A借给b使用，b有资源A的读写权限。同时，b可绑定到新的资源上面去（更新绑定的能力
let mut b = &mut a;

//含义：传参的时候，实参d绑定的资源D的所有权转移给c
fn do(c: String) {}　

// 含义：传参的时候，实参d将绑定的资源D借给c使用，c对资源D只读
fn do(c: &String) {}　

// 含义：传参的时候，实参d将绑定的资源D借给c使用，c对资源D可读写
fn do(c: &mut String) {}

// 含义：传参的时候，实参d将绑定的资源D借给c使用，c对资源D可读写。同时，c可绑定到新的资源上面去（更新绑定的能力）
fn do(mut c: &mut String) {}　
```



## 所有权、引用、生命周期

1. 一个值在同一时刻只有一个所有者。

   > 当所有者离开作用域，其拥有的值会被丢弃。赋值或者传参会导致值 Move，所有权被转移，一旦所有权转移，之前的变量就不能访问

2. 如果值实现了 Copy trait，那么赋值或传参会使用 Copy 语义，相应的值会被按位拷贝，产生新的值

3. 一个值可以有多个只读引用

4. 一个值可以有唯一一个活跃的可变引用。可变引用（写）和只读引用（读）是互斥的关系

5. 引用的生命周期不能超出值的生命周期



## 不同情况下的&含义

1. 在类型声明上，表示引用类型，即&T

   > 如 let a: &i32 = &123; // 此时&i32就是类型声明

2. 在表达式上，表示的是借用，其结果是得到一个引用类型

   > 如 let a = &123; 此时&123表示借用123，其得到一个&i32的引用类型，所以a的类型为&i32

3. 在变量绑定上，表示解引用操作，与*类似

   > 如
   >
   > let a = &123;
   >
   > let &b = a; // 此时b的类型是i32，&b表示解引用，所以b的值是123；也等价于 let b = *a;

   

## 不同情况下的ref的含义

1. 在变量绑定上，表示引用类型

   > 如let ref a = 123;  // 此时表示a的类型是&i32，等价于let a = &123;

2. 在模式匹配上，表示引用类型

```rust
fn main() {
    let s = Some(String::from("Hello!"));
    match s {
        Some(ref t) => println!("t = {}", t), // ref引用类型，此时s的所有权不会转移给t
        _ => {}
    }
    println!("s = {}", s.unwrap()); // 依然可以访问s
}
```



# 参考 

* [The Rust Programming Language](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
* [【翻译】Rust中的引用](https://juejin.cn/post/6844904106310516744)
* [陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/415988)
* [Rust入门秘籍](https://rust-book.junmajinlong.com/ch3/07_reference_type.html)
* [Rust语言圣经(Rust Course)](https://course.rs/basic/ownership/borrowing.html)
* [Rust中mut, &, &mut的区别](https://blog.csdn.net/hbuxiaofei/article/details/108471806)
* [Rust 中的 & 和 ref](https://blog.csdn.net/quicmous/article/details/120489008)
* [理解 Rust 引用和借用](https://zhuanlan.zhihu.com/p/59998584)
* [正确的Rust引用类型心智模型](https://zhuanlan.zhihu.com/p/88926962)

