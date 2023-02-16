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

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

fn print(v: impl Into<IpAddr>) {
    println!("{:?}", v.into());
}

fn main() {
    let v4: Ipv4Addr = "2.2.2.2".parse().unwrap();
    let v6: Ipv6Addr = "::1".parse().unwrap();
    
    // IPAddr 实现了 From<[u8; 4]，转换 IPv4 地址
    print([1, 1, 1, 1]);
  
    // IPAddr 实现了 From<[u16; 8]，转换 IPv6 地址
    print([0xfe80, 0, 0, 0, 0xaede, 0x48ff, 0xfe00, 0x1122]);
  
    // IPAddr 实现了 From<Ipv4Addr>
    print(v4);
  
    // IPAddr 实现了 From<Ipv6Addr>
    print(v6);
}