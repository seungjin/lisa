build:
	#cargo build --target wasm32-wasip2
	cargo component build --target wasm32-wasip2
test:
	#wasmtime wast test.wast
	cargo test -- --nocapture
run:
	wasmtime -S http -S inherit-env target/wasm32-wasip2/debug/lisa.wasm Who are you?
clean:
	cargo clean
release:
	cargo build --target wasm32-wasip2 --release
	cp target/wasm32-wasip2/release/lisa.wasm lisa.wasm
install:
	cargo build --target wasm32-wasip2 --release
	cp target/wasm32-wasip2/release/lisa.wasm ~/.local/bin/lisa.wasm
	cp lisa, ~/.local/bin/lisa,
