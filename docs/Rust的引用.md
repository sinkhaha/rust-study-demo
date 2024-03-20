# 引用

**什么是引用**

1. 引用其实就是指针，是指向特定类型数据的一个指针 或 胖指针，胖指针是有额外元数据的指针

2. 引用可分为共享引用和可变引用

   > 表达式 `&x` 会生成一个对 `x` 的引用（共享引用），把创建对某个值的引用的操作称为 **借用** 那个值。如 `&123` 即是一个引用，可以称为 借用了123， `&123` 表示的是一个指向数值 123 的一个指针，引用 `&123` 的类型是 `&i32`，`&i32` 表示对 `i32` 的引用

3. 引用的值是内存地址，引用可以指向内存中任何地方的值，不仅仅是栈上的

3. 引用能在不影响值的所有权的情况下访问值。因为引用是 非拥有型指针，这种指针对引用目标的生命周期没有影响，要注意**任何引用的生命周期不能超出其指向的值**。还有一种是拥有型指针，如`Box<T>`、String值和Vec值内部的指针，当拥有者被丢弃时，它的引用目标也会随之消失




**例子**

当打印一个引用，结果就是引用的值，这个值表示指向的变量的内存地址。如果要打印一个引用本身的地址，就要对引用再加上一层引用，如

```rust
fn main() {
    let a = &11;
    let b = a;
    let c = &a; // c是"引用a"的引用，存的是“引用a”的地址

    //a=0x1061f958c b=0x1061f958c c=0x1061f958c d=$0x7ff7b9d406d8 前3个是一样的地址
    println!("a={:p} b={:p} c={:p} d=${:p}", a, b, &11, c);            
}
```

例子中打印了“引用a”的值，结果就是变量11的内存地址，即 0x106c2b58c。最后打印a变量本身的地址，要对引用再加上一层引用，即 &a，因为a是一个引用，它指向a的内存地址，即$0x7ff7b930e6d8



# 共享引用

表达式 &e 会产生对 e值的共享引用，如果 e 的类型为T，那么 &e 的类型就是 &T。



## 引用类型

引用类型：是一种数据类型，它所保存的值是一个引用。引用类型可以分为 引用类型 和 可变引用类型。



**引用类型的表示方式**

1. 用 `&T` 表示类型 T 的引用类型，是一个对于类型为 T的数据的共享引用，如 `&i32`

2. 也可以用`ref`表示

   * ref 用在变量绑定上，也是指引用类型

   * 在模式匹配时，用ref关键字也是表示引用类型



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

例3：

```rust
fn main() {
    // 引用类型声明时可以不赋值
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

> `let ref a` 表示声明了一个引用类型，它只能绑定到某次借用动作上，`&1` 即借用1

例4：

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





## 共享引用是只读访问

1. 同一个值，可以同时被多个共享引用拥有

2. 共享引用是只读访问，在共享引用的整个生命周期中，它引用的目标值是只读状态，不能修改其目标值

   > 例如当目标值存在共享引用时，不能对目标值赋值、不能将目标值移动到别处、也不能存在对该目标值的有效可变引用

3. 可以从共享引用中重新借入共享引用

   

例1:

```rust
fn main() {
    let mut x = 10;
    let r1 = &x;
    let r2 = &x; // 符合第1点，正确：允许多个共享借用，此时r1和r2都借用了x

    x += 10; //符合第2点，错误，不能赋值给x，因为它已经被借出，不能修改

    let m = &mut x; // 符合第2点，错误，不能把x借入为可变引用，因为它覆盖在已借出的不可变引用的生命周期内

    println!("{}, {}, {}", r1, r2, m); // 这些引用在这里使用的，所以它们的生命周期至少要存续这么长
}
```

例2：

```rust
let mut w: (i32, i32) = (107, 109);
let r: &(i32, i32) = &w;
let r0: &i32 = &r.0; // 正确，把共享引用重新借入为共享引用
let m1 = &mut r.1; // 错误，不能把共享引用重新借入为可变引用
println!("{}", r0);
```

例3：如下代码会出现悬空指针，因为r的生命周期内发生了移动向量的操作

```rust
 let v = vec![4, 8, 19, 27, 34, 10];
 let r = &v;
 let aside = v; // 把向量移给aside，v是未初始化状态
 r[0]; // 错误，这里v是未初始化状态，r成了悬空指针
```



例4：分析以下例子输出什么

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



# 可变引用

表达式 &mut e 会产生对 e值的可变引用，如果 e的类型为T，可以将其类型写成&mut T



## 可变引用类型

**可变引用类型的表示方式**

1. 用`&mut T` 表示类型T的可变引用类型， 是一个对类型为 T的数据的独占引用，如 `&mut i32`
   * 可变引用才能修改目标值，所以想要通过引用去修改源数据，需要使用`&mut v`来创建可修改源数据 v 的 `可变引用`
   * 想要通过`&mut`引用去修改源数据，要求原变量是可变的，即定义时需要有 `mut` 关键字，如 `let mut n: i32 = 66;` 表示 n 这个值是可修改的
2. 也可以用`ref mut`表示



例1：

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

例5：

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



## 可变引用是独占访问

1. 在一个作用域内，一个值只能存在一个对该值的可变引用

2. 可变引用会独占对该值的访问权，在可变引用的整个生命周期中，只能通过该引用读取或修改目标值

   > 无论是它的引用目标值，还是该引用目标值间接访问的任何目标，都没有任何其他路径可访问该值

3. 可以从可变引用中重新借入可变引用。对可变引用来说，唯一能和自己的生命周期重叠的引用就是从可变引用本身借出的引用

4. 在一个作用域内，可变引用（写）和只读引用（读）是互斥的，不能同时存在

   > 简单来说，就是要注意交叉使用 可变引用 和 只读引用



例1：

```rust
 let mut y = 20;
 let m1 = &mut y;
 let m2 = &mut y; // 错误，符合第1点，不能多次借入可变引用
 let z = y; // 错误，符合第2点，不能使用 y, 因为它涵盖的已借出的可变引用的生命周期内
 println!("{}, {}, {}", m1, m2, z);
```



例2：

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



例3:

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



## **按值传递和按引用传递**

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

// 按引用传递
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



# 解引用

## 什么是解引用

解引用表示解除引用，即**通过引用获取到该引用所指向的原始值**。



**解引用的表示方式**

1. 可以用 * 表示，在引用前面加一个星号，如`*a`（其中a是一个引用）
2. 也可以用 “&绑定变量”表示：在声明变量时，在变量前加上&，即 “&绑定变量”，如`let &b = a;`



例子1:

```rust
fn main() {  
   let a = &666; // a是&i32类型，是一个引用类型
  
   // *a 表示解引用
   let b = *a; // b是i32类型

   println!("a: {} b: {}", a, b); // a: 666 b: 666 
}
```

例子2：rust会自动解多层嵌套引用，如

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

例子3:

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

会按需对其左操作数隐式解引用

1. 使用点运算符`.`时，会隐式地对左操作数隐式解引用 或者 隐式借用对其左操作数的引用（即创建引用，比如取属性值是解引用，调用方式是创建引用

   > 例如 `abc.func()` 有可能会自动转换为`&abc.func()`，反之，`&abc.func()`也有可能会自动转换为`abc.func()`
   
2. 使用比较操作符时，若比较的两边是`相同类型的引用`，则会自动解除引用到它们的值然后比较

   > 例如有引用类型的变量n，那么`n > &30`和`*n > 30`的效果是一样的



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

例4：引用的比较

```rust
fn main() {
    let x = 10;
    let y = 10;

    let rx = &x;
    let ry = &y;

    let rrx = &rx;
    let rry = &ry;

    assert!(rrx <= rry); // 比较引用，这里比较的是引用的目标值相等，并不是比较所占的地址(自身的值)相等
    assert!(rrx == rry);
 
    assert!(rx == rrx); // 不同类型，不能比较，会报错
    assert!(rx == *rrx); // 可以比较
}
```

如果要知道两个引用是否指向同一块内存，可以使用 std::ptr::eq，它会将两者作为地址进行比较

```rust
assert!(!std::ptr::eq(rx, ry)); // 比较所占据的地址（自身的值）不同
```



# 小结

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





# 使用引用

**对引用进行引用**

Rust允许对引用进行引用

```rust
fn main() {
    struct Point {
        x: i32,
        y: i32,
    }

    let point = Point { x: 1000, y: 729 };

    let r: &Point = &point; // r是对point的引用，r本身也是引用类型
    let rr: &&Point = &r; // rr是对 r 的引用，即对引用的引用
    let rrr: &&&Point = &rr; // rrr是 对“引用的引用”的引用
  
    assert_eq!(rrr.y, 729); // 这里rrr.y要遍历3层引用才能取到Point的y字段
}
```



**借用任意表达式结果值的引用**

rust允许借用任何种类的表达式结果的引用

```rust
fn factorial(n: usize) -> usize {
    (1..n + 1).product()
}

fn main() {
    let r: &usize = &factorial(6);
  
    assert_eq!(r + &1009, 1729); // &1009对1009的引用
}
```

例子的`&1009` ，rust会创建一个匿名变量来保存此表达式的值，并让该引用指向它



**引用永不为空**

* rust的引用永远都不会为空。在rust中，如果需要一个值来表示为某个“可能不存在”的事物的引用，可以使用类型 `Option<&T>`，在机器码级别，rust会将None表示为空指针，将Some(r)表示为非零地址(r就是&T型的值)
* 不能将 0 转换成引用



**对切片和特型对象的引用**

rust包括两种胖指针

* 即携带某个值地址的双字值：对切片的引用就是一个胖指针，携带着此切片的起始地址及其长度
* 以及要正确使用该值所需的某些额外信息：rust的另一种胖指针是特型对象，即对实现了指定特型的值的引用。特型对象会携带一个值的地址和指向适用于该值的特型实现的指针



# 引用安全

## **借用局部变量**

不能借用对局部变量的引用，并将其移出变量的作用域，因为引用的生命周期不能超过其引用的值的生命周期

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

rust会尝试为程序中的每个引用类型分配一个生命周期，以表达根据其使用方式应施加的约束。生命周期是rust在编译期虚构的产物。



## **将引用作为函数参数**

在函数中将一个引用存储在全局变量(静态变量)中，如

```rust
fn main() {
    static mut STASH: &i32 = &128; // 静态变量必须初始化

    fn f(p: &'static i32) {
        unsafe {
            STASH = p;
        }
    }
}
```

分析如下

* 静态变量STASH必须初始化
* 可变静态变量本质不是线程安全的，只能在unsafe块中访问可变静态变量
* 注意f函数 p的生命周期是'static ，如果不加这个限制，p的生命周期默认是 'a，因为STASH静态变量会存在程序的整个执行过程中，它的生命周期成为 'static生命周期，它的生命周期比 'a长，所以也要接受有 'static 生命周期的引用，不然STASH就是一个悬空指针



## **把引用传给函数**

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



## **返回引用**

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



## **包含引用的结构体**

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

表达式s = `S { r: &x };` 创建了一个新的S值，并存储到变量s中，变量s的生命周期为 'a，当将 &x 存储在r字段中时，就会将 'a 完全限制在了x的生命周期内部。可以修改为如下

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



## **不同的生命周期参数**

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



## **省略生命周期参数**

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
* [Rust程序设计（第2版）](https://book.douban.com/subject/36547630/)



