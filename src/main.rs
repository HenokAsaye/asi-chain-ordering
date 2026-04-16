mod errors;
mod graph;
mod merge;
mod node;
mod topo_sort;

use graph::t_ordering_linearize;
use node::Node;

fn main() {
    // Example DAGs from two sources.
    let dag_a = vec![
        Node::new("A", vec![], 100, true),
        Node::new("B", vec!["A"], 101, true),
    ];
    let dag_b = vec![
        Node::new("C", vec!["A"], 102, true),
        Node::new("D", vec!["B", "C"], 103, true),
    ];

    match t_ordering_linearize(dag_a, dag_b) {
        Ok(ordered) => {
            println!("Final committed topological order:");
            for node in ordered {
                println!(
                    "id={} ts={} deps={:?} committed={}",
                    node.id, node.timestamp, node.dependencies, node.committed
                );
            }
        }
        Err(err) => {
            eprintln!("Failed to order DAG: {err}");
            std::process::exit(1);
        }
    }
}
