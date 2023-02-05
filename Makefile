.PHONY: all clean serve npm

all: www/index.html www/cpg.js www/cpg.wasm

clean:
	rm -rf www target

serve:
	npx serve www

npm:
	npm i

www/index.html: index.html
	@mkdir -p www
	cp "$<" "$@"

www/cpg.js: cpg.mjs package.json node_modules/.bin/esbuild
	@mkdir -p www
	node_modules/.bin/esbuild cpg.mjs --bundle --outdir=./www --target=esnext --analyze --minify --sourcemap

www/cpg.wasm: target/wasm32-unknown-unknown/release/cassowary_playground.wasm
	@mkdir -p www
	cp "$<" "$@"

target/wasm32-unknown-unknown/release/cassowary_playground.wasm: ./src/lib.rs Cargo.toml
	cargo build --target wasm32-unknown-unknown --release
