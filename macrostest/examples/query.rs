use macrostest::query;

// 运行cargo run --example query
fn main() {
    query!(SELECT * FROM users WHERE age > 10);
    hello(); // 调用query宏的hello函数，输出Hello world!
}
