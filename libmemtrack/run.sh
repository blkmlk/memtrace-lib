#!/usr/bin/env bash

rm -f /tmp/pipe
mkfifo /tmp/pipe

#export RUST_BACKTRACE=full
export DYLD_LIBRARY_PATH=$(rustc --print sysroot)/lib
export DYLD_INSERT_LIBRARIES=/Users/id/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/lib/librustc-nightly_rt.asan.dylib
#export DYLD_INSERT_LIBRARIES=/Users/id/devel/Rust/memtrack-rs/libmemtrack/target/release/liblibmemtrack.dylib
export PIPE_FILEPATH=/tmp/pipe
#./examples/simple/target/debug/simple

cd /Users/id/devel/ALT/backtest/backtest
/Users/id/devel/ALT/backtest/backtest/target/release/examples/math_cmp
