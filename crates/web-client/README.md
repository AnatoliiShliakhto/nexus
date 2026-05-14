# Nexus Web Client

A high-performance, strictly typed HTTP client for the **NEXUS** ecosystem, specifically optimized for **WebAssembly (Wasm)**
and **Dioxus** frontend environments.

This crate is part of the Nexus ecosystem, prioritizing a **"Fortress-First"** security philosophy for mission-critical
applications. It automates complex security protocols like **DPoP (RFC 9449)** and provides a reactive session management
system.

## Key Features
* **Automated DPoP (RFC 9449):** Transparently generates ephemeral Ed25519 keys and signs proofs for every request to ensure sender-constrained access tokens.
* **Proactive Token Refresh:** Built-in state machine that monitors token expiration and triggers refresh flows before requests fail.
* **Secure Persistence:** Deep integration with `nx-lockbox` for encrypted storage of refresh tokens and session data.
* **Dioxus Native:** Provides reactive `watch` channels for session state, allowing UI components to respond instantly to auth changes.
* **High Performance:** Leverages `arc-swap` for lock-free state access and `fxhash` to minimize overhead in high-frequency operations.

## Quick Start

### 1. Initialization
The WebClient is designed to be initialized once and shared across your application (usually wrapped in an Arc or provided via Dioxus use_context).

```rust

use nx_web_client::WebClient;
use url::Url;

fn main() {
    let base_url = Url::parse("https://api.nexus.gateway").unwrap();
    
    // "portal-web" is the service name used for Lockbox isolation
    let client = WebClient::init("portal-web", base_url)
        .expect("Failed to initialize Web Client");
}
```

### 2. Authentication
The client handles the exchange of credentials for DPoP-bound tokens and automatically persists the session.

```rust
async fn login(client: &WebClient) -> Result<(), WebClientError> {
    client.login_with_credentials("administrator", "secure_password").await?;
    
    println!("Logged in and session stored in Lockbox!");
    Ok(())
}
```

### 3. Executing Authorized Requests
Methods like `get`, `post`, `put`, and `delete` automatically inject DPoP proofs and Authorization headers.

```rust
use nx_web_client::{Json, ResponseExt};
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct CreateTask {
    title: String,
}

#[derive(Deserialize)]
struct TaskResponse {
    id: String,
}

async fn create_new_task(client: &WebClient) -> Result<TaskResponse, WebClientError> {
    let body = Json(CreateTask { title: "Deploy Nexus".to_owned() });
    
    // .resolve() handles status checks and JSON deserialization
    let task = client.post("/api/v1/tasks", body)
        .await?
        .resolve::<TaskResponse>()
        .await?;
        
    Ok(task)
}
```

### 4. Reactive Session Tracking (Dioxus)
Use the `session_rx` to listen for real-time changes in permissions or session status.

```rust,ignore
let mut session_rx = client.session_rx();

tokio::spawn(async move {
    while session_rx.changed().await.is_ok() {
        let session = session_rx.borrow();
        println!("Current Session Status: {:?}", session.status);
    }
});
```

## API Overview

| Component      | Description                                                                              |
|----------------|------------------------------------------------------------------------------------------|
| WebClient      | The main entry point; thread-safe handle for all HTTP and Auth operations.               |
| RequestBody    | Enum for various payload types (JSON, Multipart, Forms, Bytes).                          |
| ResponseExt    | Extension trait for `reqwest::Response` to provide the async `.resolve<T>()` method.     |
| Session        | Contains current [`SessionStatus`] and granular [`Actions`] (permissions) per component. |
| Actions        | A bitmask (u8) representing `CREATE`, `READ`, `UPDATE`, `DELETE` permissions.            |
| WebClientError | A comprehensive error type integrating with `nx-error` for detailed API feedback.        |

## Feature Flags
* 
* `dioxus`: Enables integration features and helper types for the Dioxus framework.

## License

Copyright © 2026 Nexus Project. All rights reserved.