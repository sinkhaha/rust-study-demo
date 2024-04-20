# 1 结构体

`Rust` 中结构体有3类：具名字段结构体、元组型结构体、单元型结构体

> 结构体可以简单的看成是类



`Rust` 中约定

* 所有类型（包括结构体）的名称命名都是大驼峰格式，即首字母大写
* 字段和方法的命名都是蛇形格式，即单词小写，单词之间下划线分隔



## 1.1 具名字段结构体

具名字段结构体：结构体里面有定义具体名字的字段，如下

```rust
struct GrayscaleMap { // GrayscaleMap是结构体类型名称， 有两个具体名字的字段
  pixels: Vec<u8>, // 名字为pixels，类型是Vec<u8>
  size: (usize, usize),
}

let width = 1024;
let height = 576;

// 初始化结构体
let image = GrayscaleMap {
  pixels: vec![0; whidth, * height],
  size: (width, hegiht)
}

// 访问结构体，用 . 运算符
assert_eq!(image.size, (1024, 576));
```

结构体默认情况下是私有的，仅在声明它们的模块及其子模块中可见。要想结构体在其他模块外部可见，可以在 `struct` 前面加上 `pub`。如

```rust
pub struct GrayscaleMap { 
  pub pixels: Vec<u8>, 
  pub size: (usize, usize)
}
```

结构体中的每个字段默认下也是私有的，也可以使用 `pub` 使得字段可见



## 1.2 元组型结构体

元组型结构体：即结构体是一个元组类型，如下

```rust
// 定义元组结构体
struct Bounds(usize, usize);

// 使用元组结构体
let image_bounds = Bounds(1024, 768);

// 像元组一样使用访问它们
assert_eq!(image_bounds.0 * image_bounds.1, 786432);
```

表达式 `Bounds(1024, 768)` 看起来像是一个函数，实际上它就是函数，即定义这种类型时也隐式定义了一个函数

```rust
fn Bounds(elem0: usize, elem1: usize) -> Bounds { ... }
```



## 1.3 单元型结构体

单元型结构体是声明了一个完全没有元素的结构体类型

```rust
// 定义一个没有元素的结构体
struct Onesuch;

// 使用
let o = Onesuch;
```

这种类型的值不会占内存，很像单元类型 `()`



# 2 结构体布局

在内存中，具名字段型结构体 和 元组型结构体是一样的：值（可能是混合类型）的集合以特定方式在内存中布局。如下结构体

```rust
struct GrayscaleMap {
  pixels: Vec<u8>,
  size: (usize, usize)
}
```

其在内存中的布局如下图：

![](https://sink-blog-pic.oss-cn-shenzhen.aliyuncs.com/img/rust-chengxusheji/%E7%BB%93%E6%9E%84%E4%BD%93%E5%86%85%E5%AD%98.drawio.png)



* `Rust` 不保证它会如何在内存中对结构体的字段或元素进行排序，图中仅是一种可能的安排

  > 可以使用 `#[repr(C)]` 属性要求 `Rust` 以兼容 `C` 和 `C++` 的方式对结构体进行布局

* 但是 `Rust` 会保证将字段的值直接存储在结构体本身的内存块中，这里 `pixels` 值和 `size` 值直接嵌入 `GrayscaleMap` 的值中。只有由 `pixels` 向量拥有的在堆上分配的缓冲区才会留在它自己的块中



# 3 用 impl 为结构体定义方法

**impl 为结构体定义方法**

可以在自己定义的结构体类型上定义方法。但是不能直接写在结构体中，要写在单独的 `impl` 块中，`impl` 块只是 `fn` 定义的结合体，每个定义都会成为块顶部命名的结构体类型上的一个方法。一个类型可以有很多独立的 `impl` 块，但是它们必须在定义该类型的同一个 `crate` 中。例如

```rust
pub struct Queue {
    older: Vec<char>, // 较旧的元素，最早进来的在后面
    younger: Vec<char>, // 较新的元素，最后进来的在后面
}
```

接着为 `Queue` 结构体定义方法

```rust
impl Queue {
    pub fn new() -> Queue { // 参数没有self
       Queue { older: Vec::new(), younger: Vec::new() }
    }
  
    /// 把字符推入队列的最后
    pub fn push(&mut self, c: char) { // 因为要修改Queue，所以要接收&mut self参数，表示借用所传入Queue对象的可变引用。如果不需要修改self，可以定义为共享引用，即&self
        self.younger.push(c);
    }

    /// 从队列的前面弹出一个字符，如果确实有要弹出的字符，就返回 Some(c)，如果队列为空，则返回None
    pub fn pop(&mut self) -> Option<char> { // 因为要修改Queue，所以要接收&mut self参数
        if self.older.is_empty() {
            if self.younger.is_empty() {
                return None;
            }

            // 将younger中的元素移到older中
            use std::mem::swap;
            swap(&mut self.older, &mut self.younger);
            self.older.reverse();
        }

        // 现在older能保证有值了，Vec的pop方法已经返回一个Option，所以可以放心使用
        self.older.pop()
    }
}

fn main() {
    let mut q = Queue {
        older: Vec::new(),
        younger: Vec::new(),
    };
    q.push('0'); // 会借入对q的可变引用
    q.push('1');

    assert_eq!(q.pop(), Some('0'));
}
```

1. `Rust` 方法中，必须显示使用 `self` 来调用结构体的值。
2. `Rust` 会将调用关联函数的结构体值作为第一个参数传给方法，该参数必须具有特殊名称 `self`，`self` 的类型显示就是结构体的类型 或 对该类型的引用，其实
   *  `self` 是  `self: Queue` 的简写
   * `&self` 是 `self: &Queue`  的简写
   * `&mut self`  是 `self: &mut Queue`的简写

如果一个方法要获取 `self` 的所有权，就可以通过值来获取 `selft`，如

```rust
impl Queue {
    pub fn split(self) -> (Vec<char>, Vec<char>) {
        (self.older, self.younger)
    }
}
```



**以 Box、Rc 或 Arc 形式传入 self**

结构体方法的 `self` 参数也可以是 `Box<Self>` 类型、`Rc<Self>` 类型 或 `Arc<Self>`类型，这种方法只能在给定的指针类型值上调用，调用该方法会将指针的所有权传给它。



例如：如果某些方法需要获取指向 `Self` 的指针的所有权，并且其调用者手头恰好有这样一个指针，那么 `Rust` 也允许将它作为方法的 `self` 参数传入，但是必须明确写出 `self` 的类型，就好像它是普通函数一样，如

```rust
impl Node {
   // 明确写出是 Rc 类型
   fn append_to(self: Rc<Self>, parent: &mut Node) {
     parent.children.push(self);
  }
}
```



但是对于方法调用和字段访问，结构体方法大多情况只需要是 `&self` 和`&mut self` 类型就行，因为 `Rust` 会自动从 `Box`、`Rc`、`Arc` 等指针类型中借入引用，例如

```rust
let mut bq: Box<Queue> = Box::new(Queue::new());

bq.push('g');

assert_eq!(bq.pop(), Some('g'));
```

此时 `bq` 是 `Box<Queue>` 类型，它也可以调用 `push` 方法，此时 `push` 方法的第一个参数是可变引用类型 `&mut self`，即 `&mut Queue`，此时 `Rust` 在调用期间从 `Box` 借入了 `&mut Queue`，所以也可以调用该方法



**关联函数 和 自由函数**

* 关联函数：指 `impl` 块中定义的函数，因为它们是与特定类型(结构体)相关联的
* 自由函数：指未定义在 `impl` 块中的函数，与关联函数相对



**类型关联函数**

类型关联函数：`impl` 块中还可以定义根本不以 `self` 为参数的函数，它们在 `impl` 中，但它们不是方法，因为不接受 `self` 参数，如

```rust
impl Queue {
    pub fn new() -> Queue { // 参数没有self
       Queue { older: Vec::new(), younger: Vec::new() }
  }
}
```

要使用类型关联函数，直接 `Queue::new` ，即 `"类型名称 + 双冒号 + 函数名称"`。

> 注意，`new` 在 `Rust` 中并不是关键字



**关联常量**

关联常量：表示与类型关联的值，而不是与该类型的特定实例关联起来的值。关联常量是常量，如

```rust
pub struct Vector2 {
    x: f32,
    y: f32,
}

impl Vector2 {
    // 定义3个关联常量
    const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };
    const UNIT: Vector2 = Vector2 { x: 1.0, y: 0.0 };
    const ID: u32 = 18; // 关联常量的类型不必是其所关联的类型
}

// 使用，直接用 “类型::关联常量” 进行使用
let scaled = Vector2::UNIT.scaled_by(2.0);
```



# 4 泛型结构体

`Rust` 结构体是可以泛型的，如定义一个 `Queue`，可以保存任意类型的值

```rust
pub struct Queue<T> {
    older: Vec<T>,
    younger: Vec<T>
}
```

使用尖括号`<>` 中的类型名 `T` 称作类型参数。`Queue<T>` 读作对于任意类型 `T `



泛型结构体的 `impl` 块也要标明泛型，如

```rust
impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
       Queue { older: Vec::new(), younger: Vec::new() }
    }
  
    pub fn push(&mut self, t: T) { 
        self.younger.push(c);
    }
  
    pub fn is_empty(&self) -> bool { 
        self.older.is_empty() && self.younger.is_empty()
    }

    ...
}
```

`impl<T> Queue<T>` 读作对于任意类型 `T`，这里有一些在 `Queue<T>`上可用的关联函数



`Self` 关键字:  `Self` 代表当前的类型,每个 `impl` 块，无论是不是泛型，都会将特殊类型的参数 `Self` 定义为我们要为其添加方法的任意类型，对上面的代码来说，`Self` 其实就是 `Queue<T>`类型，所以也可以写成

```rust
pub fn new() -> Self {
   Queue { older: Vec::new(), younger: Vec::new() }
}
```



在调用关联函数时，可以让 `Rust` 帮你推断出来

```rust
let mut q = Queue::new();
q.push(0.74); // 显然是 Queue<f64>
```

也可以使用 `::<> `（比目鱼）表示法显示的提供类型参数

```rust
let mut q = Queue::<f64>new();
```



**带生命周期参数的泛型结构体**

如果结构体类型包含引用，则必须为这些引用的生命周期命名。

```rust
struct Extrema<'a> {
    greatest: &'a i32,
    least: &'a i32
}
```

`Extrema<'a>` 表示任意生命周期 `'a` ，都可以创建一个 `Extrema<'a>` 来持有对该生命周期的引用。



**带常量参数的泛型结构体**

泛型结构体也可以接受 `常量值` 作为参数。常量泛型参数可以是任意整数类型、`char` 或 `bool`。不允许使用浮点数、枚举和其他类型。如

```rust
struct Polynomial<const N: usize> {
    coefficients: [f64, N]
}
```

* `<const N: usize>` 子句表示 `Polynomial` 类型需要一个 `usize` 值作为它的泛型参数，以此来决定要存储多少个系数。

也可以在类型的关联函数中使用参数 `N`，如

```rust
impl<const N: usize> Polynomial<N> {
    fn new(coefficients: [f64, N]) -> Polynomial<N> {
      Polynomial { coefficients }
    }
     
    fn eval(&self, x: f64) -> f64 {
        let mut sum = 0.0;
        for i in (0..N).rev() {
          sum = self.coefficients[i] + x * sum;
        }
        
         sum
    }
}
```

如果结构体还接受其他种类的泛型参数，则**生命周期必须排在第一位**，然后是类型，接下来是任何 `const`值。如

```rust
struct LumpOfReferences<'a, T, const N: usize> {
    the_lump: [&'a; N]
}
```



# 5 让结构体实现特型

如下结构体 `Point` 是不可复制 或 克隆的，不能用 `println!("{:?}", point)` 打印，而且不支持 == 和 ！= 运算符

```rust
struct Point {
    x: f64,
    y: f64
}
```

可以将 `#[derive]` 属性添加一些特型到结构体上，这样结构体会自动实现它们，如

```rust
#derive[Copy, Clone, Debug, PartialEq]
struct Point {
    x: f64,
    y: f64
}
```

此时 `Point` 结构体就实现了 `Copy`、`Clone`、`Debug`、`PartialEq` 等特型



# 6 参考

* [Rust程序设计（第2版）](https://book.douban.com/subject/36547630/)

