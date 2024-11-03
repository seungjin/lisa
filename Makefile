build:
	#cargo build --target wasm32-wasip2
	cargo component build --target wasm32-wasip2
test:
	#wasmtime wast test.wast
	cargo test -- --nocapture
run:
	wasmtime -S http -S inherit-env target/wasm32-wasip2/debug/openai.wasm hello world
clean:
	cargo clean
release:
	cargo build --target wasm32-wasip2 --release
	cp target/wasm32-wasip2/release/openai.wasm openai.wasm
install:
	cargo build --target wasm32-wasip2 --release
	cp target/wasm32-wasip2/release/openai.wasm ~/.local/bin/openai.wasm
