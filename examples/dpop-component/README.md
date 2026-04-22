# Nexus DPoP Component

The `nx-dpop` crate provides a high-performance, Wasm-native service for validating DPoP 
(Demonstrating Proof-of-Possession) as defined in RFC 9449. Designed for the Spin ecosystem, this
crate acts as a central validation authority, enabling stateless and stateful services to perform
cryptographic verification of client-bound requests.

---

## Overview

DPoP allows clients to prove possession of a private key for a given public key (JWK) by signing a
request-specific proof. This crate offers two primary services:

* **Thumbprint Computation**: Generates the unique JWK Thumbprint (JKT) for a given proof.
* **Request Verification**: Validates the proof’s integrity, temporal validity, context 
  (method/URI), and access token binding (ATH).

## Architecture

This component is built specifically for WASI/Spin (Wasm). It uses binary serialization (postcard)
to minimize payload size and processing time, ensuring sub-millisecond validation latency.

## Features

* **High Performance**: Uses optimized Ed25519 and P-256 signatures verification and manual JSON
  canonicalization to stay within Wasm performance budgets.
* **Strict Security**:
    * **Anti-Replay**: Atomic JTI (JWT ID) checking via Redis.
    * **Context Binding**: Strict validation of the `htm` (method) and `htu` (target URI) claims.
    * **Memory Hardening**: Enforces strict payload size limits to prevent Wasm OOM (Out-of-Memory)
      errors.
* **Protocol-Agnostic**: Designed to be called as an internal microservice, separating validation
  logic from your primary application handlers.

## API Endpoints

The service exposes the following POST endpoints:

| Endpoint   | Input                      | Output         | Description                          |
|------------|----------------------------|----------------|--------------------------------------|
| `/compute` | `ComputeThumbprintRequest` | `String (JKT)` | Extracts JKT for session binding.    |
| `/verify`  | `VerifyRequest`            | `String (JKT)` | Validates proof and returns the JKT. |

## Usage

### Prerequisites

* **Redis**: A Redis instance is required for the JTI anti-replay mechanism.

### Configuration

* Ensure `redis_url` spin variable and `DPOP_WINDOW_SECS` environment variable are set.

## Security Guardrails

* **Size Limits**: Payloads exceeding 32 KB are rejected immediately to prevent resource exhaustion
  attacks.
* **Transport**: Only `application/octet-stream` is accepted to enforce binary performance.
* **Algorithm Whitelist**: Explicitly restricted to Ed25519 and P-256 (OKP) to prevent `alg: "none"` or weak
  RSA/EC attacks.

## License

Copyright © 2026 Nexus Project. All rights reserved.