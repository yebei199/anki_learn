set shell := ["fish", "-c"]
set dotenv-load := true
set export := true

# 在当前目录下，清理所有非当前环境的构建产物
clean:
    cargo sweep --time 0 .

release:
    cargo build --release

musl_release:
    cargo build --release --target x86_64-unknown-linux-musl

bin_test:
    target/x86_64-unknown-linux-musl/release/hh

