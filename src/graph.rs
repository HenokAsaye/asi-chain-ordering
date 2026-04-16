use std::collections::HashSet;

use crate::errors::DagError;
use crate::merge::merge_dags;
use crate::node::Node;
use crate::topo_sort::kahn_topological_sort;

pub fn filter_committed_nodes(nodes: Vec<Node>) -> Vec<Node> {
    let committed_ids: HashSet<String> = nodes
        .iter()
        .filter(|node| node.committed)
        .map(|node| node.id.clone())
        .collect();

    nodes
        .into_iter()
        .filter_map(|mut node| {
            if !node.committed {
                return None;
            }

            // Keep only dependencies that still exist after filtering.
            node.dependencies.retain(|dep| committed_ids.contains(dep));
            Some(node)
        })
        .collect()
}

fn validate_nodes(nodes: &[Node]) -> Result<(), DagError> {
    for node in nodes {
        if node.id.trim().is_empty() {
            return Err(DagError::EmptyNodeId);
        }
        if node.dependencies.iter().any(|dep| dep == &node.id) {
            return Err(DagError::SelfDependency {
                node_id: node.id.clone(),
            });
        }
    }
    Ok(())
}

pub fn order_committed_dag(dag_a: Vec<Node>, dag_b: Vec<Node>) -> Result<Vec<Node>, DagError> {
    let merged = merge_dags(dag_a, dag_b);
    let committed = filter_committed_nodes(merged);
    validate_nodes(&committed)?;
    kahn_topological_sort(committed)
}

/// Public API for T-ordering linearization: combine two DAGs and produce one
/// deterministic committed topological order.
pub fn t_ordering_linearize(dag_a: Vec<Node>, dag_b: Vec<Node>) -> Result<Vec<Node>, DagError> {
    order_committed_dag(dag_a, dag_b)
}

#[cfg(test)]
mod tests {
    use crate::errors::DagError;
    use crate::node::Node;

    use super::{filter_committed_nodes, order_committed_dag, t_ordering_linearize};

    #[test]
    fn filtering_keeps_only_committed_and_trims_uncommitted_dependencies() {
        let nodes = vec![
            Node::new("A", vec![], 1, true),
            Node::new("B", vec!["A"], 2, false),
            Node::new("C", vec!["A", "B"], 3, true),
        ];

        let filtered = filter_committed_nodes(nodes);
        assert_eq!(filtered.len(), 2);

        let c = filtered
            .iter()
            .find(|n| n.id == "C")
            .expect("C should exist");
        assert_eq!(c.dependencies, vec!["A".to_string()]);
    }

    #[test]
    fn topological_order_respects_dependencies_and_tiebreaking() {
        let dag_a = vec![
            Node::new("A", vec![], 10, true),
            Node::new("B", vec!["A"], 20, true),
        ];
        let dag_b = vec![
            Node::new("C", vec!["A"], 20, true),
            Node::new("D", vec!["B", "C"], 30, true),
        ];

        let ordered = order_committed_dag(dag_a, dag_b).expect("ordering should succeed");
        let ids: Vec<String> = ordered.into_iter().map(|n| n.id).collect();
        assert_eq!(ids, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn cycle_detection_returns_error() {
        let dag_a = vec![Node::new("A", vec!["B"], 1, true)];
        let dag_b = vec![Node::new("B", vec!["A"], 2, true)];

        let err = order_committed_dag(dag_a, dag_b).expect_err("cycle should fail");
        assert!(matches!(err, DagError::CycleDetected { .. }));
    }

    #[test]
    fn tie_break_with_equal_timestamp_uses_lexicographic_id() {
        let dag_a = vec![Node::new("B", vec![], 1, true)];
        let dag_b = vec![Node::new("A", vec![], 1, true)];

        let ordered = t_ordering_linearize(dag_a, dag_b).expect("ordering should succeed");
        let ids: Vec<String> = ordered.into_iter().map(|n| n.id).collect();
        assert_eq!(ids, vec!["A", "B"]);
    }

    #[test]
    fn empty_node_id_is_rejected() {
        let dag_a = vec![Node::new("", vec![], 1, true)];
        let dag_b = vec![];
        let err = order_committed_dag(dag_a, dag_b).expect_err("must reject empty id");
        assert!(matches!(err, DagError::EmptyNodeId));
    }

    #[test]
    fn self_dependency_is_rejected() {
        let dag_a = vec![Node::new("A", vec!["A"], 1, true)];
        let dag_b = vec![];
        let err = order_committed_dag(dag_a, dag_b).expect_err("must reject self dependency");
        assert!(matches!(err, DagError::SelfDependency { .. }));
    }
}
