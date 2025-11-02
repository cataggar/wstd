# wstd-azure: wstd support for the Azure Rust SDK

This crate provides support for using the Azure Rust SDK for the `wasm32-wasip2`
target using the [`wstd`] crate.

In many wasi settings, it's necessary or desirable to use the wasi-http
interface to make http requests. Wasi-http interfaces provide an http
implementation, including the sockets layer and TLS, outside of the user's
component. `wstd` provides user-friendly async Rust interfaces to all of the
standardized wasi interfaces, including wasi-http.

The Azure Rust SDK, by default, depends on `tokio` and `reqwest`, and makes
http requests over sockets (which can be provided as wasi-sockets). Those
dependencies may not work correctly under `wasm32-wasip2`, and if they do,
they will not use the wasi-http interfaces. To avoid using http over sockets,
make sure to set the `default-features = false` setting when depending on any
`azure_*` crates in your project.

## Usage

To configure `wstd`'s wasi-http client and async runtime for the Azure Rust SDK,
call `wstd_azure::set_wstd_runtime()` once at the start of your application:

```rust
use wstd_azure::set_wstd_runtime;

#[wstd::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up wstd runtime for Azure SDK
    set_wstd_runtime()?;
    
    // Now use Azure SDK as normal
    // ...
    
    Ok(())
}
```

Alternatively, you can get the runtime and HTTP client separately if you need
more control:

```rust
use wstd_azure::{async_runtime, http_client};

// Get the async runtime
let runtime = async_runtime();

// Get the HTTP client
let client = http_client();
```

## Example

An example Azure Blob Storage client is provided. See the `examples` directory
for a complete working example.

## Requirements

- Rust 1.89 or later
- `wasm32-wasip2` target installed
- `wasmtime` or another WASI runtime with wasi-http support

[`wstd`]: https://docs.rs/wstd/latest/wstd
