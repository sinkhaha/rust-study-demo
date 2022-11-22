// 函数

// 第2个参数是一个函数类型，它有一个i32类型的形参，它的返回值为i32
fn apply(value: i32, f: fn(i32) -> i32) -> i32 {
    f(value) // 注意不能加分号，不然是返回unit
}

// 面积
fn square(value: i32) -> i32 {
    value * value
}

// 体积
fn cube(value: i32) -> i32 {
    value * value * value
}

fn main() {
    println!("面积 {}", apply(2, square));
    println!("体积 {}", apply(2, cube));
}
