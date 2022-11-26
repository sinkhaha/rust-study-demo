fn main() {
    let data = vec![1, 2, 3, 4]; // Vec<u32>类型是动态大小，存储在堆中
    let data1 = &data;
    let data2 = &data;
    // 值的地址是什么？引用的地址又是什么？

    // &data是 Vec<u32>堆数据的地址
    // data1是引用类型，data1的值是data的地址，所以也是 Vec<u32>堆数据的地址；data2同data1
    // &data1是data1这个变量本身在栈上的地址
    // &&data是“&data引用”的引用
    // &*data，即先*data是解指针，即得到堆的数据，然后加上&表示引用
    println!(
        "addr of value: {:p}({:p})({:p}), {:p}, addr of data {:p}, data1: {:p}",
        &data, data1, data2, &&data, &*data, &data1, 
    );
    println!("sum of data1: {}", sum(data1));

    // 堆上数据的地址是什么？
    println!(
        "addr of items: [{:p}, {:p}, {:p}, {:p}]",
        &data[0], &data[1], &data[2], &data[3]
    );
}

fn sum(data: &Vec<u32>) -> u32 {
    // 值的地址会改变么？引用的地址会改变么？
    // data是“data引用”即&data的地址
    // &data是“data引用”的引用，即“data引用”(&data)的地址
    // *&data：即“&data”的解引用，&data的值是Vec堆数据的地址
    println!("addr of value: {:p}, addr of ref: {:p}, {:p}", data, &data, *&data);
    data.iter().fold(0, |acc, x| acc + x)
}
