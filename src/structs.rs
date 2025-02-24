use crate::CommitMetadata;

/// Structure to store git commit stats for each author
pub struct CommitStats {
    // Author name
    pub author: String,
    // Author commit count
    pub count: usize,
    // Line insertions
    pub insertions: usize,
    // Line deletions
    pub deletions: usize,
}

impl CommitMetadata for CommitStats {
    fn author(&self) -> String {
        self.author.clone()
    }
}

/// Structure to store git working time estimate for each author
pub struct CommitTime {
    // Author name
    pub author: String,
    // Time spent on repository (in minutes)
    pub time: usize,
}

impl CommitMetadata for CommitTime {
    fn author(&self) -> String {
        self.author.clone()
    }
}
