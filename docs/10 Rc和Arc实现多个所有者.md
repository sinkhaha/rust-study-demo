# 1 Rc 和 Arc

`Rust` 的所有权机制规定了一个值只能有一个所有者。当出现不符合所有权机制的情况时要怎么处理，如下面场景

* 在图数据结构场景中：一个有向无环图（`DAG`）中，某个节点有多个节点指向它，该节点必须直到没有多个节点指向它时，才应该被释放

* 在多线程场景中：多个线程持有同一块共享内存，但是因为 `Rust` 的安全机制，无法同时获得该数据的可变引用

以上问题在程序运行过程中才会遇到，在编译期，所有权的静态检查无法处理它们，所以 `Rust`  提供了**运行时的动态检查**，来满足特殊场景下的需求。为了解决以上问题，`Rust` 引入了引用计数的方式（即 `Rc` 和 `Arc` 指针）：引用计数允许一个数据资源在同一时刻拥有多个所有者



## 1.1 Rc

`Rc（Reference counter）`：是一个引用计数的智能指针。对于任意类型 `T`，`Rc<T>` 值是指向附带引用计数的在堆上分配的 `T` 型指针。例如：


```rust
use std::rc::Rc;

fn main() {
    let s: Rc<String> = Rc::new("hello".to_string());
    let t: Rc<String> = s.clone();
    let u: Rc<String> = s.clone();
}
```

* 上面 `s、t、u` 这 3 个 `Rc<String>` 都指向了同一个内存块，其中包含引用计数和 `String` 本身的空间

* 使用 `clone` 方法克隆一个 `Rc<T>` 并不会复制 `T`，它只会创建另一个指向它的指针并递增引用计数



**特点**

* `Rc` 可以对某个值创建引用计数，使这个值拥有多个所有者
* `Rc` 使用的是线程不安全的引用计数器，只能用于单线程
* `Rc` 是一个**只读**的引用计数器，`Rc` 指针拥有的值是不可变的，因为 `Rc` 指针的引用目标通常是可以共享的，所以不能是可变的，也就无法拿到 `Rc` 结构内部数据的可变引用来修改这个数据。如

```rust
use std::rc::Rc;

fn main() {
    let s: Rc<String> = Rc::new("hello".to_string());
    s.push_str(" world"); // 在文本末尾添加字符串，这个会报错
}
```

* `Rc` 绕过了编译器的静态检查，会把对应的数据结构创建在堆上



**增加计数和减少计数**

* 增加引用计数：对一个 `Rc` 结构进行 `clone()`，会产生一个指针指向该值，不会将其内部的数据复制，只会增加引用计数

* 减少引用计数：当一个 `Rc` 结构离开作用域被 `drop()` 时，也只会减少其引用计数，直到 `Rc` 指针全部被销毁，即引用计数为 0 时，才会真正清除值对应的内存

  

使用引用计数管理内存有一个问题：两个引用计数的值是相互指向的，即循环引用计数。可以使用弱引用指针 `std::rc::Weak` 来避免建立 `Rc` 指针循环。



**例子**

```rust
use std::rc::Rc;

fn main() {
    let a = Rc::new(1);
    let b = a.clone(); // 不会复制a，只会创建一个指向它的指针并递增引用计数
    let c = a.clone();

    // 断言a的引用计数为3
    assert_eq!(3, Rc::strong_count(&a));
    assert_eq!(Rc::strong_count(&a), Rc::strong_count(&b));
    assert_eq!(Rc::strong_count(&b), Rc::strong_count(&c));

    println!("count after create a: {}", Rc::strong_count(&a)); // 3
    println!("count after create b: {}", Rc::strong_count(&b)); // 3
    println!("count after create c: {}", Rc::strong_count(&c)); // 3
}
```

1. 可以直接在 `Rc<i32>` 上使用 `i32` 的任何常用方法
2. 因为对于任意类型 `T`，`Rc<T>` 值是指向附带引用计数的在堆上分配的 `T` 型指针。`a、b、c` 这三个 `Rc<i32> `智能指针指向堆上同一个内存块，其中包含引用计数和 `i32` 本身的空间，即共享堆上的数据，也说明堆上的数据有 3 个所有者，它们的引用计数都是一样的
3. 在代码执行结束时，智能指针 `c` 先 `drop`，引用计数变成 2，接着 `b` 被 `drop`，引用计数变成 1，最后 `a` 被`drop` ，引用计数归 0，此时堆上内存被释放



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/Rc.png)





## 1.2 Arc

`Arc（Atomic reference counter）`：也是一个原子引用计数的智能指针，`Arc` 和 `Rc` 大部分东西是相同的



**特点**

1. `Arc` 是线程安全的引用计数，在多个线程中可安全的共享数据

2. `Arc` 内部的引用计数使用了 `Atomic Usize` ，而非普通的 `usize`

   > `Atomic Usize` 是 `usize` 的原子类型，它使用了 `CPU` 的特殊指令，来保证多线程下的安全。
   >
   > 虽然原子化或者加锁可以带来线程安全，但是会伴随着性能损耗，所以 `Arc` 的性能不如 `Rc`



## 1.3 Rc 和 Arc 的对比

**相同点**

1. 可以对某个值创建引用计数，使这个值拥有多个所有者

2. 它们都是只读的引用计数器，不能获取到指向数据的可变引用，如果要获取指向数据的可变引用，需要配合其他数据结构一起使用。`Rc` 配合内部可变性 `RefCell`，`Arc` 配合互斥锁 `Mutex` 或读写锁 `RwLock`

3. 绕过了编译器的静态检查

4. 两者的操作 `api` 相同

   

**不同点**

1. 线程安全：`Rc` 只能用于单线程，更新引用计数是线程不安全的，性能更好更快。`Arc` 用于多线程，是线程安全的，可以安全的在多线程之间共享数据，性能不如 `Rc`
2. 所在模块不同：`Rc` 定义在 `use std::rc::Rc` 模块中，`Arc` 定义在 `use std::sync::Arc` 模块中



## 1.4 运行时检查机制: Box::leak

**问题：`Rc` 是怎么产生在堆上的？并且为什么这段堆内存不受栈内存生命周期的控制呢？**

在所有权模型下，堆内存的生命周期，和创建它的栈内存的生命周期保持一致。如果按照单一所有权模型，`Rust `是无法处理 `Rc` 这样的引用计数的。



**Box::leak() 机制**

1. `Rust` 提供一种机制：`Box::leak()`，可以创建不受栈内存控制的堆内存，从而绕过编译时的所有权规则

   >`Box` 是 `Rust` 下的智能指针，它可以强制把任何数据结构创建在堆上，然后在栈上放一个指针指向这个数据结构，但此时堆内存的生命周期仍然是受控的，跟栈上的指针一致

2. `Box::leak()` 创建的对象，从堆内存上泄漏出去，不受栈内存控制，是一个自由的、生命周期可以大到和整个进程的生命周期一致的对象

   > 相当于主动撕开了一个口子，允许内存泄漏。注意，在 `C/C++` 下，通过 `malloc` 分配的每一片堆内存，类似 `Rust` 下的 `Box::leak()`

如下图：



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/Box.png)





有了 `Box::leak()`，就可以跳出 `Rust` 编译器的静态检查，保证 `Rc` 指向的堆内存，有最大的生命周期，然后我们再通过引用计数，在合适的时机，结束这段内存的生命周期。



**`Rust` 是如何进行所有权的静态检查和动态检查**

* 静态检查，靠编译器保证代码符合所有权规则

  > 编译时，处理大部分使用场景，保证安全性和效率

* 动态检查，通过 `Box::leak` 让堆内存拥有不受限的生命周期，然后在运行过程中，通过对引用计数的检查，保证这样的堆内存最终会得到释放

  > 运行时，处理无法在编译时处理的场景，会牺牲一部分效率，提高灵活性



# 2 内部可变性

## 2.1 内部可变性和外部可变性

1. 外部可变性：即用 `let mut` 显式地声明一个可变的值，或者用 `&mut` 声明一个可变引用时，编译器可以在编译时进行严格地检查，保证只有可变的值或者可变的引用，才能修改值内部的数据

2. 内部可变性：有时希望能够绕开编译时的检查，对**并未声明成 mut 的值或者引用**也进行修改。也就是，在编译器眼里，值是只读的，但是在运行时，这个值可以得到可变借用，从而修改值内部的数据

|            | 使用方法                   | 所有权的检查                               |
| ---------- | -------------------------- | ------------------------------------------ |
| 外部可变性 | `let mut` 或者 `&mut`      | 发生在编译时，如果不符合规则，产生编译错误 |
| 内部可变性 | 使用 `Cell` 或者 `RefCell` | 发生在运行时，如果不符合规则，产生 `panic` |



## 2.2 内部可变性 RefCell

**为什么要有内部可变性**

* 因为`Rc` 是一个只读的引用计数器，所以无法拿到 `Rc` 结构内部数据的可变引用，即无法通过它来修改数据。但`Rc` 可以配合其他数据结构一起使用，如内部可变性的 `RefCell` 类型
* 而 `Arc` 则配合使用互斥锁 `Mutex` 或读写锁 `RwLock`



`RefCell` 绕过了 `Rust` 编译器的**静态检查**，允许在运行时，获取对某个只读数据的可变借用；注意 `RefCell` 也是线程不安全的。可以通过 `RefCell` 的 `borrow_mut()` 方法获得一个 `RefCell` 数据的可变引用，例如：

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(1); // 初始值为1
  
    // 这里大括号的作用：因为根据所有权规则，在同一个作用域下，我们不能同时有活跃的可变借用和不可变借用。通过这对花括号，我们明确地缩小了可变借用的生命周期，不至于和后续的不可变借用冲突
   // 如果去掉大括号：可以编译通过，但是运行时会报错'already mutably borrowed: BorrowError'。可以看到，所有权的借用规则在此依旧有效，只不过它在`运行时检测`。
    {
        // 通过使用RefCell的borrow_mut()方法，来获得一个可变的内部引用，然后通过解引用对它做加 1 的操作
        let mut v = data.borrow_mut();
        *v += 1;
    }
  
    // 通过 RefCell 的 borrow() 方法，获得一个不可变的内部引用
    println!("data: {:?}", data.borrow()); // data: 2
}

```



通过 `RefCell` 获取 `Rc` 的可变引用，例如

```rust
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let a: Rc<RefCell<i32>> = Rc::new(RefCell::new(1));
    let b: Rc<RefCell<i32>> = a.clone();

    // 断言a的引用计数为2
    assert_eq!(2, Rc::strong_count(&a));

    let c = b.borrow_mut();
    println!("c: {}", c); // 1
}
```



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/RefCell.drawio.png)



## 2.3 Rc 和 Arc 使用内部可变性的语法

| 访问方式           | 数据             | 不可变引用   | 可变引用         |
| ------------------ | ---------------- | ------------ | :--------------- |
| 单一所有权         | `T`              | `&T`         | `&mut T`         |
|                    |                  |              |                  |
| 共享所有权: 单线程 | `Rc<T>`          | `&Rc<T>`     | 无法得到可变借用 |
|                    | `Rc<RefCell<T>>` | `v.borrow()` | `v.borrow_mut()` |
|                    |                  |              |                  |
| 共享所有权: 多线程 | `Arc<T>`         | `&Arc<T>`    | 无法得到可变借用 |
|                    | `Arc<Mutex<T>>`  | `v.lock()`   | `v.lock()`       |
|                    | `Arc<RwLock<T>>` | `v.read()`   | `v.write()`      |



# 3 实践例子

## 3.1 例1：实现不可修改的 DAG

假设每个节点 `Node` 就只包含 `id` 和指向下游（`downstream`）的指针，因为 DAG 中的一个节点可能被多个其它节点指向，所以使用 `Rc<Node>` 来表述它；一个节点可能没有下游节点，所以使用用 `Option<Rc<Node>>` 来表述它



![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/node_source/Rc-dag.drawio.png)



建立这样一个 `DAG`(有向无环图)，需要为 `Node` 提供以下方法：

* `new()`：建立一个新的 `Node`
* `update_downstream()`：设置 `Node` 的 `downstream`
* `get_downstream()`：`clone` 一份 `Node `里的 `downstream`



**代码如下：**

```rust
use std::rc::Rc;

#[derive(Debug)]
struct Node {
    id: usize,
    downstream: Option<Rc<Node>>,
}

impl Node {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            downstream: None,
        }
    }

    pub fn update_downstream(&mut self, downstream: Rc<Node>) {
        self.downstream = Some(downstream);
    }

    pub fn get_downstream(&self) -> Option<Rc<Node>> {
        self.downstream.as_ref().map(|v| v.clone())
    }
}

fn main() {
    let mut node1 = Node::new(1);
    let mut node2 = Node::new(2);
    let mut node3 = Node::new(3);
    let node4 = Node::new(4);
    node3.update_downstream(Rc::new(node4));

    node1.update_downstream(Rc::new(node3));
    node2.update_downstream(node1.get_downstream().unwrap());
    
    println!("node1: {:?}, node2: {:?}", node1, node2);
}
```



## 3.2 例2：RefCell 实现可修改的 DAG

要想 `DAG` 可以正常修改，数据结构的 `downstream` 需要 `Rc` 内部嵌套一个 `RefCell`，来获得数据的可变借用，即 `Rc<RefCell<T>>`，代码如下

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Node {
    id: usize,
    // 使用 Rc<RefCell<T>> 让节点可以被修改
    downstream: Option<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            downstream: None,
        }
    }

    pub fn update_downstream(&mut self, downstream: Rc<RefCell<Node>>) {
        self.downstream = Some(downstream);
    }

    pub fn get_downstream(&self) -> Option<Rc<RefCell<Node>>> {
        self.downstream.as_ref().map(|v| v.clone())
    }
}

fn main() {
    let mut node1 = Node::new(1);
    let mut node2 = Node::new(2);
    let mut node3 = Node::new(3);
    let node4 = Node::new(4);

    node3.update_downstream(Rc::new(RefCell::new(node4)));
    node1.update_downstream(Rc::new(RefCell::new(node3)));
    node2.update_downstream(node1.get_downstream().unwrap());
    println!("node1: {:?}, node2: {:?}", node1, node2);

    let node5 = Node::new(5);
    let node3 = node1.get_downstream().unwrap();
    // 获得可变引用，来修改 downstream
    node3.borrow_mut().downstream = Some(Rc::new(RefCell::new(node5)));

    println!("node1: {:?}, node2: {:?}", node1, node2);
}
```



# 4 参考

* [陈天 · Rust 编程第一课-所有权：一个值可以有多个所有者么](https://time.geekbang.org/column/article/416722)
* [Rust语言圣经(Rust Course)-Rc与Arc实现1vN所有权控制](https://course.rs/advance/smart-pointer/rc-arc.html)

