# Project Architecture: Nexus

Nexus is "The Organizational Kernel," a modular system built with Rust, focusing on WASM, WASI, and the Spin framework.

## 🏗️ System Overview

The project is organized as a Rust workspace with the following structure:

### 🧩 Components (`components/`)
These are the core business logic units, likely deployed as WASM components.
- **access**: Manages permissions and access control.
- **account**: Handles user accounts and profiles.
- **audit**: Provides auditing and logging of system actions.
- **auth**: Authentication logic (DPoP, JWT, etc.).
- **organization**: Manages organizational structures (tenants, teams, etc.).

### 📦 Shared Libraries (`crates/`)
Utility and infrastructure crates used across the workspace.
- **config**: Shared configuration management.
- **database**: Database abstraction layer (SurrealDB, Redis).
- **error**: Centralized error handling types and logic.
- **error-macros**: Procedural macros for error generation.
- **http**: HTTP utilities and middleware (Axum-based).
- **http-macros**: Procedural macros for HTTP-related boilerplate.
- **i18n**: Internationalization and localization support.
- **lockbox**: Likely handles sensitive data or secret management.
- **logger**: Unified logging and tracing (OpenTelemetry).
- **ui**: Shared UI components and styles (Dioxus-based).
- **web-client**: Frontend API client library.

### 💻 Clients (`clients/`)
- **console**: A Dioxus-based console application for interacting with the Nexus system.

### 🌐 Services (`services/`)
- **gateway**: Infrastructure for request routing and potentially an API gateway.

### 🧪 Examples (`examples/`)
Demonstration components and templates for various integration patterns:
- `dpop-api-lib`, `dpop-component`: Proof-of-Possession implementation examples.
- `gateway_component`, `ingress-component`: Networking and routing examples.
- `showcase`: A Dioxus UI showcase.
- `spin-component`, `wasi-component`: Low-level WASM/WASI component templates.

### 🛠️ Tooling (`tooling/`)
- **xtask**: Custom build and management tasks (using the `cargo-xtask` pattern).

### 📖 Documentation & Ops (`docs/`, `ops/`)
- **docs/articles**: In-depth technical articles about Nexus components (e.g., `nx-error`, `nx-logger`).
- **ops**: Operational configurations and environment setups (e.g., `nexus-dev` with Alloy).

---

## ⚙️ Key Technologies

- **Rust**: Primary programming language (Edition 2024).
- **WebAssembly (WASM)**: Target for components.
- **Spin**: Framework for building and running WASM microservices.
- **WASI**: WebAssembly System Interface for component interoperability.
- **SurrealDB**: Multi-model cloud database.
- **Dioxus**: Fullstack GUI library for web, desktop, and mobile.
- **OpenTelemetry**: Observability (tracing, metrics, logging).

## 🚀 Operational Details

- **Deployment**: Uses Spin (`spin.toml`, `spin.template.toml`).
- **Configuration**: `gateway.config.toml` for gateway settings.
- **Linting**: Strict clippy and rustfmt rules defined in `clippy.toml`, `rustfmt.toml`, and `deny.toml`.
