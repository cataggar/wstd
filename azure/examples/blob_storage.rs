//! Example Azure Blob Storage client running on `wstd` via `wstd_azure`
//!
//! This example demonstrates how to set up the wstd runtime for use with the
//! Azure Rust SDK. The actual Azure SDK client code will depend on which
//! version of the azure_storage_blob crate you are using.
//!
//! This example *must be compiled in release mode* - in debug mode, the azure
//! sdk's generated code will overflow the maximum permitted wasm locals in
//! a single function.
//!
//! Compile it with:
//!
//! ```sh
//! cargo build -p wstd-azure --target wasm32-wasip2 --release --examples
//! ```
//!
//! When running this example, you will need Azure credentials provided in environment
//! variables.
//!
//! Run it with:
//! ```sh
//! wasmtime run -Shttp \
//!     --env AZURE_STORAGE_ACCOUNT_NAME \
//!     --env AZURE_STORAGE_ACCESS_KEY \
//!     --dir .::. \
//!     target/wasm32-wasip2/release/examples/blob_storage.wasm
//! ```

use anyhow::Result;

#[wstd::main]
async fn main() -> Result<()> {
    // Set up wstd runtime for Azure SDK
    // This configures the Azure SDK to use wstd's async runtime and HTTP client
    wstd_azure::set_wstd_runtime()?;

    println!("wstd runtime configured for Azure SDK");
    println!("");
    println!("To use the Azure SDK:");
    println!("1. Import the Azure SDK crates you need (e.g., azure_storage_blob)");
    println!("2. Make sure to use default-features = false for Azure crates");
    println!("3. Create your Azure SDK clients as normal");
    println!("4. The SDK will use wstd's wasi-http implementation automatically");

    // Example placeholder - actual Azure SDK usage will depend on your version
    // and which services you're using. See the Azure SDK documentation at:
    // https://github.com/Azure/azure-sdk-for-rust

    Ok(())
}
