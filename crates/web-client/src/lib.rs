//! # Nexus Web Client
//!
//! A high-performance, strictly typed HTTP client for the NEXUS ecosystem,
//! optimized for WebAssembly (Dioxus) environments.
//!
//! ## Core Features
//! - **Automated DPoP**: Transparent handling of Proof-of-Possession signatures (RFC 9449).
//! - **Session Orchestration**: Automatic Access/Refresh token management with `nx-lockbox` integration.
//! - **Reactivity**: Built-in support for Dioxus via specialized hooks (optional).
//! - **Efficiency**: Leverages `arc-swap` for lock-free state access and `fxhash` for fast lookups.

mod client;
#[cfg(feature = "dioxus")]
mod dioxus;
mod dpop;
pub mod error;
mod request;
mod response;
pub mod session;

#[cfg(feature = "dioxus")]
pub use dioxus::*;
pub use request::*;
pub use response::*;
