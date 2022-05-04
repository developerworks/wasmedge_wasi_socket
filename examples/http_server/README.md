# WasmEdge WASI Socket Http Server Demo

This demo runs an echo server on `localhost`.

## Build

```shell
cargo build --target wasm32-wasi --release
```

## Run

```shell
wasmedge target/wasm32-wasi/release/http_server.wasm
```

## Test

In another terminal window, do the following.

```shell
curl -X POST http://127.0.0.1:1234 -d "name=WasmEdge Networking API"
echo: name=WasmEdge
```

## Improve 

- Add [`serde`](https://crates.io/crates/serde)/[`serde_json`](https://crates.io/crates/serde_json) to return application/json http response
- Improve logging
- 
