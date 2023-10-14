install:
	cargo update
	
build:
	wasm-pack build --target web
