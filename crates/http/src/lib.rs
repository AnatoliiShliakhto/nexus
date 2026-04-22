//! # Nexus HTTP
//!
//! `nx-http` is a comprehensive toolkit for building observable and robust HTTP services
//! within the `WebAssembly (WASI)` ecosystem, specifically optimized for the Spin framework
//! and the WASI Component Model.
//!
//! ## Core Components
//!
//! - **Telemetry & Tracing**: Integrated W3C Trace-Context propagation via the [`trace`]
//!   and [`telemetry`] modules.
//! - **Macros**: Procedural attributes ([`spin_service`], [`spin_service_unstable`], [`wasi_service`]) that automate boilerplate for service entry points, error interception, and instrumentation.
//! - **Error Handling**: Re-exports [`nx_error`] for standardized, structured JSON
//!   error reporting.
//!
//! ## Feature Flags
//!
//! - `spin`: Enables Spin-specific SDK integrations and the `spin_service` macro.
//! - `unstable`: Enables Spin-specific SDK `WASIp3` integrations and the `spin_service_unstable` macro.

pub mod error;
pub mod request;
#[cfg(feature = "spin")]
pub mod spin_tools;
pub mod telemetry;
pub mod trace;
#[cfg(feature = "wasi")]
pub mod wasi;

pub use bytes;
#[cfg(feature = "wasi")]
pub use http;
#[cfg(feature = "spin")]
pub use nx_http_macros::router;
#[cfg(feature = "spin")]
pub use nx_http_macros::spin_service;
#[cfg(feature = "wasi")]
pub use nx_http_macros::wasi_service;
#[cfg(feature = "spin")]
pub use spin_sdk;
pub use tracing;
pub use url;
