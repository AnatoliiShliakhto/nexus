# Nexus DPoP-API

A lightweight contract crate defining the Data Transfer Objects (DTOs) for DPoP
(Demonstrating Proof-of-Possession) validation within the Nexus (User Management System) ecosystem.

This crate is designed specifically for WASI and Spin components, using binary serialization for
near-instant inter-component communication.

## Overview

DPoP (RFC 9449) is a mechanism that binds an Access Token to a specific client key pair. This crate
provides the shared structures that allow the Auth Service, API Gateway (Router), and DPoP
Validation Component to communicate seamlessly.

## Key Structures

* **ComputeThumbprintRequest**: Used by the Authentication service to generate a JWK Thumbprint
  (JKT) from a proof. This JKT is then used to bind a user session to a specific public key.
* **VerifyRequest**:Used by the Router/Gateway to validate the DPoP proof for every protected
  resource request against the current HTTP context.
* **Jkt**: A type alias for the Base64URL-encoded SHA-256 thumbprint of the client's public key (RFC
  7638).

## Usage

### 1. Adding to your Component

Add this to your `Cargo.toml`:

```toml
[dependencies]
nx-dpop-api = { path = "../../examples/nx-dpop-api-lib" }
postcard = "1.0" # Recommended binary serializer
```

### 2. In the Auth Service (Thumbprint Generation)

When a user logs in, send the DPoP proof to the validation service to compute the thumbprint required for session binding:

```rust,ignore
use nx_dpop_api::ComputeThumbprintRequest;

let thumbprint_req = ComputeThumbprintRequest::new(dpop_header_value);
let payload = postcard::to_allocvec(&thumbprint_req)?;

// Send to /compute endpoint...
```

### 3. In the Router (Verification)

For every incoming request, verify the proof against the HTTP context:

```rust,ignore
use nx_dpop_api::VerifyRequest;

let verify_req = VerifyRequest::new(
    dpop_header,
    req.method(),
    req.uri(),
    Some(access_token)
);

let payload = postcard::to_allocvec(&verify_req)?;
// Send to nx-dpop component...
```

## Protocol Implementation

This crate is protocol-agnostic but optimized for Postcard over HTTP (spin.internal service
chaining).

1. **Success:** The DPoP service returns a 200 OK with the Jkt (String) serialized via Postcard.
2. **Failure:** The DPoP service returns a non-200 status code with a human-readable error message
   in the body.

## License

Copyright © 2026 Nexus Project. All rights reserved.