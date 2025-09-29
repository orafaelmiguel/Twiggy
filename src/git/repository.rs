use git2::{Repository, RepositoryState};
use std::path::{Path, PathBuf};
use crate::error::{Result, TwiggyError};
use crate::git::types::{CommitInfo, BranchInfo};

pub struct GitRepository {
    inner: Repository,
    path: PathBuf,
    repo_type: RepositoryType,
}

#[derive(Debug, Clone)]
pub enum RepositoryType {
    Normal,
    Bare,
    Worktree,
}

#[derive(Debug, Clone)]
pub enum RepositoryHealth {
    Healthy,
    InOperation(String),
    Corrupted,
    Unknown,
}

impl GitRepository {
    pub fn detect(path: impl AsRef<Path>) -> Result<Option<Self>> {
        let path = path.as_ref();
        tracing::debug!("Detecting Git repository at: {}", path.display());
        
        match Repository::discover(path) {
            Ok(repo) => {
                let repo_path = repo.path().to_path_buf();
                let repo_type = if repo.is_bare() {
                    RepositoryType::Bare
                } else if repo.is_worktree() {
                    RepositoryType::Worktree
                } else {
                    RepositoryType::Normal
                };
                
                tracing::info!("Git repository detected: {:?} at {}", repo_type, repo_path.display());
                
                Ok(Some(Self {
                    inner: repo,
                    path: repo_path,
                    repo_type,
                }))
            }
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                tracing::debug!("No Git repository found at: {}", path.display());
                Ok(None)
            }
            Err(e) => {
                tracing::error!("Git repository detection failed: {}", e);
                Err(TwiggyError::Git {
                    message: format!("Repository detection failed: {}", e),
                    source: e,
                })
            }
        }
    }
    
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        tracing::debug!("Opening Git repository at: {}", path.display());
        
        match Repository::open(path) {
            Ok(repo) => {
                let repo_path = repo.path().to_path_buf();
                let repo_type = if repo.is_bare() {
                    RepositoryType::Bare
                } else if repo.is_worktree() {
                    RepositoryType::Worktree
                } else {
                    RepositoryType::Normal
                };
                
                tracing::info!("Git repository opened: {:?} at {}", repo_type, repo_path.display());
                
                Ok(Self {
                    inner: repo,
                    path: repo_path,
                    repo_type,
                })
            }
            Err(e) => {
                tracing::error!("Failed to open Git repository: {}", e);
                Err(TwiggyError::Git {
                    message: format!("Failed to open repository: {}", e),
                    source: e,
                })
            }
        }
    }
    
    pub fn validate(&self) -> Result<RepositoryHealth> {
        let state = self.inner.state();
        
        let health = match state {
            RepositoryState::Clean => RepositoryHealth::Healthy,
            RepositoryState::Merge => RepositoryHealth::InOperation("merge".to_string()),
            RepositoryState::Revert => RepositoryHealth::InOperation("revert".to_string()),
            RepositoryState::CherryPick => RepositoryHealth::InOperation("cherry-pick".to_string()),
            RepositoryState::Bisect => RepositoryHealth::InOperation("bisect".to_string()),
            RepositoryState::Rebase => RepositoryHealth::InOperation("rebase".to_string()),
            RepositoryState::RebaseInteractive => RepositoryHealth::InOperation("interactive rebase".to_string()),
            RepositoryState::RebaseMerge => RepositoryHealth::InOperation("rebase merge".to_string()),
            RepositoryState::ApplyMailbox => RepositoryHealth::InOperation("apply mailbox".to_string()),
            RepositoryState::ApplyMailboxOrRebase => RepositoryHealth::InOperation("apply mailbox or rebase".to_string()),
            _ => RepositoryHealth::Unknown,
        };
        
        tracing::debug!("Repository health check: {:?}", health);
        Ok(health)
    }
    
    pub fn is_accessible(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }
    
    pub fn check_permissions(&self) -> Result<bool> {
        use std::fs;
        
        match fs::metadata(&self.path) {
            Ok(metadata) => {
                let readonly = metadata.permissions().readonly();
                tracing::debug!("Repository permissions - readonly: {}", readonly);
                Ok(!readonly)
            }
            Err(e) => {
                tracing::error!("Failed to check repository permissions: {}", e);
                Err(TwiggyError::Io {
                    operation: "check permissions".to_string(),
                    source: e,
                })
            }
        }
    }
    
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    pub fn repo_type(&self) -> &RepositoryType {
        &self.repo_type
    }
    
    pub fn workdir(&self) -> Option<&Path> {
        self.inner.workdir()
    }
    
    pub fn is_empty(&self) -> Result<bool> {
        match self.inner.is_empty() {
            Ok(empty) => {
                tracing::debug!("Repository empty check: {}", empty);
                Ok(empty)
            }
            Err(e) => {
                tracing::error!("Failed to check if repository is empty: {}", e);
                Err(TwiggyError::Git {
                    message: "Failed to check repository status".to_string(),
                    source: e,
                })
            }
        }
    }
    
    pub fn get_commits(&self) -> Result<Vec<CommitInfo>> {
        Ok(vec![])
    }
    
    pub fn get_branches(&self) -> Result<Vec<BranchInfo>> {
        Ok(vec![])
    }
}

pub fn is_git_repository(path: impl AsRef<Path>) -> Result<bool> {
    match GitRepository::detect(path)? {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

pub fn discover_repository(path: impl AsRef<Path>) -> Result<Option<PathBuf>> {
    let path = path.as_ref();
    tracing::debug!("Discovering Git repository from: {}", path.display());
    
    match Repository::discover(path) {
        Ok(repo) => {
            let repo_path = if let Some(workdir) = repo.workdir() {
                workdir.to_path_buf()
            } else {
                repo.path().to_path_buf()
            };
            tracing::info!("Repository discovered at: {}", repo_path.display());
            Ok(Some(repo_path))
        }
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            tracing::debug!("No Git repository discovered from: {}", path.display());
            Ok(None)
        }
        Err(e) => {
            tracing::error!("Repository discovery failed: {}", e);
            Err(TwiggyError::Git {
                message: format!("Repository discovery failed: {}", e),
                source: e,
            })
        }
    }
}

pub fn validate_repository_path(path: impl AsRef<Path>) -> Result<bool> {
    let path = path.as_ref();
    
    if !path.exists() {
        tracing::warn!("Repository path does not exist: {}", path.display());
        return Ok(false);
    }
    
    if !path.is_dir() {
        tracing::warn!("Repository path is not a directory: {}", path.display());
        return Ok(false);
    }
    
    match std::fs::metadata(path) {
        Ok(_) => {
            tracing::debug!("Repository path validation successful: {}", path.display());
            Ok(true)
        }
        Err(e) => {
            tracing::error!("Failed to validate repository path: {}", e);
            Err(TwiggyError::Io {
                operation: "validate path".to_string(),
                source: e,
            })
        }
    }
}