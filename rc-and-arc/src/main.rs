// use std::rc::Rc;

// fn main() {
//     let a = Rc::new(1);
//     let b = a.clone();
//     let c = a.clone();

//     // 断言a的引用计数为3
//     assert_eq!(3, Rc::strong_count(&a));
//     assert_eq!(Rc::strong_count(&a), Rc::strong_count(&b));
//     assert_eq!(Rc::strong_count(&b), Rc::strong_count(&c));

//     println!("count after create a: {}", Rc::strong_count(&a)); // 3
//     println!("count after create b: {}", Rc::strong_count(&b)); // 3
//     println!("count after create c: {}", Rc::strong_count(&c)); // 3
// }


#[derive(Debug, Copy, Clone)]
struct Myt {
   a: i32
}

fn main() {
    let rst = Myt { a: 32 };
    let rst1 = rst;

    drop(rst1.a);

    // std::mem::drop(rst1);

    println!("a: {} b: {}", rst.a, rst1.a);

    let v = vec![1, 2, 3];

    // drop(v); 
    println!("a: {:?} ", v);

    // let a: i32 = 666;

    // let b = a;

    // 解指针，此时的 “*表达式” 类似C++的解指针，即拿到b存的地址指向的值
    // println!("a: {} b: {}", a, b); // a: 666 b: 666
}
