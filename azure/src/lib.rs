//! wstd support for the Azure Rust SDK
//!
//! This crate provides support for using the Azure Rust SDK for the `wasm32-wasip2`
//! target using the [`wstd`] crate.
//!
//! In many wasi settings, it's necessary or desirable to use the wasi-http
//! interface to make http requests. Wasi-http interfaces provide an http
//! implementation, including the sockets layer and TLS, outside of the user's
//! component. `wstd` provides user-friendly async Rust interfaces to all of the
//! standardized wasi interfaces, including wasi-http.
//!
//! The Azure Rust SDK, by default, depends on `tokio` and `reqwest`, and makes
//! http requests over sockets (which can be provided as wasi-sockets). Those
//! dependencies may not work correctly under `wasm32-wasip2`, and if they do,
//! they will not use the wasi-http interfaces. To avoid using http over sockets,
//! make sure to set the `default-features = false` setting when depending on any
//! `azure_*` crates in your project.
//!
//! To configure `wstd`'s wasi-http client and async runtime for the Azure Rust SDK,
//! call [`set_wstd_runtime()`] once at the start of your application:
//!
//! ```no_run
//! # use wstd_azure::set_wstd_runtime;
//! set_wstd_runtime().expect("Failed to set runtime");
//! ```
//!
//! Then use the Azure SDK as normal.
//!
//! [`wstd`]: https://docs.rs/wstd/latest/wstd

use async_trait::async_trait;
use std::sync::Arc;
use typespec_client_core::async_runtime::{AsyncRuntime, SpawnedTask, TaskFuture};
use typespec_client_core::http::{BufResponse, HttpClient, Request};
use typespec_client_core::time::Duration;
use wstd::http::{Body as WstdBody, Client};

/// Set the wstd-based async runtime and HTTP client as the default for the Azure SDK.
///
/// This should be called once at the start of your application before using any
/// Azure SDK functionality.
///
/// # Errors
///
/// Returns an error if the runtime has already been set.
///
/// # Example
///
/// ```no_run
/// use wstd_azure::set_wstd_runtime;
///
/// #[wstd::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     set_wstd_runtime()?;
///     // Use Azure SDK...
///     Ok(())
/// }
/// ```
pub fn set_wstd_runtime() -> typespec_client_core::Result<()> {
    typespec_client_core::async_runtime::set_async_runtime(Arc::new(WstdAsyncRuntime))
}

/// Get a new wstd-based HTTP client for the Azure SDK.
///
/// This creates a new HTTP client that uses wstd's wasi-http implementation.
pub fn http_client() -> Arc<dyn HttpClient> {
    Arc::new(WstdHttpClient::new())
}

/// Get a wstd-based async runtime for the Azure SDK.
///
/// This creates a new async runtime that uses wstd's task spawning and timing.
pub fn async_runtime() -> Arc<dyn AsyncRuntime> {
    Arc::new(WstdAsyncRuntime)
}

/// Async runtime implementation using wstd.
#[derive(Debug)]
struct WstdAsyncRuntime;

impl AsyncRuntime for WstdAsyncRuntime {
    fn spawn(&self, f: TaskFuture) -> SpawnedTask {
        Box::pin(async move {
            wstd::runtime::spawn(f).await;
            Ok(())
        })
    }

    fn sleep(&self, duration: Duration) -> TaskFuture {
        Box::pin(async move {
            // Convert time crate Duration to std Duration to wstd Duration
            // time::Duration has whole_seconds() and subsec_nanoseconds()
            let secs = duration.whole_seconds() as u64;
            let nanos = duration.subsec_nanoseconds() as u32;
            let std_duration = std::time::Duration::new(secs, nanos);
            wstd::task::sleep(wstd::time::Duration::from(std_duration)).await;
        })
    }
}

/// HTTP client implementation using wstd.
#[derive(Debug)]
struct WstdHttpClient {
    client: Client,
}

impl WstdHttpClient {
    fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl HttpClient for WstdHttpClient {
    async fn execute_request(
        &self,
        request: &Request,
    ) -> typespec_client_core::Result<BufResponse> {
        // Convert the Azure SDK request to a wstd http request
        let url = request.url().to_string();
        let method = convert_method(request.method());

        let mut http_request = http::Request::builder().method(method).uri(&url);

        // Add headers
        for (name, value) in request.headers().iter() {
            http_request = http_request.header(name.as_str(), value.as_str());
        }

        // Add body
        let body = match request.body() {
            typespec_client_core::http::Body::Bytes(bytes) => WstdBody::from(bytes.clone()),
            #[cfg(not(target_arch = "wasm32"))]
            typespec_client_core::http::Body::SeekableStream(_) => {
                return Err(typespec_client_core::Error::with_message(
                    typespec_client_core::error::ErrorKind::Other,
                    "Streaming request bodies are not supported",
                ));
            }
        };

        let http_request = http_request.body(body).map_err(|e| {
            typespec_client_core::Error::with_message(
                typespec_client_core::error::ErrorKind::Other,
                format!("Failed to build HTTP request: {}", e),
            )
        })?;

        // Send the request
        let response = self.client.send(http_request).await.map_err(|e| {
            typespec_client_core::Error::with_message(
                typespec_client_core::error::ErrorKind::Io,
                format!("HTTP request failed: {}", e),
            )
        })?;

        // Convert the response
        let status = response.status();
        let headers = convert_headers(response.headers());

        // Collect the response body into bytes
        let mut body = response.into_body();
        let body_bytes = body.contents().await.map_err(|e| {
            typespec_client_core::Error::with_message(
                typespec_client_core::error::ErrorKind::Io,
                format!("Failed to read response body: {}", e),
            )
        })?;

        Ok(BufResponse::from_bytes(
            typespec_client_core::http::StatusCode::from(status.as_u16()),
            headers,
            bytes::Bytes::copy_from_slice(body_bytes),
        ))
    }
}

fn convert_method(method: typespec_client_core::http::Method) -> http::Method {
    use typespec_client_core::http::Method;
    match method {
        Method::Get => http::Method::GET,
        Method::Post => http::Method::POST,
        Method::Put => http::Method::PUT,
        Method::Delete => http::Method::DELETE,
        Method::Head => http::Method::HEAD,
        Method::Patch => http::Method::PATCH,
        _ => http::Method::GET, // Default fallback for any future methods
    }
}

fn convert_headers(headers: &http::HeaderMap) -> typespec_client_core::http::headers::Headers {
    let mut result = typespec_client_core::http::headers::Headers::new();
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            result.insert(
                typespec_client_core::http::headers::HeaderName::from(name.to_string()),
                typespec_client_core::http::headers::HeaderValue::from(value_str.to_string()),
            );
        }
    }
    result
}
