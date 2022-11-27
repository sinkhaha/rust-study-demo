# 引用类型 和 借用

## 1. 引用类型

**引用类型是一种数据类型，它表示其所保存的值是一个引用**。



**引用类型：分为 引用类型 和 可变引用类型。**

* &T：表示类型T的引用类型， 是一个对于 T 的「不可变引用」（immutable reference）或者「常量引用」（const reference）

  > 意味着在同一时刻，同一个值不可能存在别的引用

* &mut T：表示类型T的可变引用类型， 一般称为对类型为 T的数据的「可变引用」（mutable reference），

  > 意味着可能存在对同一个值的其它引用，也许是在别的线程或是当前线程的调用栈中



**Rust的引用其实就是指针，是指向特定类型数据的一个指针或一个胖指针(有额外元数据的指针)**，它的值是内存地址。

> 例如`&123`表示的是一个指向数据值123的一个指针

> 打印一个`i32`变量，结果是这个变量的值；同理，打印一个引用，结果就是引用的值：它表示指向的变量的内存地址。
>
> 如果要打印一个引用本身的地址，就要对引用再加上一层引用，如打印引用(&a)的地址，要打印`&&a`才行。

引用可以指向内存中任何地方的值，不仅仅是栈上的。



注意：此时的“&”符是应用在类型声明上的，表示的是引用类型



### 1.1 类型T的引用类型

类型T的引用类型：可以用&T表示，也可以用ref表示。



#### 用法1：用&T表示

&符号用在类型声明上，表示的是引用类型

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

用法1：ref关键字用在变量绑定上，也是指引用类型

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



用法2：在模式匹配时，用ref关键字也是表示引用类型

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



### 1.2 类型T的可变引用类型

类型T的引用类型：可以用`&mut T`表示，也可以用`ref mut`表示。



#### 用法1：用&mut T表示

因为直接使用`&`创建出来的引用是只读的，所以只可以通过该引用去读取其指向的数据，但是不能通过引用去修改指向的数据。



**如果想要通过引用去修改源数据，需要使用`&mut v`来创建可修改源数据v的可变引用**。



注意，想要通过`&mut`引用去修改源数据，要求原变量是可变的。

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



## 2. 借用

### 用法

借用：即 &符号用在表达式上，如`let b = &a; `，此时`&a`表示借用a，这是一个动作，它的结果是得到一个引用类型，所以b是引用类型。

> 此处可以把&理解为C++的指针

**默认情况下，Rust 的借用都是只读的**。

一个值可以有多个只读引用。

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

借用对值的引用的约束：借用不能超过值的生命周期。

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



## 3. rust的限制

为了保证内存安全，Rust 对可变引用的使用也做了严格的约束：

1. 在一个作用域内，仅允许一个活跃的可变引用。

   > 所谓活跃，就是真正被使用来修改数据的可变引用，如果只是定义了，却没有使用或者当作只读引用使用，不算活跃。

2. 在一个作用域内，活跃的可变引用（写）和只读引用（读）是互斥的，不能同时存在。

   > 不要交叉使用 可变引用 和 只读引用

   

如以下代码会报错，此时存在多个可变引用

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &mut s;
    let r2 = &mut s; // 报错，多s创建了两个引用

    println!("{}, {}", r1, r2);
}
```

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

Rust绝大多数时候不会自动地解除引用。但在某些环境下，Rust会自动进行解引用。

自动解引用的情况有

1. 使用`.`操作符时(包括取属性值和方法调用)，会隐式地尽可能解除或创建多层引用

   > Rust会自动分析func()的参数，并在需要的时候自动创建或自动解除引用。例如以`abc.func()`有可能会自动转换为`&abc.func()`，反之，`&abc.func()`也有可能会自动转换为`abc.func()`

2. 使用比较操作符时，若比较的两边是相同类型的引用，则会自动解除引用到它们的值然后比较

   > 例如有引用类型的变量n，那么`n > &30`和`*n > 30`的效果是一样的



例子：

```rust
// 使用 .操作符 自动解引用的例子
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



```rust
// 使用 .操作符 自动创建引用的例子
fn main() {
    let mut numbers = [3, 1, 2];
    // 数组的sort()方法需要一个&mut self，.操作符会隐式地对左边的操作符借用一个引用
    // 此时 .sort()等价于 (&mut numbers).sort();
    numbers.sort();

    println!("{:?}", numbers);
}
```



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







# 一些含义

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



# 例子

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



# 总结

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

   

## 不同情况下的ref

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

* https://blog.csdn.net/hbuxiaofei/article/details/108471806
* https://blog.csdn.net/quicmous/article/details/120489008
* https://www.jianshu.com/p/ac519d8c5ec9
* https://zhuanlan.zhihu.com/p/59998584
* https://zhuanlan.zhihu.com/p/88926962
*  https://course.rs/basic/ownership/borrowing.html
*  https://zhuanlan.zhihu.com/p/149850061
* https://rust-book.junmajinlong.com/ch3/07_reference_type.html
* https://time.geekbang.org/column/article/415988
* https://juejin.cn/post/6844904106310516744
* https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html

