use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use git2::Oid;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommitId(pub Oid);

impl CommitId {
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
    
    pub fn short(&self) -> String {
        self.0.to_string()[..7].to_string()
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: CommitId,
    pub author: Signature,
    pub committer: Signature,
    pub message: String,
    pub summary: String,
    pub parents: Vec<CommitId>,
    pub tree_id: String,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub name: String,
    pub email: String,
    pub time: DateTime<Utc>,
}

impl From<&git2::Signature<'_>> for Signature {
    fn from(sig: &git2::Signature) -> Self {
        let time = DateTime::from_timestamp(sig.when().seconds(), 0)
            .unwrap_or_else(|| Utc::now());
        
        Self {
            name: sig.name().unwrap_or("Unknown").to_string(),
            email: sig.email().unwrap_or("").to_string(),
            time,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: AuthorInfo,
    pub timestamp: DateTime<Utc>,
    pub parents: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub name: String,
    pub email: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub commit_id: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffInfo {
    pub old_file: Option<String>,
    pub new_file: Option<String>,
    pub hunks: Vec<DiffHunk>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
}