use std::collections::{HashMap, HashSet};

use crate::node::Node;

pub fn merge_dags(dag_a: Vec<Node>, dag_b: Vec<Node>) -> Vec<Node> {
    let mut merged: HashMap<String, Node> = HashMap::new();

    for node in dag_a.into_iter().chain(dag_b) {
        match merged.get_mut(&node.id) {
            Some(existing) => {
                // Combine dependency sets without duplicates.
                let mut dep_set: HashSet<String> = existing.dependencies.drain(..).collect();
                dep_set.extend(node.dependencies);
                existing.dependencies = dep_set.into_iter().collect();
                existing.dependencies.sort_unstable();

                // Use earliest timestamp as stable tiebreaking baseline.
                existing.timestamp = existing.timestamp.min(node.timestamp);

                // A node is considered committed if any source reports it committed.
                existing.committed |= node.committed;
            }
            None => {
                let mut n = node;
                n.dedup_dependencies();
                n.dependencies.sort_unstable();
                merged.insert(n.id.clone(), n);
            }
        }
    }

    merged.into_values().collect()
}

#[cfg(test)]
mod tests {
    use crate::node::Node;

    use super::merge_dags;

    #[test]
    fn merge_combines_dependencies_and_removes_duplicates() {
        let dag_a = vec![Node::new("A", vec!["B"], 10, true)];
        let dag_b = vec![Node::new("A", vec!["C", "B"], 20, false)];

        let merged = merge_dags(dag_a, dag_b);
        assert_eq!(merged.len(), 1);
        let a = &merged[0];
        assert_eq!(a.id, "A");
        assert_eq!(a.dependencies, vec!["B".to_string(), "C".to_string()]);
        assert_eq!(a.timestamp, 10);
        assert!(a.committed);
    }

    #[test]
    fn merge_dedups_dependencies_within_single_input_node() {
        let dag_a = vec![Node::new("X", vec!["A", "A", "B"], 7, true)];
        let dag_b = vec![];
        let merged = merge_dags(dag_a, dag_b);

        assert_eq!(merged.len(), 1);
        assert_eq!(
            merged[0].dependencies,
            vec!["A".to_string(), "B".to_string()]
        );
    }
}
