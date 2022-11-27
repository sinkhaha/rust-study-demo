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
