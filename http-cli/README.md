
# 编译后运行
## 开发阶段编译
```bash
# 编译 并 运行发送post命令， 默认cargo build 编译出来的二进制，在项目根目录的 target/debug
cargo build 

# 进入debug目录
cd ../target/debug/

# post命令
./http-cli post https://httpbin.org/post a=1 b=2

# get命令
./http-cli get http://abc.xyz

```

## 编译成release版本
```bash

# 加上release编译后存放在release目录下
cargo build --release

# 进入release目录
cd ../target/release

# post命令
./http-cli post https://httpbin.org/post a=1 b=2

# get命令
./http-cli get http://abc.xyz

```
# 使用cargo run
```bash
# 在main.rs目录下，运行即可， --相当于target/debug/http-cli
cargo run -- post https://httpbin.org/post a=1 b=2
```

# 单元测试
```bash
cargo test
```