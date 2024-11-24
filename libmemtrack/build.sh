#!/usr/bin/env bash

cargo build --release
cd examples/simple
cargo build --profile dev