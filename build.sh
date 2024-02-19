#!/usr/bin/env bash
rm -rf dist
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
trunk build --release

