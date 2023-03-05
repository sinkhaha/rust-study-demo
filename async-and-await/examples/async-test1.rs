use std::future::Future;

// 运行 cargo run --example async-test1
#[tokio::main]
async fn main() {
    let name1 = "zhangsan".to_string();
    let name2 = "lisi".to_string();

    say_hello1(&name1).await;
    say_hello2(&name2).await;

}

async fn say_hello1(name: &str) -> usize {
    println!("Hello {}", name);
    42
}

// async fn 关键字相当于一个返回 impl Future<Output> 的语法糖
fn say_hello2<'fut>(name: &'fut str) -> impl Future<Output = usize> + 'fut {
    async move {
        println!("Hello {}", name);
        42
    }
}