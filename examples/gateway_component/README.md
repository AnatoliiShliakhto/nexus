# Nexus Gateway

The `nx-gateway` crate is an API Gateway and Request Router engineered for the Nexus.
Acting as the "Fortress Gate," it serves as the single, secure entry point for all incoming traffic, leveraging
Wasm-native execution to provide sub-millisecond routing and identity enforcement.

---

## Overview

`nx-gateway` centralizes the complexity of distributed microservices by handling request lifecycle management at the
edge. It ensures that backend services remain "thin" and focused on business logic by offloading identity verification,
session management, and protocol translation.

## Key Features

* **Dynamic Routing:** Automatically resolves routes using the internal `Route` engine.
* **Request Rebasing:** Rewrites URIs and prefixes on the fly to match backend service expectations.
* **Wasm-Native:** Compiled to `wasm32-wasip2` for near-native performance and sandboxed security.
* **Config-Driven:** Pulls service URLs from the Spin configuration provider, allowing for
  environment-specific deployments without code changes.

## Tech Stack

* **Runtime:** [Fermyon Spin](https://spin.fermyon.dev/)
* **Language:** Rust
* **Target:** `wasm32-wasip2` (WASI)
* **Crates:** `nx-http`

## License

Copyright © 2026 Nexus Project. All rights reserved.