// 参考 https://github.com/tyrchen/geektime-rust/tree/master/04_httpie

use std::{str::FromStr, collections::HashMap};
use clap::Parser;
use anyhow::{anyhow, Result};
use reqwest::{header, Client, Response, Url};
use mime::Mime;
use colored::Colorize;
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

#[derive(Parser, Debug)]
#[clap(version = "1.0", author="sinkhaha")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

// 子命令对应两个不同的http方法
#[derive(Parser, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

// get子命令
#[derive(Parser, Debug)]
struct Get {
    // 使用parse_url函数解析url
    #[clap(parse(try_from_str=parse_url))]
    url: String, // 请求的url
}

// post子命令
#[derive(Parser, Debug)]
struct Post {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    #[clap(parse(try_from_str = parse_kv_pair))]
    body: Vec<KvPair> // 请求体
}

// 解析url的方法
fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?; // 检查下url是否合法
    Ok(s.into())
}

// key-value对
#[derive(Debug, PartialEq)]
struct KvPair {
    k: String,
    v: String,
}

// KvPair实现FromStr
impl FromStr for KvPair {
    type Err = anyhow::Error;

    // 实现from_str方法
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 根据=分割，得到一个迭代器
        let mut split = s.split("=");
        let err = || anyhow!(format!("failed to parse {}", s));

        Ok(Self {
            k: (split.next().ok_or_else(err)?).to_string(), // 从迭代器取第一个结果为key,迭代器返回 Some(T)/None,将其转换成 Ok(T)/Err(E)，然后用 ? 处理错误
            v: (split.next().ok_or_else(err)?).to_string(), // 从迭代器取第二个结果为value
        })
    }
}

// 因为KvPair 实现了 FromStr，这里可以直接 s.parse() 得到 KvPair
fn parse_kv_pair(s: &str) -> Result<KvPair>{
    Ok(s.parse()?)
}

// get命令的处理方法
async fn get_handle(client: Client, args: &Get) -> Result<()>{
    let resp = client.get(&args.url).send().await?;

    Ok(print_resp(resp).await?)
}

// post命令的处理方法
async fn post_handle(client: Client, args: &Post)-> Result<()> {
    let mut body = HashMap::new();

    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }

    let resp = client.post(&args.url).json(&body).send().await?;

    Ok(print_resp(resp).await?)
}

//===========================一些工具函数=====================================

// 打印服务器版本 + 状态
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

// 打印返回的响应头
fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
    println!("\n");
}

// 打印http响应体
fn print_body(m: Option<Mime>, body: &String) {
    match m {
        // 对于 "application/json"美化输出
        Some(v) if v == mime::APPLICATION_JSON => print_syntect(body, "json"),
        Some(v) if v == mime::TEXT_HTML => print_syntect(body, "html"),

        // 其它 mime type，直接输出
        _ => println!("{}", body),
    }
}

// 将服务器返回的 content-type 解析成 Mime 类型
fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())   
}

// 打印整个响应
async fn print_resp(resp: Response) -> Result<()>{
    print_status(&resp);
    print_headers(&resp);

    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);

    Ok(())
}

fn print_syntect(s: &str, ext: &str) {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_extension(ext).unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    for line in LinesWithEndings::from(s) {
        let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
}

//================================================================

// 在http请求时使用异步
#[tokio::main]
async fn main() -> Result<()>{
    let opts: Opts = Opts::parse(); // parse函数是#[derive(Clap)]自动生成的

    let mut headers = header::HeaderMap::new();
    // 添加一些默认的请求头
    headers.insert("X-POWERED-BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);

    // http请求客户端
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    // 匹配命令，不同命令执行不同的处理方法
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get_handle(client, args).await?,
        SubCommand::Post(ref args) => post_handle(client, args).await?,
    };

    Ok(result)
}

// ========================单元测试========================================
// 仅在运行 cargo test 时才编译
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_works() {
        assert!(parse_url("abc").is_err());
        assert!(parse_url("http://abc.xyz").is_ok());
        assert!(parse_url("https://httpbin.org/post").is_ok());
    }

    #[test]
    fn parse_kv_pair_works() {
        assert!(parse_kv_pair("a").is_err());
        assert_eq!(
            parse_kv_pair("a=1").unwrap(),
            KvPair {
                k: "a".into(),
                v: "1".into()
            }
        );

        assert_eq!(
            parse_kv_pair("b=").unwrap(),
            KvPair {
                k: "b".into(),
                v: "".into()
            }
        );
    }
}