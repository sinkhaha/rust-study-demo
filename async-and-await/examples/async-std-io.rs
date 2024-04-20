use anyhow::Result;
use async_std::{fs, task};
use futures::try_join;
use serde_yaml::Value;

fn toml2yaml(content: &str) -> Result<String> {
    let value: Value = toml::from_str(&content)?;
    Ok(serde_yaml::to_string(&value)?)
}

// 异步函数
async fn read_toml2yaml() -> Result<()> {
    // 这里使用异步的read_to_string，注意后面没有加 await
    let f1 = fs::read_to_string("./Cargo.toml");
    let f2 = fs::read_to_string("./Cargo.lock");

    // 这里用try_join宏等待两个异步任务完成
    let (content1, content2) = try_join!(f1, f2)?;

    // 计算
    let yaml1 = toml2yaml(&content1)?;
    let yaml2 = toml2yaml(&content2)?;

    let f3 = fs::write("/tmp/Cargo.yml", &yaml1);
    let f4 = fs::write("/tmp/Cargo.lock", &yaml2);

    try_join!(f3, f4)?;

    println!("{}", yaml1);
    println!("{}", yaml2);

    Ok(())
}

// 运行 cargo run --example async-std-io.rs

// main还是一个同步方法
fn main() -> Result<()> {
    // main是同步方法，不能使用await，所以用了block_on
    // 这里用了 block_on 去调度和执行 read_toml2yaml 返回的 Future，这里会阻塞等待 Future 的完成
    task::block_on(read_toml2yaml())?;
    Ok(())
}
