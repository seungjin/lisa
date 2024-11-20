default:
    just --list

wit-update:
    ./wit-download.sh

build:
	#cargo build --target wasm32-wasip2
	cargo clean
	cargo component build --target wasm32-wasip2

test:
	#wasmtime wast test.wast
	cargo test -- --nocapture

run:
    #!/usr/bin/env bash
    #export OPENAI_API_KEY="hello_I_am_lisa"
    wasmtime -S http -S inherit-env target/wasm32-wasip2/debug/lisa.wasm hello

clean:
	cargo clean

release:
	cargo build --target wasm32-wasip2 --release
	cp target/wasm32-wasip2/release/lisa.wasm lisa.wasm

install:
	cargo build --target wasm32-wasip2 --release
	install -m 755 target/wasm32-wasip2/release/lisa.wasm ~/.local/bin/lisa.wasm
	install -m 755 lisa, ~/.local/bin/lisa,
