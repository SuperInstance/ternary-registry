# ternary-registry

Capability and skill registry with semantic versioning, dependency resolution, and multi-instance synchronization.

## Why This Exists

Distributed systems with ternary decision-making need a way to register, discover, and resolve capabilities across instances. When multiple nodes each have a set of skills with version constraints and dependencies, you need a registry that handles:

- **Discovery**: find skills by name, tier, or capability
- **Dependency resolution**: topological sort with cycle detection
- **Version compatibility**: semver-aware constraint checking
- **Synchronization**: detect drift and conflicts between local and remote registries

While this crate doesn't directly compute on ternary values, it serves as the **infrastructure backbone** for the SuperInstance ecosystem — tracking which ternary capabilities each instance provides, ensuring dependencies are met before activation, and keeping distributed registries in sync.

## Core Concepts

| Type | Meaning |
|---|---|
| `SkillTier` | Capability level: `Basic` (0) → `Standard` (1) → `Advanced` (2) → `Expert` (3) |
| `SkillId` | Fully qualified skill identifier: `namespace::name@version` |
| `SemVersion` | Semantic version (major.minor.patch) with compatibility checks |
| `Skill` | A registered capability with tier, description, dependencies, and capabilities |
| `SkillRegistry` | Central registry: register, query, and discover skills |
| `CapabilityMatrix` | Maps tiers to their allowed capabilities (read, write, network, admin, etc.) |
| `SkillDependencyResolver` | Topological sort with cycle detection |
| `VersionConstraint` | Range-based semver constraint checking |
| `RegistrySync` | Detects drift, conflicts, and sync needs between instances |

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-registry = "0.1"
```

```rust
use ternary_registry::*;

fn main() {
    let mut reg = SkillRegistry::new();

    // Register a skill with dependencies
    let dep_id = SkillId::new("core", "math", SemVersion::new(1, 0, 0));
    reg.register(Skill::new(dep_id.clone(), SkillTier::Basic, "Basic math ops"));

    let main_id = SkillId::new("ai", "kalman", SemVersion::new(1, 2, 0));
    reg.register(
        Skill::new(main_id.clone(), SkillTier::Advanced, "Kalman filter")
            .with_dependency(dep_id.clone())
            .with_capability("estimation")
    );

    // Query by capability
    let skills = reg.find_by_capability("estimation");
    println!("Estimation skills: {}", skills.len());

    // Resolve dependencies (topological order)
    let resolver = SkillDependencyResolver::new(reg.clone());
    let order = resolver.resolve(&main_id).unwrap();
    println!("Load order: {:?}", order.iter().map(|s| &s.name).collect::<Vec<_>>());

    // Check capabilities
    let matrix = CapabilityMatrix::new();
    println!("Can Advanced write? {}", matrix.supports(SkillTier::Advanced, "write"));
}
```

## API Overview

### SkillRegistry
- `register(skill) → bool` — add a skill (rejects duplicates)
- `unregister(id) → bool` — remove a skill
- `get(id)`, `get_mut(id)` — direct lookup
- `find_by_name(query)`, `find_by_tier(tier)`, `find_by_capability(cap)` — search
- `query(&SkillQuery)` — multi-criteria filtering

### SkillQuery (builder pattern)
- `SkillQuery::new().name("kalman").min_tier(SkillTier::Advanced).capability("math")`

### CapabilityMatrix
- `supports(tier, capability) → bool` — check tier → capability mapping
- `min_tier_for(capability) → Option<SkillTier>` — find minimum required tier
- `supports_all(tier, &[caps])` — batch check

### SkillDependencyResolver
- `resolve(skill_id) → Result<Vec<SkillId>, String>` — topological sort, detects cycles

### VersionConstraint
- `at_least(min)` / `range(min, max)` — define constraints
- `satisfies(&version) → bool` — check compatibility

### RegistrySync
- `check_status() → SyncStatus` — InSync / Behind / Ahead / Conflict
- `skills_to_pull()`, `skills_to_push()` — diff local vs remote
- `detect_conflicts() → Vec<String>` — same name, different versions

## How It Works

The registry stores skills in a `HashMap<String, Skill>` keyed by their fully qualified ID (`namespace::name@version`). Queries are executed as linear scans with predicate filters — appropriate for the typical registry size of dozens to hundreds of skills.

Dependency resolution uses recursive depth-first search with a visiting set to detect cycles. The result is a topological ordering: load dependencies before dependents. Circular dependencies are reported as errors with the offending skill ID.

Synchronization compares local and remote skill sets by version. Two registries are "in sync" when their version counters match. When they differ, the sync module computes the set difference in both directions and detects conflicts where the same skill has diverged to different versions.

## Use Cases

- **Plugin system** — register capabilities at runtime, resolve dependencies before loading, enforce tier-based access control
- **Distributed skill discovery** — nodes advertise skills, sync registries, detect conflicts, and pull missing capabilities
- **CI/CD capability gates** — ensure deployment targets have all required capabilities at compatible versions before proceeding

## Ecosystem

Part of the **SuperInstance** ternary computing ecosystem:

- [`ternary`](https://crates.io/crates/ternary) — core trit types and balanced ternary arithmetic
- [`ternary-registry`](https://crates.io/crates/ternary-registry) — this crate
- [`ternary-constraint`](https://crates.io/crates/ternary-constraint) — constraint satisfaction for ternary variables
- [`ternary-control`](https://crates.io/crates/ternary-control) — ternary control theory
- [`ternary-sensor`](https://crates.io/crates/ternary-sensor) — sensor classification and fusion

## License

MIT
