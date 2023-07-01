build:
	RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
	mkdir -p ./out
	cp target/wasm32-unknown-unknown/release/*.wasm ./out/main.wasm

test:
	cargo test

dev-deploy:
	rm -rf ./neardev
	near dev-deploy ./out/main.wasm new '{"overseer_id": "storehub.testnet"}'

.PHONY: build test dev-deploy