//! # ternary-registry
//!
//! Ternary service registry for GPU fleet.
//! Nodes register capabilities, clients discover services.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Healthy = 1,
    Degraded = 0,
    Failed = -1,
}

#[derive(Debug, Clone)]
pub struct ServiceNode {
    pub id: String,
    pub capabilities: Vec<String>,
    pub health: Health,
    pub load: f64,
    pub version: String,
}

impl ServiceNode {
    pub fn new(id: &str, caps: Vec<&str>) -> Self {
        Self {
            id: id.into(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            health: Health::Healthy,
            load: 0.0,
            version: "v1".into(),
        }
    }
}

pub struct ServiceRegistry {
    nodes: HashMap<String, ServiceNode>,
    service_index: HashMap<String, Vec<String>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            service_index: HashMap::new(),
        }
    }

    pub fn register(&mut self, node: ServiceNode) {
        // Remove any existing entry with the same ID so that
        // re-registration does not leave stale or duplicate
        // entries in the service index.
        self.deregister(&node.id);
        for cap in &node.capabilities {
            self.service_index
                .entry(cap.clone())
                .or_default()
                .push(node.id.clone());
        }
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn deregister(&mut self, id: &str) {
        if let Some(node) = self.nodes.remove(id) {
            for cap in &node.capabilities {
                if let Some(v) = self.service_index.get_mut(cap) {
                    v.retain(|n| n != id);
                }
            }
            // Prune empty capability entries so service_count() stays accurate.
            self.service_index.retain(|_, v| !v.is_empty());
        }
    }

    pub fn discover(&self, capability: &str) -> Vec<&ServiceNode> {
        self.service_index
            .get(capability)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.nodes.get(id))
                    .filter(|n| n.health != Health::Failed)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn discover_healthy(&self, capability: &str) -> Vec<&ServiceNode> {
        self.discover(capability)
            .into_iter()
            .filter(|n| n.health == Health::Healthy)
            .collect()
    }

    pub fn discover_least_loaded(&self, capability: &str) -> Option<&ServiceNode> {
        self.discover_healthy(capability)
            .into_iter()
            .min_by(|a, b| a.load.total_cmp(&b.load))
    }

    pub fn update_health(&mut self, id: &str, health: Health) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.health = health;
        }
    }

    pub fn update_load(&mut self, id: &str, load: f64) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.load = load;
        }
    }

    pub fn crdt_merge(&mut self, other: &ServiceRegistry) {
        for (id, node) in &other.nodes {
            match self.nodes.get_mut(id) {
                Some(local) => {
                    if node.health == Health::Failed {
                        local.health = Health::Failed;
                    }
                }
                None => {
                    self.register(node.clone());
                }
            }
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn healthy_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|n| n.health == Health::Healthy)
            .count()
    }
    pub fn service_count(&self) -> usize {
        self.service_index.len()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_discover() {
        let mut reg = ServiceRegistry::new();
        reg.register(ServiceNode::new("gpu-0", vec!["matmul", "attention"]));
        reg.register(ServiceNode::new("gpu-1", vec!["matmul", "filter"]));
        let nodes = reg.discover("matmul");
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_single_capability() {
        let mut reg = ServiceRegistry::new();
        reg.register(ServiceNode::new("gpu-2", vec!["attention"]));
        let nodes = reg.discover("filter");
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_failed_excluded() {
        let mut reg = ServiceRegistry::new();
        reg.register(ServiceNode::new("gpu-0", vec!["matmul"]));
        reg.update_health("gpu-0", Health::Failed);
        let nodes = reg.discover("matmul");
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_least_loaded() {
        let mut reg = ServiceRegistry::new();
        let mut n1 = ServiceNode::new("gpu-0", vec!["matmul"]);
        n1.load = 0.8;
        let mut n2 = ServiceNode::new("gpu-1", vec!["matmul"]);
        n2.load = 0.3;
        reg.register(n1);
        reg.register(n2);
        let best = reg.discover_least_loaded("matmul").unwrap();
        assert_eq!(best.id, "gpu-1");
    }

    #[test]
    fn test_deregister() {
        let mut reg = ServiceRegistry::new();
        reg.register(ServiceNode::new("gpu-0", vec!["matmul"]));
        reg.deregister("gpu-0");
        assert_eq!(reg.node_count(), 0);
    }

    #[test]
    fn test_least_loaded_with_nan_load() {
        let mut reg = ServiceRegistry::new();
        let mut n1 = ServiceNode::new("gpu-0", vec!["matmul"]);
        n1.load = f64::NAN;
        let mut n2 = ServiceNode::new("gpu-1", vec!["matmul"]);
        n2.load = 0.5;
        reg.register(n1);
        reg.register(n2);
        // Must not panic; NaN should not crash discovery.
        let best = reg.discover_least_loaded("matmul").unwrap();
        assert_eq!(best.id, "gpu-1");
    }

    #[test]
    fn test_deregister_cleans_service_index() {
        let mut reg = ServiceRegistry::new();
        reg.register(ServiceNode::new("gpu-0", vec!["matmul"]));
        reg.deregister("gpu-0");
        // After deregistering the only node with "matmul", the capability
        // should no longer be counted by service_count().
        assert_eq!(
            reg.service_count(),
            0,
            "empty capability entries must be pruned from service_index"
        );
    }

    #[test]
    fn test_reregister_no_duplicate_index() {
        let mut reg = ServiceRegistry::new();
        reg.register(ServiceNode::new("gpu-0", vec!["matmul"]));
        // Re-register same node ID with same capability
        reg.register(ServiceNode::new("gpu-0", vec!["matmul"]));
        let nodes = reg.discover("matmul");
        assert_eq!(
            nodes.len(),
            1,
            "re-registration must not duplicate index entries"
        );
    }

    #[test]
    fn test_crdt_merge() {
        let mut r1 = ServiceRegistry::new();
        r1.register(ServiceNode::new("gpu-0", vec!["matmul"]));
        let mut r2 = ServiceRegistry::new();
        r2.register(ServiceNode::new("gpu-0", vec!["matmul"]));
        r2.update_health("gpu-0", Health::Failed);
        r1.crdt_merge(&r2);
        assert_eq!(r1.nodes["gpu-0"].health, Health::Failed);
    }

    #[test]
    fn test_counts() {
        let mut reg = ServiceRegistry::new();
        reg.register(ServiceNode::new("a", vec!["x", "y"]));
        reg.register(ServiceNode::new("b", vec!["x"]));
        assert_eq!(reg.node_count(), 2);
        assert_eq!(reg.service_count(), 2);
        assert_eq!(reg.healthy_count(), 2);
    }
}
