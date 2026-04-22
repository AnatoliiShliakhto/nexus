# Nexus Database

A type-safe `SurrealDB` client built for Rust and specifically optimized for `WebAssembly` (Wasm)
environments using the Spin SDK.

This crate is part of the Nexus ecosystem, prioritizing a "Fortress-First" security philosophy
for mission-critical applications.

---

## Key Features

* **Type-State Builder:** Leverage Rust's type system to ensure database handles are fully configured and authenticated
  before use.
* **FlatBuffers Native:** Uses the binary `FlatBuffers` protocol for high-performance serialization, minimizing
  overhead
  in Wasm execution.
* **Zero-Copy Principles:** Designed for efficiency in memory-constrained environments.
* **Flexible Auth:** Support for both static JWT Bearer tokens and dynamic credential-based sign-in.
* **Spin Integration:** First-class support for `Spin` config variables and environment-based configuration.

## Quick Start

### 1. Configuration

The client provides built-in support for automatic configuration discovery via environment variables or Spin-specific
configuration variables.

#### Supported Environment Variables
These variables are used when calling `Database::builder_from_env()`. This is ideal for local development or traditional
containerized deployments.

| Variable           | Description                             |
|--------------------|-----------------------------------------|
| DATABASE_URL       | The base URL of the SurrealDB instance. |
| DATABASE_NAMESPACE | The target Namespace.                   |
| DATABASE_NAME      | The target Database.                    |

#### Spin Configuration Variables
These are used when calling `Database::builder_from_vars()`. This is the recommended way for production Spin components, as it integrates with Spin's dynamic configuration providers.
Define these in your `spin.toml`:

```toml
[variables]
database_url = { default = "http://localhost:8000" }
database_namespace = { default = "nexus" }
database_name = { default = "core" }

[[component]]
# ...
[component.config]
database_url = "{{ database_url }}"
database_namespace = "{{ database_namespace }}"
database_name = "{{ database_name }}"
```

| Spin Variable Key  | Description                          |
|--------------------|--------------------------------------|
| database_url       | Maps to the database connection URL. |
| database_namespace | Maps to the SurrealDB namespace.     |
| database_name      | Maps to the SurrealDB database name. |

> **Note on Security:** For credentials (username/password/tokens), it is strongly recommended to use DatabaseBuilder::credentials() or DatabaseBuilder::token() at runtime, pulling the actual secrets from a secure vault or encrypted Spin variables.

### 2. Initialize the `Database`

The `Database` handle is thread-safe and designed to be wrapped in an `Arc` or cloned cheaply.

```rust
use nx_database::Database;
use nx_database::error::DatabaseError;

async fn init_db() -> Result<Database, DatabaseError> {
    // Initialize using Spin configuration variables
    let db = Database::builder_from_vars()?
        .credentials("admin", "root_password")?
        .init()
        .await?;

    Ok(db)
}
```

### 3. Execute Queries

The query builder supports parameter binding via `LET` statements to prevent SQL injection.

```rust
use nx_database::Response;
use nx_database::error::DatabaseError;
use surreal_types::{SurrealValue, RecordId};

#[derive(SurrealValue)]
struct User {
    id: RecordId,
    name: String,
    email: String,
}


async fn get_user(db: &Database, user_id: &str) -> Result<Option<User>, DatabaseError> {
    let res = db.query("SELECT * FROM user WHERE id = $id")
        .bind("id", user_id)
        .execute()
        .await?;

    // Validate segments for database-level errors
    let user = res
        .check()?
        .last::<User>()?;

    Ok(user)
}
```

### 4. Handling Transactions

The client is designed to handle multi-segment responses (e.g., blocks containing `BEGIN` and `COMMIT` statements).

```rust,ignore
let sql = r#"
    BEGIN TRANSACTION;
    LET $count = (SELECT count() FROM action);
    CREATE audit SET timestamp = time::now(), total = $count;
    COMMIT TRANSACTION;
"#;

let res = db.query(sql).execute().await?;
// Use .take(index) to grab a specific segment from a transaction
```

## API Overview

| Component         | Description                                                                |
|-------------------|----------------------------------------------------------------------------|
| `Database`        | The main entry point; a cloneable handle to the connection.                |
| `DatabaseBuilder` | A state-machine builder (Empty -> Configured -> Ready).                    |
| `Query`           | A consuming builder for constructing SQL statements and binding variables. |
| `Response`        | A wrapper around multi-segment `SurrealDB` results with lazy decoding.     |
| `DatabaseError`   | A comprehensive error type covering transport, config, and query failures. |

## License

Copyright © 2026 Nexus Project. All rights reserved.