RFLAGS="-C link-arg=-s"

build: build-grid build-common

build-grid: contracts/grid
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p grid --target wasm32-unknown-unknown --release
	mkdir -p res
	rm ./contracts/grid/res/grid.wasm
	cp target/wasm32-unknown-unknown/release/grid.wasm ./contracts/grid/res/grid.wasm
	cp target/wasm32-unknown-unknown/release/grid.wasm ./res/grid.wasm

build-common: contracts/common
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p common --target wasm32-unknown-unknown --release
	mkdir -p res
	rm ./contracts/grid/res/token.wasm
	cp target/wasm32-unknown-unknown/release/common.wasm ./contracts/grid/res/token.wasm

release:
	$(call docker_build,_rust_setup.sh)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/grid.wasm res/grid_release.wasm

unittest: build
ifdef TC
	RUSTFLAGS=$(RFLAGS) cargo test $(TC) -p grid --lib -- --nocapture
else
	RUSTFLAGS=$(RFLAGS) cargo test -p grid --lib -- --nocapture
endif

test: build
ifdef TF
	RUSTFLAGS=$(RFLAGS) cargo test -p grid --test $(TF) -- --nocapture
else
	RUSTFLAGS=$(RFLAGS) cargo test -p grid --tests -- --nocapture
endif

clean:
	cargo clean
	rm -rf res/

define docker_build
	docker build -t my-grid-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-grid-builder \
		/bin/bash $(1)
endef
