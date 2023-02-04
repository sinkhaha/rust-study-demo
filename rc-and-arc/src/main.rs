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

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let a = Rc::new(RefCell::new(1));
    let b = a.clone();

    // 断言a的引用计数为2
    assert_eq!(2, Rc::strong_count(&a));

    let c = b.borrow_mut();
    println!("c: {}", c); // 1
}
