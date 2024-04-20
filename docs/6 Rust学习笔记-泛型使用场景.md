# 1 泛型参数的 3 种使用场景

## 1.1 延迟绑定

第1种场景是使用泛型参数延迟数据结构的绑定。例如下面的 `HashMap` 数据结构使用了一个泛型参数 `S`

```rust
// 使用了3个泛型，分别是K、V、S，并且泛型S的默认类型是RandomState
struct HashMap<K, V, S = RandomState> {
  base: base::HashMap<K, V, S>,
}
```

并且这个泛型参数有一个缺省值 `RandomState`，指定了泛型参数缺省值的好处：在使用时，可以不必提供泛型参数，直接使用缺省值。这个泛型参数在随后的实现中可以被逐渐约束。



下面以标准库的 `BufReader` 结构为例说明泛型参数的逐步约束。

1、`BufReader` 的定义如下，在定义阶段，没有对 `BufReader` 的泛型参数 `R` 做限制

```rust
pub struct BufReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}
```

2、到了实现阶段，根据不同的需求，在实现时泛型参数 `R` 可以做不同的约束，只需要添加刚好满足实现需要的限制即可

* 不做任何限制的情况：比如 `capacity()`、`buffer() `等不需要使用 `R` 的任何特殊能力

```rust
impl<R> BufReader<R> {
    pub fn capacity(&self) -> usize { ... }
    pub fn buffer(&self) -> &[u8] { ... }
}
```

* `R` 满足 `Read` 约束：在实现 `new()` 时，因为使用了 `Read trait` 里的方法，所以需要明确传进来的 `R` 满足 `Read` 约束

```rust
impl<R: Read> BufReader<R> {
    pub fn new(inner: R) -> BufReader<R> { ... }
    pub fn with_capacity(capacity: usize, inner: R) -> BufReader<R> { ... }
}
```

* `R` 满足 `Debug trait` 约束：在实现 `Debug` 时，也可以要求 `R` 满足 `Debug trait` 的约束

```rust
impl<R> fmt::Debug for BufReader<R>
where
    R: fmt::Debug
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result { ... }
}
```



## 1.2 PhonatomData 提供额外类型

第 2 种使用场景是使用 泛型参数 和 幽灵数据([PhantomData](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)) 提供额外类型，即声明数据结构中不直接使用，但在实现过程中需要用到的类型。



**PhonatomData 的作用**

`PhantomData` 实际上长度为零，是个 `ZST（Zero-Sized Type）`，就像不存在一样，唯一作用就是**类型的标记**。在定义数据结构时，对于额外的、暂时不需要的泛型参数，用 `PhantomData` 来“拥有”它们，这样可以规避编译器的报错。



**例子**

设计一个 `User` 和 `Product` 数据结构，它们都有一个 `u64` 类型的`id`。要求每个数据结构的 `id` 只能和同种类型的 `id` 比较，也就是说如果 `user.id` 和 `product.id` 比较，编译器会直接报错。

1、先用一个自定义的数据结构 `Identifier<T>` 来表示 `id`

```rust
pub struct Identifier<T> {
    inner: u64,
}
```

2、然后，在 `User` 和 `Product` 中，各自用 `Identifier<Self>` 来让 `Identifier` 和自己的类型绑定，达到让不同类型的 `id` 无法比较的目的。如下：

```rust
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Identifier<T> {
    inner: u64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct User {
    id: Identifier<Self>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Product {
    id: Identifier<Self>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_should_not_be_the_same() {
        let user = User::default();
        let product = Product::default();

        // 两个 id 不能比较，因为他们属于不同的类型
        // assert_ne!(user.id, product.id);

        assert_eq!(user.id.inner, product.id.inner);
    }
}
```

3、编译时发现，上面的代码无法编译通过

因为 `Identifier<T>` 在定义时，并没有使用泛型参数 `T`，编译器认为 `T` 是多余的，所以只能把 `T` 删除掉才能编译通过。但是删除掉 `T`，`User` 和 `Product` 的 `id` 虽然可以比较了，但无法实现想要的功能了



可以用 `PhantomData`(幽灵数据)来解决这个问题。`PhantomData` 被广泛用在处理，数据结构定义过程中不需要，但是在实现过程中需要的泛型参数。在使用了 `PhantomData` 后，编译器允许泛型参数 `T` 的存在，最终能比编译通过，修正代码如下：

```rust
use std::marker::PhantomData;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Identifier<T> {
    inner: u64,
    _tag: PhantomData<T>, // 幽灵数据
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct User {
    id: Identifier<Self>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Product {
    id: Identifier<Self>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_should_not_be_the_same() {
        let user = User::default();
        let product = Product::default();

        // 两个 id 不能比较，因为他们属于不同的类型
        // assert_ne!(user.id, product.id);

        assert_eq!(user.id.inner, product.id.inner);
    }
}
```



## 1.3 提供多个实现

第3种使用场景：使用泛型参数让同一个数据结构对同一个 `trait` 可以拥有多个不同的实现



**例子：**

有时对于同一个 `trait`，想要有不同的实现，要怎么处理？比如一个方程，它可以是线性方程，也可以是二次方程，我们希望为不同的类型实现不同 `Iterator`

```rust
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct Equation<IterMethod> {
    current: u32,
    _method: PhantomData<IterMethod>,
}

// 线性增长
#[derive(Debug, Default)]
pub struct Linear;

// 二次增长
#[derive(Debug, Default)]
pub struct Quadratic;

// 实现线性增长的迭代器
impl Iterator for Equation<Linear> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;
        if self.current >= u32::MAX {
            return None;
        }

        Some(self.current)
    }
}

// 实现二次增长的迭代器
impl Iterator for Equation<Quadratic> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;
        if self.current >= u16::MAX as u32 {
            return None;
        }

        Some(self.current * self.current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let mut equation = Equation::<Linear>::default();
        assert_eq!(Some(1), equation.next());
        assert_eq!(Some(2), equation.next());
        assert_eq!(Some(3), equation.next());
    }

    #[test]
    fn test_quadratic() {
        let mut equation = Equation::<Quadratic>::default();
        assert_eq!(Some(1), equation.next());
        assert_eq!(Some(4), equation.next());
        assert_eq!(Some(9), equation.next());
    }
}
```

疑问：为什么不构建两个数据结构 `LinearEquation `和 `QuadraticEquation`，分别实现 `Iterator` ？

答：对于这个演示例子，使用泛型的意义并不大，因为 `Equation` 自身没有很多共享的代码，这里只是为了演示用。但如果 `Equation`，只除了实现 `Iterator` 的逻辑不一样，其它大量的代码都是相同的，并且未来除了一次方程和二次方程，还会支持三次、四次，那么用泛型数据结构来统一相同的逻辑，用泛型参数的具体类型来处理变化的逻辑，就非常有必要了。



# 2 泛型函数的使用技巧

## 2.1 返回值携带泛型参数怎么办

返回值携带泛型参数时怎么办：返回值携带泛型参数时可以采用返回特型对象（ `trait object`） 进行处理。



因为在函数中使用泛型作为返回值时，在实现的时候会很麻烦，很难在函数中正确构造一个返回泛型参数的语句，如下

```rust
// 可以正确编译
pub fn generics_as_return_working(i: u32) -> impl Iterator<Item = u32> {
    std::iter::once(i)
}

// 期待泛型类型，但却返回一个具体类型
pub fn generics_as_return_not_working<T: Iterator<Item = u32>>(i: u32) -> T {
    std::iter::once(i)
}
```

此时可以采用返回 `trait object`，它消除了类型的差异，把所有不同的实现 `Iterator` 的类型都统一到一个相同的 `trait object` 下：

```rust
// 返回 trait object
pub fn trait_object_as_return_working(i: u32) -> Box<dyn Iterator<Item = u32>> {
    Box::new(std::iter::once(i))
}
```

还有一个原因是，`Rust` 目前还不支持在 `trait` 里使用 `impl trait `做返回值，所以在 `trait` 中，要使用 `trait object`，比如

```rust
pub trait ImplTrait {
    // 不允许使用 impl Into<String>
    fn impl_as_return(s: String) -> impl Into<String> {
        s
    }
}
```



## 2.2 复杂的泛型参数的声明

在泛型函数中，有时候泛型参数可以非常复杂，需要一步步做分解。



如下示例：比如泛型参数是一个闭包，闭包返回一个 `Iterator`，`Iterator` 中的 `Item` 又有某个约束

```rust
pub fn comsume_iterator<F, Iter,  T>(mut f: F)
where
    F: FnMut(i32) -> Iter, // F 是一个闭包，接受 i32，返回 Iter 类型
    Iter: Iterator<Item = T>, // Iter 是一个 Iterator，Item 是 T 类型
    T: std::fmt::Debug, // T 实现了 Debug trait
{
    // 根据 F 的类型，f(10) 返回 iterator，所以可以用 for 循环
    for item in f(10) {
        println!("{:?}", item); // item 实现了 Debug trait，所以可以用 {:?} 打印
    }
}
```

针对这种复杂的泛型参数，可一步步分解，这样就可以理解它的实质，分解如下

1. 参数 `F` 是一个闭包，接受 `i32`，返回 `Iter` 类型

2. 参数 `Iter` 是一个 `Iterator`，`Item` 是 `T` 类型

3. 参数 `T` 是一个实现了 `Debug trait` 的类型

   

这么分解下来理解后，就可以写出合适的测试示例来测试这个方法了，如

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_iterator() {
        // 不会 panic 或者出错
        comsume_iterator(|i| (0..i).into_iter())
    }
}
```



# 3 参考

[陈天 · Rust 编程第一课](https://time.geekbang.org/column/article/427082)_

