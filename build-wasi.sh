#!/bin/sh

export RUSTFLAGS='-C target-feature=+simd128'

cargo \
	build \
	--target wasm32-wasip1 \
	--profile release-wasi
