// 错误处理的例子

use std::fs;

// main 函数返回一个 Result<T, E>
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.rust-lang.org/";
    let output = "rust.md";

    println!("Fetching url: {}", url);

    let body = reqwest::blocking::get(url)?.text()?; // 把unwrap()换成了?，并让main函数返回一个Result<T, E>。原先使用unwrap方法只会关心返回成功的结果

    println!("Converting html to markdown...");

    let md = html2md::parse_html(&body);

    fs::write(output, md.as_bytes())?;
    println!("Converted markdown has been saved in {}.", output);

    Ok(())
}