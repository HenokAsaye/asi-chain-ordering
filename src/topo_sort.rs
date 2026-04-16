use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

use crate::errors::DagError;
use crate::node::Node;

#[derive(Debug, Clone, Eq, PartialEq)]
struct ReadyKey {
    timestamp: u64,
    id: String,
}

impl Ord for ReadyKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp
            .cmp(&other.timestamp)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for ReadyKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn kahn_topological_sort(nodes: Vec<Node>) -> Result<Vec<Node>, DagError> {
    let mut node_by_id: HashMap<String, Node> = HashMap::with_capacity(nodes.len());
    for node in nodes {
        node_by_id.insert(node.id.clone(), node);
    }

    let mut indegree: HashMap<String, usize> =
        node_by_id.keys().map(|id| (id.clone(), 0_usize)).collect();
    let mut outgoing: HashMap<String, Vec<String>> = node_by_id
        .keys()
        .map(|id| (id.clone(), Vec::new()))
        .collect();

    // Build graph edges from dependency -> node.
    for (id, node) in &node_by_id {
        for dep in &node.dependencies {
            if node_by_id.contains_key(dep) {
                *indegree.get_mut(id).expect("indegree must exist") += 1;
                outgoing
                    .get_mut(dep)
                    .expect("outgoing list must exist")
                    .push(id.clone());
            }
        }
    }

    let mut ready = BTreeSet::new();
    for (id, degree) in &indegree {
        if *degree == 0 {
            let node = node_by_id.get(id).expect("node must exist");
            ready.insert(ReadyKey {
                timestamp: node.timestamp,
                id: id.clone(),
            });
        }
    }

    let mut ordered = Vec::with_capacity(node_by_id.len());

    while let Some(next_key) = ready.pop_first() {
        let next_id = next_key.id;
        let node = node_by_id
            .get(&next_id)
            .expect("ready node must be present")
            .clone();
        ordered.push(node);

        if let Some(children) = outgoing.get(&next_id) {
            for child_id in children {
                let entry = indegree
                    .get_mut(child_id)
                    .expect("child indegree must exist");
                *entry = entry.saturating_sub(1);

                if *entry == 0 {
                    let child = node_by_id.get(child_id).expect("child node must exist");
                    ready.insert(ReadyKey {
                        timestamp: child.timestamp,
                        id: child_id.clone(),
                    });
                }
            }
        }
    }

    if ordered.len() != node_by_id.len() {
        let mut remaining: Vec<String> = indegree
            .into_iter()
            .filter_map(|(id, degree)| if degree > 0 { Some(id) } else { None })
            .collect();
        remaining.sort_unstable();
        return Err(DagError::CycleDetected {
            remaining_nodes: remaining,
        });
    }

    Ok(ordered)
}
