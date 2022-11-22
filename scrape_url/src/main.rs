use std::fs;

// 拉取rust文档html,然后转成markdown的例子
// 执行 cargo run 会编译后运行 main.rs

// 没有错误处理，unwrap()只关心成功结果
fn rust_html_to_md() {
    let url = "https://www.rust-lang.org/";
    let output = "rust1.md";

    println!("请求url {}", url);

    let body = reqwest::blocking::get(url).unwrap().text().unwrap();

    println!("开始转换html为markdown");
    
    let md = html2md::parse_html(&body);

    fs::write(output, md.as_bytes()).unwrap();

    println!("转换后的markdown文件为 {}", output); 
}

// 错误处理
// main 函数返回一个 Result<T, E>
fn rust_html_to_md_error() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.rust-lang.org/";
    let output = "rust2.md";

    println!("请求url {}", url);

    // 把unwrap()换成了?，并让函数返回一个Result<T, E>，原先使用unwrap方法只会关心返回成功的结果
    let body = reqwest::blocking::get(url)?.text()?; 

    println!("开始转换html为markdown");

    let md = html2md::parse_html(&body);

    fs::write(output, md.as_bytes())?;
    println!("转换后的markdown文件为 {}", output); 

    Ok(())
}

fn main() {
    rust_html_to_md();

    let download_rst = rust_html_to_md_error();

    // 用match匹配返回的Result结果
    let rst = match download_rst {
        Ok(()) => (),
        Err(error) => panic!("下载错误: {:?}", error),
    };

    rst
}
