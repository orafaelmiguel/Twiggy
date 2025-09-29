use git2::Repository;
use std::path::{Path, PathBuf};
use crate::error::{Result, TwiggyError};

#[derive(Debug, Clone, PartialEq)]
pub enum RepositoryType {
    Normal,
    Bare,
    Worktree,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RepositoryHealth {
    Healthy,
    InOperation(String),
    Corrupted,
    Unknown,
}

pub struct GitRepository {
    inner: Repository,
    path: PathBuf,
    repo_type: RepositoryType,
    current_branch: Option<String>,
    is_detached: bool,
}

impl GitRepository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        tracing::info!("Opening Git repository at: {}", path.display());
        
        let repo = Repository::open(path)
            .map_err(|e| TwiggyError::Git {
                message: format!("Failed to open repository: {}", e),
                source: e,
            })?;
        
        let repo_type = if repo.is_bare() {
            RepositoryType::Bare
        } else if repo.is_worktree() {
            RepositoryType::Worktree
        } else {
            RepositoryType::Normal
        };
        
        let (current_branch, is_detached) = Self::get_current_branch_info(&repo)?;
        
        tracing::info!("Repository opened successfully: {:?}, branch: {:?}, detached: {}", 
            repo_type, current_branch, is_detached);
        
        Ok(Self {
            inner: repo,
            path: path.to_path_buf(),
            repo_type,
            current_branch,
            is_detached,
        })
    }

    pub fn detect(path: impl AsRef<Path>) -> Result<Option<Self>> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Ok(None);
        }
        
        match Repository::open(path) {
            Ok(_) => {
                match Self::open(path) {
                    Ok(repo) => Ok(Some(repo)),
                    Err(_) => Ok(None),
                }
            }
            Err(_) => Ok(None),
        }
    }
    
    fn get_current_branch_info(repo: &Repository) -> Result<(Option<String>, bool)> {
        match repo.head() {
            Ok(head) => {
                let is_detached = !head.is_branch();
                
                if is_detached {
                    if let Some(oid) = head.target() {
                        let short_id = format!("{:.7}", oid);
                        Ok((Some(format!("HEAD detached at {}", short_id)), true))
                    } else {
                        Ok((Some("HEAD (detached)".to_string()), true))
                    }
                } else if let Some(branch_name) = head.shorthand() {
                    Ok((Some(branch_name.to_string()), false))
                } else {
                    Ok((Some("HEAD".to_string()), false))
                }
            }
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                Ok((Some("main".to_string()), false))
            }
            Err(e) => {
                tracing::warn!("Failed to get current branch: {}", e);
                Ok((None, false))
            }
        }
    }
    
    pub fn refresh_branch_info(&mut self) -> Result<()> {
        let (current_branch, is_detached) = Self::get_current_branch_info(&self.inner)?;
        self.current_branch = current_branch;
        self.is_detached = is_detached;
        Ok(())
    }
    
    pub fn current_branch(&self) -> Option<&str> {
        self.current_branch.as_deref()
    }
    
    pub fn is_detached(&self) -> bool {
        self.is_detached
    }
    
    pub fn repository_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }
    
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    pub fn repo_type(&self) -> &RepositoryType {
        &self.repo_type
    }
    
    pub fn commit_count(&self) -> Result<usize> {
        let mut revwalk = self.inner.revwalk()
            .map_err(|e| TwiggyError::Git {
                message: "Failed to create revwalk".to_string(),
                source: e,
            })?;
        
        match revwalk.push_head() {
            Ok(_) => Ok(revwalk.count()),
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => Ok(0),
            Err(e) => Err(TwiggyError::Git {
                message: "Failed to push HEAD".to_string(),
                source: e,
            })
        }
    }
    
    pub fn validate(&self) -> Result<RepositoryHealth> {
        if self.inner.is_empty()? {
            return Ok(RepositoryHealth::Healthy);
        }
        
        let state = self.inner.state();
        match state {
            git2::RepositoryState::Clean => Ok(RepositoryHealth::Healthy),
            git2::RepositoryState::Merge => Ok(RepositoryHealth::InOperation("merge".to_string())),
            git2::RepositoryState::Revert => Ok(RepositoryHealth::InOperation("revert".to_string())),
            git2::RepositoryState::CherryPick => Ok(RepositoryHealth::InOperation("cherry-pick".to_string())),
            git2::RepositoryState::Bisect => Ok(RepositoryHealth::InOperation("bisect".to_string())),
            git2::RepositoryState::Rebase => Ok(RepositoryHealth::InOperation("rebase".to_string())),
            git2::RepositoryState::RebaseInteractive => Ok(RepositoryHealth::InOperation("interactive rebase".to_string())),
            git2::RepositoryState::RebaseMerge => Ok(RepositoryHealth::InOperation("rebase merge".to_string())),
            git2::RepositoryState::ApplyMailbox => Ok(RepositoryHealth::InOperation("apply mailbox".to_string())),
            git2::RepositoryState::ApplyMailboxOrRebase => Ok(RepositoryHealth::InOperation("apply mailbox or rebase".to_string())),
            _ => Ok(RepositoryHealth::Unknown),
        }
    }
    
    pub fn is_accessible(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }
    
    pub fn check_permissions(&self) -> Result<bool> {
        use std::fs;
        
        if !self.path.exists() {
            return Ok(false);
        }
        
        match fs::metadata(&self.path) {
            Ok(metadata) => Ok(!metadata.permissions().readonly()),
            Err(_) => Ok(false),
        }
    }
    
    pub fn is_empty(&self) -> Result<bool> {
        self.inner.is_empty().map_err(|e| TwiggyError::Git {
            message: "Failed to check if repository is empty".to_string(),
            source: e,
        })
    }
    
    pub fn workdir(&self) -> Option<&Path> {
        self.inner.workdir()
    }
}

pub fn is_git_repository(path: impl AsRef<Path>) -> bool {
    Repository::open(path).is_ok()
}

pub fn discover_repository(path: impl AsRef<Path>) -> Result<Option<PathBuf>> {
    match Repository::discover(path) {
        Ok(repo) => Ok(Some(repo.path().to_path_buf())),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(TwiggyError::Git {
            message: format!("Failed to discover repository: {}", e),
            source: e,
        }),
    }
}

pub fn validate_repository_path(path: impl AsRef<Path>) -> Result<bool> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Ok(false);
    }
    
    if !path.is_dir() {
        return Ok(false);
    }
    
    use std::fs;
    match fs::metadata(path) {
        Ok(_) => Ok(true),
        Err(e) => Err(TwiggyError::Io {
            operation: format!("Failed to access path: {}", path.display()),
            source: e,
        }),
    }
}