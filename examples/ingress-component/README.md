# Nexus Ingress Component

## Overview

`nx-ingress` is the **front door** of the *Nexus*. It acts as the primary entry
point for all external HTTP traffic before it reaches the `nx-gateway`.

Its main responsibilities are:

1. **Health Monitoring:** Provides a `/health` endpoint for infrastructure probes (Kubernetes,
   Nomad, etc.).
2. **Traffic Forwarding:** Seamlessly proxies all other requests to the internal Gateway.

## Tech Stack

* **Runtime:** [Fermyon Spin](https://spin.fermyon.dev/)
* **Target:** `wasm32-wasip2`
* **Communication:** Internal Service Chaining via `spin.internal`

## Routing Logic

The ingress logic is intentionally thin to ensure maximum throughput:

* `GET /health` → Returns `200 OK` (Used by load balancers).
* `ANY /*` → Rebases the URI and forwards the request to the **Gateway URL** (default:
  `http://nx-gateway.spin.internal`).

## Configuration

You can override the target gateway URL using the Spin configuration:

| Key                        | Default Value                     | Description                                          |
|:---------------------------|:----------------------------------|:-----------------------------------------------------|
| `nx_gateway_component_url` | `http://nx-gateway.spin.internal` | The internal address of the Nexus Gateway component. |

## License

Copyright © 2026 Nexus Project. All rights reserved.