#![forbid(unsafe_code)]

//! Capability and skill registry for construct-core integration.

use std::collections::HashMap;

/// Skill tiers matching construct-core capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SkillTier {
    Basic,
    Standard,
    Advanced,
    Expert,
}

impl SkillTier {
    pub fn level(&self) -> u8 {
        match self {
            SkillTier::Basic => 0,
            SkillTier::Standard => 1,
            SkillTier::Advanced => 2,
            SkillTier::Expert => 3,
        }
    }

    pub fn from_level(level: u8) -> Option<Self> {
        match level {
            0 => Some(SkillTier::Basic),
            1 => Some(SkillTier::Standard),
            2 => Some(SkillTier::Advanced),
            3 => Some(SkillTier::Expert),
            _ => None,
        }
    }
}

/// Skill ID matching construct-core's SkillId enum.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SkillId {
    pub namespace: String,
    pub name: String,
    pub version: SemVersion,
}

impl SkillId {
    pub fn new(namespace: &str, name: &str, version: SemVersion) -> Self {
        Self {
            namespace: namespace.to_string(),
            name: name.to_string(),
            version,
        }
    }

    pub fn full_id(&self) -> String {
        format!("{}::{}@{}", self.namespace, self.name, self.version)
    }
}

/// Semantic versioning.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SemVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn is_compatible_with(&self, other: &SemVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl std::fmt::Display for SemVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A registered skill.
#[derive(Clone, Debug)]
pub struct Skill {
    pub id: SkillId,
    pub tier: SkillTier,
    pub description: String,
    pub dependencies: Vec<SkillId>,
    pub capabilities: Vec<String>,
}

impl Skill {
    pub fn new(id: SkillId, tier: SkillTier, description: &str) -> Self {
        Self {
            id,
            tier,
            description: description.to_string(),
            dependencies: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, dep: SkillId) -> Self {
        self.dependencies.push(dep);
        self
    }

    pub fn with_capability(mut self, cap: &str) -> Self {
        self.capabilities.push(cap.to_string());
        self
    }
}

/// Skill registry: register, discover, and query skills.
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: Skill) -> bool {
        let key = skill.id.full_id();
        if self.skills.contains_key(&key) {
            return false;
        }
        self.skills.insert(key, skill);
        true
    }

    pub fn unregister(&mut self, id: &SkillId) -> bool {
        self.skills.remove(&id.full_id()).is_some()
    }

    pub fn get(&self, id: &SkillId) -> Option<&Skill> {
        self.skills.get(&id.full_id())
    }

    pub fn get_mut(&mut self, id: &SkillId) -> Option<&mut Skill> {
        self.skills.get_mut(&id.full_id())
    }

    /// Find skills by name (partial match).
    pub fn find_by_name(&self, query: &str) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.id.name.contains(query))
            .collect()
    }

    /// Find skills by tier.
    pub fn find_by_tier(&self, tier: SkillTier) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.tier == tier)
            .collect()
    }

    /// Find skills by capability.
    pub fn find_by_capability(&self, cap: &str) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| s.capabilities.iter().any(|c| c.contains(cap)))
            .collect()
    }

    /// Query skills matching all criteria.
    pub fn query(&self, query: &SkillQuery) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|s| {
                if let Some(ref name) = query.name_contains {
                    if !s.id.name.contains(name) {
                        return false;
                    }
                }
                if let Some(tier) = query.min_tier {
                    if s.tier.level() < tier.level() {
                        return false;
                    }
                }
                if let Some(ref cap) = query.capability {
                    if !s.capabilities.iter().any(|c| c.contains(cap)) {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

/// Query parameters for skill search.
pub struct SkillQuery {
    pub name_contains: Option<String>,
    pub min_tier: Option<SkillTier>,
    pub capability: Option<String>,
}

impl SkillQuery {
    pub fn new() -> Self {
        Self {
            name_contains: None,
            min_tier: None,
            capability: None,
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name_contains = Some(name.to_string());
        self
    }

    pub fn min_tier(mut self, tier: SkillTier) -> Self {
        self.min_tier = Some(tier);
        self
    }

    pub fn capability(mut self, cap: &str) -> Self {
        self.capability = Some(cap.to_string());
        self
    }
}

/// Capability matrix: what each tier supports.
pub struct CapabilityMatrix {
    matrix: HashMap<SkillTier, Vec<String>>,
}

impl CapabilityMatrix {
    pub fn new() -> Self {
        let mut m = Self {
            matrix: HashMap::new(),
        };
        // Default capabilities per tier
        m.matrix.insert(SkillTier::Basic, vec!["read".into(), "query".into()]);
        m.matrix.insert(SkillTier::Standard, vec!["read".into(), "query".into(), "write".into(), "compute".into()]);
        m.matrix.insert(SkillTier::Advanced, vec!["read".into(), "query".into(), "write".into(), "compute".into(), "network".into(), "persist".into()]);
        m.matrix.insert(SkillTier::Expert, vec!["read".into(), "query".into(), "write".into(), "compute".into(), "network".into(), "persist".into(), "admin".into(), "delegate".into()]);
        m
    }

    pub fn supports(&self, tier: SkillTier, capability: &str) -> bool {
        self.matrix
            .get(&tier)
            .map(|caps| caps.iter().any(|c| c == capability))
            .unwrap_or(false)
    }

    pub fn capabilities_for(&self, tier: SkillTier) -> &[String] {
        self.matrix.get(&tier).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Check if tier supports all given capabilities.
    pub fn supports_all(&self, tier: SkillTier, capabilities: &[&str]) -> bool {
        capabilities.iter().all(|c| self.supports(tier, c))
    }

    /// Find minimum tier that supports a capability.
    pub fn min_tier_for(&self, capability: &str) -> Option<SkillTier> {
        for tier in [SkillTier::Basic, SkillTier::Standard, SkillTier::Advanced, SkillTier::Expert] {
            if self.supports(tier, capability) {
                return Some(tier);
            }
        }
        None
    }
}

/// Skill dependency resolver.
pub struct SkillDependencyResolver {
    registry: SkillRegistry,
}

impl SkillDependencyResolver {
    pub fn new(registry: SkillRegistry) -> Self {
        Self { registry }
    }

    /// Resolve dependencies for a skill, returning ordered list (topological sort).
    pub fn resolve(&self, skill_id: &SkillId) -> Result<Vec<SkillId>, String> {
        let mut resolved = Vec::new();
        let mut visited = Vec::new();
        self.resolve_recursive(skill_id, &mut resolved, &mut visited)?;
        Ok(resolved)
    }

    fn resolve_recursive(
        &self,
        skill_id: &SkillId,
        resolved: &mut Vec<SkillId>,
        visiting: &mut Vec<String>,
    ) -> Result<(), String> {
        let key = skill_id.full_id();
        if resolved.iter().any(|r| r.full_id() == key) {
            return Ok(());
        }
        if visiting.contains(&key) {
            return Err(format!("Circular dependency detected: {}", key));
        }
        visiting.push(key.clone());

        if let Some(skill) = self.registry.get(skill_id) {
            for dep in &skill.dependencies {
                self.resolve_recursive(dep, resolved, visiting)?;
            }
            resolved.push(skill_id.clone());
        } else {
            return Err(format!("Skill not found: {}", key));
        }

        visiting.pop();
        Ok(())
    }
}

/// Version constraint checking.
pub struct VersionConstraint {
    pub min_version: SemVersion,
    pub max_version: Option<SemVersion>,
}

impl VersionConstraint {
    pub fn at_least(min: SemVersion) -> Self {
        Self {
            min_version: min,
            max_version: None,
        }
    }

    pub fn range(min: SemVersion, max: SemVersion) -> Self {
        Self {
            min_version: min,
            max_version: Some(max),
        }
    }

    pub fn satisfies(&self, version: &SemVersion) -> bool {
        if version.major != self.min_version.major {
            return false;
        }
        if version.minor < self.min_version.minor {
            return false;
        }
        if version.minor == self.min_version.minor && version.patch < self.min_version.patch {
            return false;
        }
        if let Some(ref max) = self.max_version {
            if version.minor > max.minor {
                return false;
            }
            if version.minor == max.minor && version.patch > max.patch {
                return false;
            }
        }
        true
    }
}

/// Registry sync status between instances.
#[derive(Clone, Debug, PartialEq)]
pub enum SyncStatus {
    InSync,
    Behind(u32),
    Ahead(u32),
    Conflict(Vec<String>),
}

/// Registry synchronizer between instances.
pub struct RegistrySync {
    local_version: u64,
    remote_version: u64,
    local_skills: Vec<SkillId>,
    remote_skills: Vec<SkillId>,
}

impl RegistrySync {
    pub fn new(local_version: u64, remote_version: u64) -> Self {
        Self {
            local_version,
            remote_version,
            local_skills: Vec::new(),
            remote_skills: Vec::new(),
        }
    }

    pub fn set_local(&mut self, skills: Vec<SkillId>) {
        self.local_skills = skills;
    }

    pub fn set_remote(&mut self, skills: Vec<SkillId>) {
        self.remote_skills = skills;
    }

    pub fn check_status(&self) -> SyncStatus {
        if self.local_version == self.remote_version {
            return SyncStatus::InSync;
        }
        if self.local_version < self.remote_version {
            return SyncStatus::Behind((self.remote_version - self.local_version) as u32);
        }
        SyncStatus::Ahead((self.local_version - self.remote_version) as u32)
    }

    /// Find skills in remote not in local.
    pub fn skills_to_pull(&self) -> Vec<&SkillId> {
        self.remote_skills
            .iter()
            .filter(|r| !self.local_skills.iter().any(|l| l.full_id() == r.full_id()))
            .collect()
    }

    /// Find skills in local not in remote.
    pub fn skills_to_push(&self) -> Vec<&SkillId> {
        self.local_skills
            .iter()
            .filter(|l| !self.remote_skills.iter().any(|r| r.full_id() == l.full_id()))
            .collect()
    }

    /// Detect conflicts (same skill ID, different versions).
    pub fn detect_conflicts(&self) -> Vec<String> {
        let mut conflicts = Vec::new();
        for local in &self.local_skills {
            for remote in &self.remote_skills {
                if local.namespace == remote.namespace && local.name == remote.name
                    && local.version != remote.version
                {
                    conflicts.push(format!(
                        "{}::{} local={} remote={}",
                        local.namespace, local.name, local.version, remote.version
                    ));
                }
            }
        }
        conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str, tier: SkillTier) -> Skill {
        Skill::new(
            SkillId::new("test", name, SemVersion::new(1, 0, 0)),
            tier,
            "test skill",
        )
    }

    #[test]
    fn test_skill_tier_level() {
        assert_eq!(SkillTier::Basic.level(), 0);
        assert_eq!(SkillTier::Standard.level(), 1);
        assert_eq!(SkillTier::Advanced.level(), 2);
        assert_eq!(SkillTier::Expert.level(), 3);
    }

    #[test]
    fn test_skill_tier_from_level() {
        assert_eq!(SkillTier::from_level(0), Some(SkillTier::Basic));
        assert_eq!(SkillTier::from_level(4), None);
    }

    #[test]
    fn test_sem_version_compatible() {
        let v1 = SemVersion::new(1, 2, 0);
        let v2 = SemVersion::new(1, 3, 0);
        assert!(v1.is_compatible_with(&v1));
        assert!(!v1.is_compatible_with(&v2));
        assert!(v2.is_compatible_with(&v1));
    }

    #[test]
    fn test_sem_version_display() {
        let v = SemVersion::new(2, 1, 3);
        assert_eq!(format!("{}", v), "2.1.3");
    }

    #[test]
    fn test_skill_registry_register() {
        let mut reg = SkillRegistry::new();
        let skill = make_skill("foo", SkillTier::Basic);
        assert!(reg.register(skill));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_skill_registry_duplicate() {
        let mut reg = SkillRegistry::new();
        let s1 = make_skill("foo", SkillTier::Basic);
        let s2 = make_skill("foo", SkillTier::Standard);
        assert!(reg.register(s1));
        assert!(!reg.register(s2));
    }

    #[test]
    fn test_skill_registry_unregister() {
        let mut reg = SkillRegistry::new();
        let skill = make_skill("foo", SkillTier::Basic);
        let id = skill.id.clone();
        reg.register(skill);
        assert!(reg.unregister(&id));
        assert!(reg.is_empty());
    }

    #[test]
    fn test_skill_registry_find_by_name() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("compute", SkillTier::Standard));
        reg.register(make_skill("network", SkillTier::Advanced));
        let results = reg.find_by_name("comp");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_skill_registry_find_by_tier() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("a", SkillTier::Basic));
        reg.register(make_skill("b", SkillTier::Advanced));
        let results = reg.find_by_tier(SkillTier::Advanced);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_skill_registry_find_by_capability() {
        let mut reg = SkillRegistry::new();
        let s = make_skill("foo", SkillTier::Standard).with_capability("math");
        reg.register(s);
        let results = reg.find_by_capability("math");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_skill_query() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("alpha", SkillTier::Basic));
        reg.register(make_skill("alpha_adv", SkillTier::Advanced));
        let query = SkillQuery::new().name("alpha").min_tier(SkillTier::Advanced);
        let results = reg.query(&query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_capability_matrix_basic() {
        let cm = CapabilityMatrix::new();
        assert!(cm.supports(SkillTier::Basic, "read"));
        assert!(!cm.supports(SkillTier::Basic, "write"));
    }

    #[test]
    fn test_capability_matrix_advanced() {
        let cm = CapabilityMatrix::new();
        assert!(cm.supports(SkillTier::Advanced, "network"));
        assert!(cm.supports(SkillTier::Expert, "admin"));
    }

    #[test]
    fn test_capability_matrix_min_tier() {
        let cm = CapabilityMatrix::new();
        assert_eq!(cm.min_tier_for("write"), Some(SkillTier::Standard));
        assert_eq!(cm.min_tier_for("admin"), Some(SkillTier::Expert));
    }

    #[test]
    fn test_capability_matrix_supports_all() {
        let cm = CapabilityMatrix::new();
        assert!(cm.supports_all(SkillTier::Standard, &["read", "write"]));
        assert!(!cm.supports_all(SkillTier::Basic, &["read", "write"]));
    }

    #[test]
    fn test_version_constraint_satisfies() {
        let vc = VersionConstraint::at_least(SemVersion::new(1, 2, 0));
        assert!(vc.satisfies(&SemVersion::new(1, 2, 0)));
        assert!(vc.satisfies(&SemVersion::new(1, 3, 0)));
        assert!(!vc.satisfies(&SemVersion::new(1, 1, 0)));
        assert!(!vc.satisfies(&SemVersion::new(2, 0, 0)));
    }

    #[test]
    fn test_version_constraint_range() {
        let vc = VersionConstraint::range(SemVersion::new(1, 1, 0), SemVersion::new(1, 3, 0));
        assert!(vc.satisfies(&SemVersion::new(1, 2, 0)));
        assert!(!vc.satisfies(&SemVersion::new(1, 4, 0)));
    }

    #[test]
    fn test_dependency_resolution() {
        let mut reg = SkillRegistry::new();
        let dep_id = SkillId::new("test", "dep", SemVersion::new(1, 0, 0));
        reg.register(Skill::new(dep_id.clone(), SkillTier::Basic, "dep"));
        let main_id = SkillId::new("test", "main", SemVersion::new(1, 0, 0));
        reg.register(Skill::new(main_id.clone(), SkillTier::Standard, "main").with_dependency(dep_id.clone()));
        let resolver = SkillDependencyResolver::new(reg);
        let result = resolver.resolve(&main_id).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], dep_id);
        assert_eq!(result[1], main_id);
    }

    #[test]
    fn test_dependency_circular() {
        let mut reg = SkillRegistry::new();
        let a_id = SkillId::new("test", "a", SemVersion::new(1, 0, 0));
        let b_id = SkillId::new("test", "b", SemVersion::new(1, 0, 0));
        reg.register(Skill::new(a_id.clone(), SkillTier::Basic, "a").with_dependency(b_id.clone()));
        reg.register(Skill::new(b_id.clone(), SkillTier::Basic, "b").with_dependency(a_id.clone()));
        let resolver = SkillDependencyResolver::new(reg);
        assert!(resolver.resolve(&a_id).is_err());
    }

    #[test]
    fn test_registry_sync_in_sync() {
        let sync = RegistrySync::new(5, 5);
        assert_eq!(sync.check_status(), SyncStatus::InSync);
    }

    #[test]
    fn test_registry_sync_behind() {
        let sync = RegistrySync::new(3, 5);
        assert_eq!(sync.check_status(), SyncStatus::Behind(2));
    }

    #[test]
    fn test_registry_sync_pull_push() {
        let mut sync = RegistrySync::new(1, 2);
        sync.set_local(vec![SkillId::new("ns", "a", SemVersion::new(1, 0, 0))]);
        sync.set_remote(vec![
            SkillId::new("ns", "a", SemVersion::new(1, 0, 0)),
            SkillId::new("ns", "b", SemVersion::new(1, 0, 0)),
        ]);
        assert_eq!(sync.skills_to_pull().len(), 1);
        assert_eq!(sync.skills_to_push().len(), 0);
    }

    #[test]
    fn test_registry_sync_conflicts() {
        let mut sync = RegistrySync::new(1, 2);
        sync.set_local(vec![SkillId::new("ns", "foo", SemVersion::new(1, 0, 0))]);
        sync.set_remote(vec![SkillId::new("ns", "foo", SemVersion::new(2, 0, 0))]);
        let conflicts = sync.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
    }
}
