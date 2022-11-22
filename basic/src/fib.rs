// 例子：使用“if和loop循环、for循环、while循环”这3种循环实现斐波拉契数列

// if和loop循环的例子
fn fib_loop(n: u8) -> u8 {
    let mut a = 1; // mut表示可变
    let mut b = 1;

    let mut i = 2u8;

    loop {
        let c = a + b;
        a = b;
        b = c;
        i += 1;

        println!("loop next val is {}", b);

        if i >= n {
            break;
        }
    }

    b
}

// while循环
fn fib_while(n: u8) -> u8 {
    // 在同一行声明多个变量
    let (mut a, mut b, mut i) = (1, 1, 2);
    
    while i < n {
        let c = a + b;
        a = b;
        b = c;
        i += 1;

        println!("while next val is {}", b);
    }

    b
}

// for 循环
// for 循环实际只是一个语法糖，编译器会将其展开使用 loop 循环对迭代器进行循环访问，直至返回 None
fn fib_for(n: u8) -> u8 {
    let (mut a, mut b) = (1, 1);

    // 2..n 表示 2<= x < n
    for _i in 2..n {
        let c = a + b;
        a = b;
        b = c;

        println!("for next val is {}", b);
    }

    return b
}

fn main() {
    let n = 10;
    let rst1 = fib_loop(n);
    println!("rst1 {}", rst1);
    
    let rst2 = fib_while(n);
    println!("rst2 {}", rst2);

      
    let rst3 = fib_for(n);
    println!("rst3 {}", rst3);
}
