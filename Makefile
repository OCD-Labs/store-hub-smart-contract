build:
	RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
	mkdir -p ./out
	cp target/wasm32-unknown-unknown/release/*.wasm ./out/main.wasm

test:
	cargo test

.PHONY: build test