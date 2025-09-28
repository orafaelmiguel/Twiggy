use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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