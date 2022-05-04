#!/bin/bash

cargo build --target wasm32-wasi --release && wasmedge target/wasm32-wasi/release/http_server.wasm