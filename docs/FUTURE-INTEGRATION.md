# Future Integration: ternary-registry

## Current State

ternary-registry provides capability and skill registry for construct-core integration. `SkillId` (namespace::name@version) identifies skills with `SemVersion`. `Skill` has tier (`Basic`/`Standard`/`Advanced`/`Expert`), description, dependencies, and capabilities. `SkillRegistry` manages registration, lookup, and querying (`find_by_name`, `find_by_tier`, `find_by_capability`, multi-criteria `query()`). `SkillDependencyResolver` performs topological sort with circular dependency detection. `CapabilityMatrix` defines tier → capability mappings (Basic: read/query, Expert: all + admin/delegate). `RegistrySync` tracks local vs remote versions with conflict detection.

## Integration Opportunities

### Equipment Pattern Bridge (Primary Integration)

The Cross-Pollination Report maps TypeScript `EquipmentSlot` to `SkillTier`: Memory/Communication → Basic, Spreadsheet/Distillation/Monitoring → Standard, Reasoning/Perception → Advanced, Consensus/Coordination/SelfImprovement → Expert. The `SkillRegistry` IS the equipment inventory. When `OriginCore` (from SuperInstance-Starter-Agent) needs a skill:

1. `SkillDependencyResolver::resolve(skill_id)` returns the topological load order
2. Each dependency is loaded via construct-core's `load_skill()`
3. `CapabilityMatrix::supports_all(tier, capabilities)` validates the hardware can run it

### Ensign Pattern → Skill-Backed Specialists

Every `ternary-ensign::Ensign` maps to a `Skill` in the registry. The `EnsignBridge` (domain → skill_name) uses `SkillRegistry::find_by_name()` to locate the skill. The ensign's `required_skills()` maps to `Skill::dependencies`. When an ensign is loaded into a room, `SkillDependencyResolver` ensures all dependencies are available on the hardware tier. If `CapabilityMatrix::supports(tier, "network")` returns false for the current tier, network-dependent ensigns can't load.

### RegistrySync → Fleet-Wide Skill Distribution

`RegistrySync` with its `check_status()` (InSync/Behind/Ahead/Conflict), `skills_to_pull()`, `skills_to_push()`, and `detect_conflicts()` enables fleet-wide skill distribution:

1. Oracle1 (PLATO) holds the master registry
2. Each Codespace/Edge room syncs on entry: `RegistrySync::check_status()`
3. New skills are pulled: `skills_to_pull()` → `SkillDependencyResolver::resolve()` → download
4. Conflicts detected: `detect_conflicts()` → version negotiation via `VersionConstraint`

### linguistic-polyformalism → 7-Type Capability Auditing

The 7 constraint types from `linguistic-polyformalism-shell` (Boundary, Pattern, ProcessShape, KnowledgeSource, SocialStructure, DeepStructure, Instrument) become the `Skill::capabilities` taxonomy. A skill with all 7 types declared has complete self-description. `SkillRegistry::find_by_capability("Boundary")` finds all boundary-aware skills. Missing types indicate blind spots.

## Potential in Mature Systems

`SkillRegistry` becomes the package manager for the ternary ecosystem. Every crate, every ensign, every construct-core skill registers here. `SkillDependencyResolver` prevents circular dependencies across the entire fleet. `CapabilityMatrix` ensures skills only run on hardware that supports them. `RegistrySync` keeps all rooms in sync. `VersionConstraint` enables rolling upgrades without breaking compatibility.

## Cross-Pollination Ideas

- **ternary-locks → Skill access control**: `Lock` patterns gate skill access. A `LockComposition::And([Capability::Admin, Capability::Delegate])` = only expert-tier rooms can load admin skills.
- **ternary-econ → Skill pricing**: `PortfolioOptimizer` determines which skills to load based on cost/benefit. Skills with high invocation count but low resource cost get priority.
- **Skill registry → crates.io mirror**: `SkillRegistry` mirrors crates.io packages as skills. Every published ternary crate auto-registers.

## Dependencies for Next Steps

1. `SkillRegistry` → `construct-core` `load_skill()` integration
2. `RegistrySync` → PLATO tile store protocol
3. `CapabilityMatrix` → hardware tier auto-detection
4. Fleet-wide skill resolution across org boundaries
5. `VersionConstraint` → semantic version policy enforcement
