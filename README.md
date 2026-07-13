# ternary-registry

Ternary service registry for GPU fleets. Nodes register their capabilities,
clients discover services by capability, and a CRDT merge protocol keeps
registries synchronized across instances.

## Why This Exists

Distributed GPU fleets need a way to track which compute nodes are available,
what each node can do, and whether each node is healthy. This crate provides a
simple, in-memory registry that answers three questions:

- **Discovery**: which nodes can serve a given capability?
- **Health filtering**: which of those nodes are healthy enough to use?
- **Synchronization**: how do we merge registries from different instances?

Health is modeled as a ternary value — healthy (1), degraded (0), failed (−1) —
matching the ternary computing model used across the SuperInstance ecosystem.

## Core Concepts

| Type | Meaning |
|---|---|
| `Health` | Ternary health status: `Healthy` (1), `Degraded` (0), `Failed` (−1) |
| `ServiceNode` | A registered compute node with capabilities, health, load, and version |
| `ServiceRegistry` | Central registry: register, discover, and merge nodes |

### Health Semantics

| State | Discoverable? | Load-balanced? |
|---|---|---|
| `Healthy` | Yes | Yes (`discover_least_loaded`) |
| `Degraded` | Yes | No (excluded from `discover_healthy`) |
| `Failed` | No | No |

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-registry = "0.1"
```

```rust
use ternary_registry::{Health, ServiceNode, ServiceRegistry};

fn main() {
    let mut reg = ServiceRegistry::new();

    // Register nodes with capabilities
    reg.register(ServiceNode::new("gpu-0", vec!["matmul", "attention"]));
    reg.register(ServiceNode::new("gpu-1", vec!["matmul", "filter"]));

    // Discover all non-failed nodes for a capability
    let nodes = reg.discover("matmul");
    println!("matmul nodes: {}", nodes.len()); // 2

    // Mark a node as failed — it disappears from discovery
    reg.update_health("gpu-0", Health::Failed);
    let healthy = reg.discover("matmul");
    println!("healthy matmul nodes: {}", healthy.len()); // 1

    // Find the least-loaded healthy node for load balancing
    reg.update_load("gpu-1", 0.42);
    if let Some(best) = reg.discover_least_loaded("matmul") {
        println!("least loaded: {}", best.id); // gpu-1
    }
}
```

## API Overview

### ServiceRegistry
- `register(node)` — add or replace a node (duplicates by ID are handled cleanly)
- `deregister(id)` — remove a node and prune its capability index entries
- `discover(capability)` — all non-failed nodes with the given capability
- `discover_healthy(capability)` — only `Healthy` nodes (subset of `discover`)
- `discover_least_loaded(capability)` — the `Healthy` node with the lowest load
- `update_health(id, health)` — change a node's health status
- `update_load(id, load)` — change a node's load value
- `crdt_merge(other)` — merge another registry's nodes into this one
- `node_count()`, `healthy_count()`, `service_count()` — registry statistics

### CRDT Merge Semantics

The `crdt_merge` method propagates failures monotonically: if any replica
marks a node as `Failed`, all replicas converge to `Failed` after merging.
Nodes that exist only in the source registry are added. Non-failed health
states are not overwritten, ensuring merge is idempotent and commutative with
respect to the failure-propagation semilattice.

### ServiceNode
- `new(id, capabilities)` — create a node (defaults: `Healthy`, load `0.0`)
- Fields: `id`, `capabilities`, `health`, `load`, `version` (all public)

## Known Limitations

- **In-memory only**: no persistence layer. State is lost on restart.
- **No concurrent access**: `ServiceRegistry` is not `Sync`. Wrap in
  `Arc<Mutex<_>>` for multi-threaded use.
- **Linear discovery**: `discover` scans the capability index vector. Fine for
  dozens to hundreds of nodes; not indexed for thousands.
- **CRDT merge propagates failures only**: degraded/healthy states from the
  remote registry do not overwrite local state. This is intentional — failures
  are the only monotonic signal.
- **No version negotiation**: the `version` field is stored but not used for
  compatibility checks.

## Ecosystem

Part of the **SuperInstance** ternary computing ecosystem:

- [`ternary`](https://crates.io/crates/ternary) — core trit types and balanced ternary arithmetic
- [`ternary-registry`](https://crates.io/crates/ternary-registry) — this crate
- [`ternary-constraint`](https://crates.io/crates/ternary-constraint) — constraint satisfaction for ternary variables
- [`ternary-control`](https://crates.io/crates/ternary-control) — ternary control theory
- [`ternary-sensor`](https://crates.io/crates/ternary-sensor) — sensor classification and fusion

## License

MIT
