use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub dependencies: Vec<String>,
    pub timestamp: u64,
    pub committed: bool,
}

impl Node {
    pub fn new(
        id: impl Into<String>,
        dependencies: Vec<&str>,
        timestamp: u64,
        committed: bool,
    ) -> Self {
        Self::from_dependencies(id, dependencies, timestamp, committed)
    }

    pub fn from_dependencies<I, S>(
        id: impl Into<String>,
        dependencies: I,
        timestamp: u64,
        committed: bool,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            id: id.into(),
            dependencies: dependencies.into_iter().map(Into::into).collect(),
            timestamp,
            committed,
        }
    }

    pub fn dedup_dependencies(&mut self) {
        let mut seen = HashSet::new();
        self.dependencies.retain(|dep| seen.insert(dep.clone()));
    }
}
