// 函数的返回值例子

fn pi() -> f64 {
    3.1415926 // 默认最后一个表达式就是返回值(不加分号)
}

fn not_pi() {
    3.1415926; // 最后一个表达式加了分号，会返回unit
}

fn main() {
    let is_pi = pi();
    let is_unit1 = not_pi();

    let is_unit2 = {
        pi();
    };
    
    // 输出 is_pi:3.1415926 is_unit1:() is_unit2:()
    println!("is_pi:{:?} is_unit1:{:?} is_unit2:{:?}", is_pi, is_unit1, is_unit2);
}
