#!/usr/bin/env bash

rm -f /tmp/pipe
mkfifo /tmp/pipe

export DYLD_LIBRARY_PATH=$(rustc --print sysroot)/lib
export DYLD_INSERT_LIBRARIES=./target/release/liblibmemtrack.dylib
export PIPE_FILEPATH=/tmp/pipe
./examples/simple/target/debug/simple
