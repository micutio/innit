#!/bin/bash

# exit early on errors and unbound variables
set -euo pipefail

# install wasm target and bindgen
# rustup target add wasm32-unknown-unknown
# cargo install wasm-bindgen-cli

# compile to wasm
cargo build --release --target wasm32-unknown-unknown

# create deployable files
wasm-bindgen target/wasm32-unknown-unknown/release/innit.wasm --no-typescript --out-dir wasm --target no-modules

# optional extra step: copy files to github-pages website
cp wasm/innit* ../micutio.github.io/wasm 
