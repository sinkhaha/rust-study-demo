# 所有权

## 解决的问题

所有权是为了限制堆上数据的多重引用，解决了谁真正拥有数据的生杀大权问题。



## 所有权的规则

* 一个值只能被一个变量所拥有，这个变量被称为所有者（即 值的所有者是某个变量）

* 一个值同一时刻只能有一个所有者，即不能有两个变量拥有相同的值。

  > 所以变量赋值、参数传递、函数返回等行为，旧的所有者会把值的所有权转移给新的所有者，以便保证单一所有者的约束

* 当所有者(变量)离开作用域，其拥有的值被销毁，内存得到释放

  > 这个过程实际上是调用一个名为drop的函数来销毁数据释放内存



## 所有者的判断

如下语句，谁是谁的所有者？

```rust
let s = String::from("hello");
```

> 注意：变量s不是`堆中字符串数据hello`的所有者

**实际上，变量s是栈中胖指针的所有者，而不是堆中实际数据的所有者。**

因为String字符串的实际数据在堆中，但是String大小不确定，所以在栈中使用一个胖指针结构来表示这个String类型的数据，这个`胖指针中的指针`指向堆中的String实际数据。也就是，**变量s的值是那个胖指针，而不是堆中的实际数据。**

> 但是，由于胖指针是指向堆中数据的，多数时候为了简化理解简化描述方式，也经常会说s是那个堆中实际数据的所有者



## 所有权的转移(move)

```rust
// 错误用法
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

// TODO 补一张所有权转移的图



# 怎么避免所有权被转移

1. clone：克隆数据（深拷贝）
2. copy：如果一个数据结构实现了 Copy trait，那么它就会使用 Copy 语义。这样，当赋值或者传参时，值会自动按位拷贝（浅拷贝）。
3. borrowing：“借用”数据



## Clone Trait

只有实现了Clone Trait的类型才可以进行克隆，使用`clone()`方法可以手动拷贝变量的数据，同时不会让原始变量变回未初始化状态。

```rust
fn main() {
    let data = vec![1, 2, 3, 4];
  
    let data1 = data.clone(); // 克隆data，克隆之后，变量data仍然绑定原始数据，data的所有权不会转移到data1，此时堆会拷贝一份新的[1,2,3,4]数据，然后data1指向新的这份堆数据


    println!("sum of data1: {}", sum(data1));
    println!("sum of data: {:?}", data);
}

fn sum(data: Vec<u32>) -> u32 {
    data.iter().fold(0, |acc, x| acc + x)
}
```

// TODO 补充一张clone后堆上数据的图



**手动实现Clone Trait**

对于那些没有实现Clone Trait的自定义类型，需要手动实现Clone Trait。在自定义类型之前加上`#[derive(Clone)]`即可，如下Test类型的值就可以使用clone()方法进行克隆。

```rust
#[derive(Clone)]
struct Test(i32, i32);
```



## Copy Trait

如果值的类型实现了Copy trait，当要移动一个值时(如赋值、传参、函数返回)，值会自动**按位拷贝（浅拷贝）**，否则就是使用Move进行所有权移动。

> 例如以上的错误示例，data 的类型 Vec<i32>，它没有实现 Copy trait，在赋值或者函数调用时无法 Copy，于是就按默认使用 Move 语义。而 Move 之后，原先的变量 data 无法访问，所以出错。



// TODO copy后堆上的数据是怎么表示的



**rust哪些数据结构实现了Copy Trait：**

1. 原生类型，包括函数、不可变引用和裸指针实现了 Copy
2. 数组和元组，如果其内部的数据结构实现了 Copy，那么它们也实现了 Copy
3. 可变引用没有实现 Copy
4. 非固定大小的数据结构，没有实现 Copy

> 也可参考 [官方文档介绍Copy trait](https://doc.rust-lang.org/std/marker/trait.Copy.html)



**手动实现Copy Trait**

对于那些没有实现Copy的自定义类型，可以手动去实现Copy(要求同时实现Clone)，如下：

```rust
#[derive(Copy, Clone)]
struct test(i32, i32);
```



**Copy和Clone时的区别**

当Copy和Clone都是rust的默认实现时(不考虑自己实现Copy trait和Clone trait的情况)：

- Copy时：只拷贝变量本身的值，如果这个变量指向了其它数据，则不会拷贝其指向的数据
- Clone时：拷贝变量本身的值，如果这个变量指向了其它数据，则也会拷贝其指向的数据

也就是，Copy是浅拷贝，Clone是深拷贝，Rust会对每个字段每个元素递归调用clone()，直到最底部。

> 所以，使用Clone的默认实现时，clone()操作的性能是较低的。但可以自己实现自己的克隆逻辑，也不一定总是会效率低。比如Rc，它的clone用于增加引用计数，同时只拷贝少量数据，它的clone效率并不低。



## borrowing借用

### 用法

借用：即 `&符号用`在表达式上

> 如`let b = &a; `，此时`&a`表示借用a，这是一个借用动作，它的结果是得到一个引用类型，所以b是引用类型

> 此处可以把&理解为C++的指针



**默认情况下，Rust 的借用都是只读的**。一个值可以有多个只读引用。



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

对值的引用的约束：借用不能超过值的生命周期

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



# 引用类型

**引用类型是一种数据类型，它所保存的值是一个引用。引用类型可以分为 引用类型 和 可变引用类型。**

* `&T`：表示类型T的引用类型， 是一个对于 T 的「不可变引用」（immutable reference）或者「常量引用」（const reference），也叫共享引用，意味着可能存在对同一个值的其它引用，也许是在别的线程或是当前线程的调用栈中

* `&mut T`：表示类型T的可变引用类型， 一般称为对类型为 T的数据的「可变引用」（mutable reference），也叫独占引用，意味着在同一时刻，同一个值不可能存在别的引用



**Rust的引用其实就是指针，是指向特定类型数据的一个指针或一个胖指针(有额外元数据的指针)**，它的值是内存地址。

> 例如`&123`表示的是一个指向数据值123的一个指针

> 打印一个`i32`变量，结果是这个变量的值；同理，打印一个引用，结果就是引用的值：它表示指向的变量的内存地址。
>
> 如果要打印一个引用本身的地址，就要对引用再加上一层引用，如打印引用(&a)的地址，要打印`&&a`才行。



**引用可以指向内存中任何地方的值，不仅仅是栈上的。**



## （1）类型T的引用类型

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



## （2）类型T的可变引用类型

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



## （3）rust引用的2个限制

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



# 堆栈数据分析例子

```rust
fn main() {
    let data = vec![1, 2, 3, 4]; // Vec<u32>类型是动态大小，存储在堆中
    let data1 = &data;
    let data2 = &data;
    // 值的地址是什么？引用的地址又是什么？

    // &data是 Vec<u32>堆数据的地址
    // data1是引用类型，data1的值是data的地址，所以也是 Vec<u32>堆数据的地址；data2同data1
    // &data1是data1这个变量本身在栈上的地址
    // &&data是“&data引用”的引用
    // &*data，即先*data是解指针，即得到堆的数据，然后加上&表示引用
    println!(
        "addr of value: {:p}({:p})({:p}), {:p}, addr of data {:p}, data1: {:p}",
        &data, data1, data2, &&data, &*data, &data1, 
    );
    println!("sum of data1: {}", sum(data1));

    // 堆上数据的地址是什么？
    println!(
        "addr of items: [{:p}, {:p}, {:p}, {:p}]",
        &data[0], &data[1], &data[2], &data[3]
    );
}

fn sum(data: &Vec<u32>) -> u32 {
    // 值的地址会改变么？引用的地址会改变么？
    // data是“data引用”即&data的地址
    // &data是“data引用”的引用，即“data引用”(&data)的地址
    // *&data：即“&data”的解引用，&data的值是Vec堆数据的地址
    println!("addr of value: {:p}, addr of ref: {:p}, {:p}", data, &data, *&data);
    data.iter().fold(0, |acc, x| acc + x)
}
```

// TODO 堆栈图



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

