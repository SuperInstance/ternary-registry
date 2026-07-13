# Future Integration: ternary-registry

## Current State

ternary-registry provides a ternary service registry for GPU fleets. `ServiceNode` represents a compute node with an ID, capabilities, health status (`Healthy`/`Degraded`/`Failed`), load, and version. `ServiceRegistry` manages registration, deregistration, and capability-based discovery. Discovery filters by health: `discover()` returns all non-failed nodes, `discover_healthy()` returns only healthy nodes, and `discover_least_loaded()` finds the healthiest, least-busy node. A `crdt_merge()` protocol propagates failure state across registry instances.

## Integration Opportunities

### Fleet Health Aggregation

Each GPU node registers with its capabilities and current health. A central coordinator can call `crdt_merge()` across all node-local registries to build a fleet-wide view. Because the merge propagates `Failed` monotonically, any node marked failed by any replica converges to `Failed` everywhere — enabling automatic drain of unhealthy nodes without explicit coordination.

### Load-Aware Scheduling

`discover_least_loaded(capability)` provides a simple load-balancing primitive. A scheduler queries the registry for the least-loaded healthy node capable of handling a given workload (e.g., `"matmul"`, `"attention"`). Nodes report their load via `update_load()`, and the scheduler picks the minimum. Degraded nodes remain discoverable via `discover()` but are excluded from the load-balancing path.

### Health-Driven Circuit Breaking

The ternary health model maps naturally to circuit-breaker states:
- **Healthy (1)**: normal operation, eligible for all discovery paths
- **Degraded (0)**: still serving but excluded from `discover_healthy()` — a "soft degrade" that keeps the node visible without routing new traffic to it
- **Failed (−1)**: completely removed from discovery, triggering failover

External health checks call `update_health()` to transition nodes between states. The registry enforces the invariant: failed nodes are invisible to all discovery methods.

### Multi-Instance Sync

Two `ServiceRegistry` instances can diverge independently (e.g., edge nodes vs. central coordinator). `crdt_merge()` reconciles them:

1. Nodes only in the source registry are added to the target
2. Nodes marked `Failed` in the source propagate `Failed` to the target
3. Non-failed health states are preserved locally (idempotent for the failure semilattice)

This makes merge safe to call repeatedly without risk of "un-failing" a node.

## Dependencies for Next Steps

1. **Persistence layer**: currently in-memory only; needs a checkpoint/restore mechanism
2. **Concurrency**: `ServiceRegistry` is not `Sync`; production use requires `Arc<Mutex<_>>` or a lock-free redesign
3. **Version negotiation**: the `version` field is stored but unused; future work could add semver-based compatibility checks for capability routing
4. **Network transport**: `crdt_merge` operates on in-memory references; a serialization layer (serde) is needed for cross-process sync
