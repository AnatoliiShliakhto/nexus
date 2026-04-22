//! # Distributed Tracing Context
//!
//! This module implements the [W3C Trace Context](https://www.w3.org/TR/trace-context/)
//! specification. It provides a lightweight, stack-allocated structure for parsing,
//! generating, and propagating trace identifiers across service boundaries.
//!
//! ## Traceparent Format
//! A `traceparent` identifier is a 55-character string formatted as:
//! `version-traceid-parentid-flags`
//! (e.g., `00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01`)

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// A stack-allocated wrapper around a W3C `traceparent` byte array.
///
/// `TraceContext` is designed for high-performance scenarios:
/// - **Zero Allocations**: Uses a fixed-size `[u8; 55]` buffer.
/// - **Fast Accessors**: Uses `unsafe` string conversions (pre-validated) for zero-copy access to IDs.
/// - **Thread-Safe**: Includes a fall-back PRNG for generating IDs if the system entropy fails.
#[derive(Debug, Clone, Copy)]
pub struct TraceContext([u8; 55]);

impl TraceContext {
    /// Generates a completely new `TraceContext` with a random Trace ID and Span ID.
    #[must_use]
    pub fn new() -> Self {
        let mut buffer = [0u8; 55];

        buffer[0..3].copy_from_slice(b"00-");
        buffer[35] = b'-';
        buffer[52] = b'-';
        buffer[53..55].copy_from_slice(b"01");

        let mut trace_id_bytes = [0u8; 16];
        let mut parent_id_bytes = [0u8; 8];

        fill_random(&mut trace_id_bytes);
        fill_random(&mut parent_id_bytes);

        write_hex(&mut buffer[3..35], &trace_id_bytes);
        write_hex(&mut buffer[36..52], &parent_id_bytes);

        Self(buffer)
    }

    /// Creates a child context from the current one.
    ///
    /// This keeps the same **Trace ID** but generates a new **Span ID** (`parent_id`).
    /// Use this when performing sub-operations or calling upstream services
    #[must_use]
    pub fn child(&self) -> Self {
        let mut buffer = self.0;

        let mut new_span_id_bytes = [0u8; 8];

        fill_random(&mut new_span_id_bytes);

        write_hex(&mut buffer[36..52], &new_span_id_bytes);

        Self(buffer)
    }

    /// Parses a `traceparent` string.
    ///
    /// If the string is invalid, it logs a warning and returns a **newly generated** context
    /// to ensure the trace chain is never broken, even if it has to start fresh.
    pub fn from(s: impl AsRef<str>) -> Self {
        Self::try_from(s.as_ref()).unwrap_or_else(|e| {
            tracing::warn!(err = %e, "Invalid `traceparent` format: {}", s.as_ref());
            Self::new()
        })
    }

    /// Returns the 32-character Hex representation of the Trace ID.
    #[must_use]
    pub fn trace_id(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0[3..35]) }
    }

    /// Returns the 16-character Hex representation of the Parent (Span) ID.
    #[must_use]
    pub fn parent_id(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0[36..52]) }
    }

    /// Returns the full 55-character W3C `traceparent` string.
    #[must_use]
    pub const fn traceparent(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<&str> for TraceContext {
    type Error = &'static str;

    /// Strict validation of the `traceparent` string according to W3C standards.
    ///
    /// Validates length, delimiters, and hex-encoding of components.
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let bytes = s.as_bytes();
        if bytes.len() != 55 {
            return Err("Invalid traceparent format");
        }

        if &bytes[0..3] != b"00-" {
            return Err("Invalid traceparent format");
        }

        if bytes[35] != b'-' || bytes[52] != b'-' {
            return Err("Invalid traceparent format");
        }

        if !is_hex_range(&bytes[3..35])
            || !is_hex_range(&bytes[36..52])
            || !is_hex_range(&bytes[53..55])
        {
            return Err("Invalid traceparent format");
        }

        let mut buffer = [0u8; 55];
        buffer.copy_from_slice(bytes);
        Ok(Self(buffer))
    }
}

// --- Helpers ---

fn is_hex_range(slice: &[u8]) -> bool {
    slice.iter().all(u8::is_ascii_hexdigit)
}

fn write_hex(out: &mut [u8], input: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for (i, &byte) in input.iter().enumerate() {
        out[i * 2] = HEX[(byte >> 4) as usize];
        out[i * 2 + 1] = HEX[(byte & 0x0F) as usize];
    }
}

// --- RNG ---

static RNG_INIT: OnceLock<()> = OnceLock::new();
static RNG_STATE: AtomicU64 = AtomicU64::new(42);

/// Ensures the fallback PRNG is initialized with a time-based seed.
///
/// This is used only if `getrandom` (system entropy) fails.
#[allow(clippy::cast_possible_truncation)]
pub fn ensure_rng_init() {
    RNG_INIT.get_or_init(|| {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(42, |d| d.as_micros() as u64);

        RNG_STATE.store(now, Ordering::SeqCst);

        tracing::debug!("PRNG initialized with seed: {}", now);
    });
}

fn xor_shift64() -> u64 {
    ensure_rng_init();

    let mut x = RNG_STATE.load(Ordering::Relaxed);
    if x == 0 {
        x = 1;
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    RNG_STATE.store(x, Ordering::Relaxed);
    x
}

fn fill_random(dest: &mut [u8]) {
    if getrandom::fill(dest).is_ok() {
        return;
    }

    tracing::warn!("`getrandom` failed, falling back to PRNG");

    for chunk in dest.chunks_mut(8) {
        let rand_val = xor_shift64();
        let bytes = rand_val.to_be_bytes();
        let len = chunk.len();
        chunk.copy_from_slice(&bytes[..len]);
    }
}
