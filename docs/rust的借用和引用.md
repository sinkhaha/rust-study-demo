# 引用类型 和 借用

引用类型有两种，分为引用类型和可变引用类型

> 如类型T的引用类型用&T表示，类型T的可变引用类型用&mut T表示

* `&T` 是一个对于 T 的「不可变引用」（immutable reference）或者「常量引用」（const reference），也叫共享引用

  > 独占引用意味着在同一时刻，同一个值不可能存在别的引用

* `&mut T` 一般称为对类型为 T的数据的「可变引用」（mutable reference），也叫独占引用

  > 共享引用则意味着*可能*存在对同一个值的其它引用，也许是在别的线程（如果 `T` 实现了 `Sync` 的话）或是当前线程的调用栈中。
  >
  > Rust 借用检查器的一个关键职能就是确保独占引用真的是独占性的。



注意：此时的“&”符是应用在类型声明上的，表示引用类型



**引用类型是一种数据类型，它表示其所保存的值是一个引用**。



**引用：Rust的引用其实就是指针，是指向其他数据的一个指针或一个胖指针(有额外元数据的指针)**，它的值是内存地址。

> 打印一个`i32`变量，结果是这个变量的值；同理，打印一个引用，结果就是引用的值：它指向的变量的内存地址。
>
> 如果要打印一个引用本身的地址，就要对引用再加上一层引用，如打印引用(&T)的地址，要打印`&&T`才行。

> 例如`&123`表示的是一个指向数据值123的一个指针。



## 类型T的引用类型

类型T的引用类型：可以用&T表示，也可以用ref表示。



### 用&T表示

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



### 用ref表示

ref关键字用在变量绑定上，也是指引用类型

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



在模式匹配时，用ref关键字也是表示引用类型

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

类型T的引用类型：可以用`&mut T`表示，也可以用`ref mut`表示。



### 用&mut T表示

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

### 用ref mut表示

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



## 借用

借用：即 &符号用在表达式上，如`let b = &a; `，此时`&a`表示借用a，这是一个动作，它的结果是得到一个引用类型，所以b是引用类型

> 此处可以把&理解为C++的指针

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



# 解引用

解引用表示解除引用，即**通过引用获取到该引用所指向的原始值**。可以用*表示，也可以用 “&绑定变量”表示

## 用*表示

解引用用`*T`表示，其中T是一个引用

```rust
fn main() {  
   let a = &666; // a是&i32类型，是一个引用类型
  
   // *表示解引用
   let b = *a; // b是i32类型

   println!("a: {} b: {}", a, b); // a: 666 b: 666 
}
```

rust会自动解多层嵌套引用

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



## 用”&绑定变量“表示

也可以用 "&绑定变量"表示解引用，如`let &b = a;`

```rust
fn main() {
    let a = &666; // a是&i32类型，是一个引用类型
    // 解引用
    let &b: i32 = a; // 在&变量绑定上表示解地址，此时b是i32类型
    println!("a: {} b: {}", a, b); // 666 
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
    let data = vec![1, 2, 3, 4];
    let data1 = &data;
    let data2 = &data;
    // 值的地址是什么？引用的地址又是什么？
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
    println!("addr of value: {:p}, addr of ref: {:p}, {:p}", data, &data, *&data);
    data.iter().fold(0, |acc, x| acc + x)
}
```



# 总结

## 不同情况下的&含义

1. 在表达式上，表示的是借用，其结果是得到一个引用类型

   > 如 let a = &123; 此时&123表示借用123，其得到一个&i32的引用类型，所以a的类型为&i32

2. 在变量绑定上，表示解引用操作，与*类似

   > 如
   >
   > let a = &123;
   >
   > let &b = a; // 此时b的类型是i32，&b表示解引用，所以b的值是123；也等价于 let b = *a;

3. 在类型声明上，表示引用类型，即&T

   > 如 let a: &i32 = &123; // 此时&i32就是类型声明

   

## 不同情况下的ref

1. 在变量绑定上，表示引用类型。

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

