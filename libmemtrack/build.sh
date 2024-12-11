#!/usr/bin/env bash

#cargo build --release
#cd examples/simple
#cargo build --profile dev

export RUSTFLAGS="-Zsanitizer=address" 
export RUSTDOCFLAGS="-Zsanitizer=address"

cargo +nightly build --release --target=aarch64-apple-darwin
