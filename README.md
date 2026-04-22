# Nexus: The Organizational Kernel

![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg?logo=rust)
![WASM](https://img.shields.io/badge/target-wasm32--wasip2-blue?logo=webassembly)
![CI Status](https://img.shields.io/github/actions/workflow/status/AnatoliiShliakhto/nexus/ci.yml?branch=dev&label=build&logo=github)
![Code Coverage](https://codecov.io/gh/AnatoliiShliakhto/nexus/branch/dev/graph/badge.svg)

![Clippy](https://img.shields.io/github/actions/workflow/status/AnatoliiShliakhto/nexus/ci.yml?branch=dev&label=clippy&logo=rust)
![Cargo Audit](https://img.shields.io/github/actions/workflow/status/AnatoliiShliakhto/nexus/ci.yml?branch=dev&label=audit&logo=github)
![Security](https://img.shields.io/badge/security-Vault--Hardened-brightgreen?logo=hashicorpvault)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)
![Version](https://img.shields.io/badge/version-0.1.0-green.svg)

**Nexus** is a high-performance, universal organizational kernel designed to eliminate data silos and infrastructure
redundancy. It acts as a Modular PaaS for the enterprise, providing a centralized, cryptographically secure environment
where developers can deploy business logic as isolated plugins without managing the underlying plumbing.

By abandoning traditional monolithic API servers in favor of a hybrid architecture that unites a **Contract-First
WebAssembly (WASI P2) Component Model** with **classic containerized microservices**, Nexus achieves sub-millisecond
cold starts, instantaneous scaling, and absolute fault isolation.

> **⚠️ WARNING: Early Development (WIP)** \
> Core WASM architectures and event bridges are currently being established.
> APIs and schemas are subject to frequent breaking changes.
> **This project is not yet ready for production use.**

## The Vision: Orchestrating Complexity

Modern organizations suffer from a "software zoo"—fragmented applications that duplicate data, security logic, and
effort. Nexus solves this by offering:

* **For Organizations:** A Single Source of Truth. No duplicate entries; every node in the hierarchy operates on a
  unified, audited data graph.
* **For Developers:** A "Zero-Infrastructure" Framework. Developers focus strictly on business logic. Nexus handles
  identity, matrix-based permissions, high-throughput routing, and persistence.

## Core Architecture

Nexus is built as a distributed system utilizing a robust, data-centric architecture:

### 1. Intelligent Context-Enrichment Gateway

Powered by native Rust **(Axum)** for maximum throughput, the Gateway is the brain of the system. It goes beyond simple
proxying by validating DPoP (RFC 9449) proofs, evaluating complex matrix permissions, and injecting strict security
contexts before requests ever reach the business logic.

### 2. Heterogeneous Plugin Runtime

Nexus supports a hybrid execution model orchestrated by **Fermyon Spin**:

* **Wasm Components:** Ultra-lightweight modules targeting WASI P2/P3 for ephemeral, stateless business logic.
* **Containerized Microservices:** Standard Docker environments for heavy-lifting or stateful workloads, unified under
  the Nexus routing umbrella.

### 3. Immutable Audit & State

Persistence is driven by **SurrealDB**. Schema-level event triggers capture every state mutation automatically,
decoupled from application logic. Institutional structures are modeled as hierarchical graphs, enabling complex "Matrix"
RBAC that scales to any depth.

---

## Fortress-First Security Stack

Security is woven directly into the kernel and leverages WebAssembly's default-deny memory isolation model.

* **Cryptographic Identity:** Ed25519 digital signatures and JWT + DPoP prevent token theft and replay attacks.
* **Hardware-Grade Secrets:** Deep integration with HashiCorp Vault for transit encryption and secret management.
* **Memory Safety:** Strict `#![forbid(unsafe_code)]` compliance and automated zeroization for sensitive cryptographic
  material.

---

## Getting Started

Nexus provides a specialized `xtask` automation suite to abstract the complexity of managing WebAssembly targets,
component builds, and local infrastructure

### Prerequisites

* Rust toolchain (managed via `rust-toolchain.toml`)
* [Fermyon Spin CLI v4+](https://spinframework.dev)
* Docker & Docker Compose

### Bootstrapping the Environment

Clone the repository and bring up the infrastructure stack (SurrealDB, Vault, Redis, MinIO, Tempo, Loki, Jaeger):

```bash
git clone https://github.com/AnatoliiShliakhto/nexus.git
cd nexus

# 1. Install WASM targets and auxiliary tools
cargo setup

# 2. Launch the infrastructure stack
cargo dev up

# 4. Generate component manifests and DB migrations
cargo codegen

# 5. Compile components to wasm32-wasip2
cargo dist components

# 6. Start the Spin development server
cargo serve
```

---

## Developer Workflow

The internal CLI (`cargo x`) provides ergonomic commands for common development tasks:

### 1. Environment Initialization
Setup the local toolchain, including WASM targets and security auditing tools.
```bash 
cargo setup
```

### Infrastructure Management (`dev`)
Orchestrate the backing services (SurrealDB, Redis, Vault, MinIO).
```bash
cargo dev up              # Spin up the full infrastructure stack
cargo dev logs <service>  # Stream logs from a specific container
cargo dev down --volumes  # Tear down and wipe all persistent data
```

### Building & Deployment
Context-aware builds that handle both WASM components and native binaries.
```bash
cargo x build               # Smart-compiles all targets (WASM components & native services)
cargo x build components    # Target only WASI components
cargo x build --release     # Build with production optimizations
cargo lint <component>      # Runs static analysis, formatting checks, and security audits for a specific crate
cargo dist <component>      # Performs a production-ready build with aggressive size optimizations (WASM strip/opt)
cargo serve                 # Launches the Spin runtime to host and test WASM components locally
```

### 4. Workspace Expansion (`add`)
Scaffold new modules with pre-configured templates.
```bash
cargo x add <name> --kind client     # Create a new client Rust crate
cargo x add <name> --kind component  # Create a new WASI plugin
cargo x add <name> --kind library    # Create a shared Rust crate
```

### 5. Utility & Security (`codegen`, `keys`)
Manage the security layer and automated artifacts.
```bash
cargo codegen               # Generate DB migrations and i18n bundles
cargo x keys generate       # Create Ed25519 pairs for JWT/DPoP signing
```

---

## Security & Observability
* **Zero-Trust Identity:** Hardened identity using `Argon2` and `Ed25519` signatures to prevent token theft.
* **Capability-Based Security:** Wasm components operate in a "default-deny" sandbox, accessing host resources only through explicit contracts.
* **Telemetry Mesh:** Integrated OpenTelemetry support for distributed tracing (Tempo), metrics (Prometheus), and logs (Loki).

---

## Workspace Layout

```text
nexus/
├── clients/          # High-fidelity UIs (Desktop/Web/Mobile).
├── components/       # WASI P2 modules for ephemeral, stateless business logic.
├── services/         # Long-running, stateful microservices (Docker/Native).
├── crates/           # The "DNA" of Nexus—shared logic used across all layers.
├── tooling/          # The 'xtask' automation engine and codegen scripts.
├── ops/              # Production & Dev orchestration (Compose, K8s, Vault).
├── examples/         # Reference implementations for new developers.
└── dist/             # Build artifacts and optimized WASM binaries (git-ignored).
```

*For detailed documentation on specific modules, refer to the `README.md` files located within individual **crates/**
and **components/** directories.*

---

## License

This project is dual-licensed under the **MIT License** and the **Apache License (Version 2.0)**.
You may choose to use this software under the terms of either license.

* See [LICENSE-MIT](LICENSE-MIT) for details.
* See [LICENSE-APACHE](LICENSE-APACHE) for details.