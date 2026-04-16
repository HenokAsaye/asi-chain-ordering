use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagError {
    EmptyNodeId,
    SelfDependency { node_id: String },
    CycleDetected { remaining_nodes: Vec<String> },
}

impl Display for DagError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyNodeId => write!(f, "node id cannot be empty"),
            Self::SelfDependency { node_id } => {
                write!(f, "node '{node_id}' cannot depend on itself")
            }
            Self::CycleDetected { remaining_nodes } => {
                write!(f, "cycle detected among nodes: {:?}", remaining_nodes)
            }
        }
    }
}

impl Error for DagError {}
