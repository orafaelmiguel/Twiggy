use git2::{Repository, Branch, BranchType, Direction};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::error::{Result, TwiggyError};
use crate::git::types::{Commit, CommitId, Signature};

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

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub state: BranchState,
}

#[derive(Debug, Clone)]
pub enum BranchState {
    Normal,
    DetachedHead,
    Unborn,
}

pub struct GitRepository {
    inner: Repository,
    path: PathBuf,
    repo_type: RepositoryType,
    current_branch: Option<String>,
    is_detached: bool,
    commits: Vec<Commit>,
    commit_cache: HashMap<CommitId, Commit>,
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
            commits: Vec::new(),
            commit_cache: HashMap::new(),
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
    
    pub fn get_branch_info(&self) -> Result<BranchInfo> {
        match self.inner.head() {
            Ok(head) => {
                if head.is_branch() {
                    let branch_name = head.shorthand()
                        .ok_or_else(|| TwiggyError::Git {
                            message: "Invalid branch name".to_string(),
                            source: git2::Error::from_str("Invalid branch name"),
                        })?
                        .to_string();
                    
                    let branch = self.inner.find_branch(&branch_name, BranchType::Local)
                        .map_err(|e| TwiggyError::Git {
                            message: "Failed to find branch".to_string(),
                            source: e,
                        })?;
                    
                    let upstream = branch.upstream()
                        .ok()
                        .and_then(|u| u.name().ok().flatten().map(|s| s.to_string()));
                    
                    let (ahead, behind) = self.calculate_ahead_behind(&branch)?;
                    
                    Ok(BranchInfo {
                        name: branch_name,
                        upstream,
                        ahead,
                        behind,
                        state: BranchState::Normal,
                    })
                } else {
                    let name = if let Some(oid) = head.target() {
                        format!("HEAD detached at {:.7}", oid)
                    } else {
                        "HEAD (detached)".to_string()
                    };
                    
                    Ok(BranchInfo {
                        name,
                        upstream: None,
                        ahead: 0,
                        behind: 0,
                        state: BranchState::DetachedHead,
                    })
                }
            }
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                Ok(BranchInfo {
                    name: "main".to_string(),
                    upstream: None,
                    ahead: 0,
                    behind: 0,
                    state: BranchState::Unborn,
                })
            }
            Err(e) => Err(TwiggyError::Git {
                message: "Failed to get HEAD".to_string(),
                source: e,
            })
        }
    }
    
    fn calculate_ahead_behind(&self, branch: &Branch) -> Result<(usize, usize)> {
        let local_oid = branch.get().target()
            .ok_or_else(|| TwiggyError::Git {
                message: "Branch has no target".to_string(),
                source: git2::Error::from_str("No target"),
            })?;
        
        if let Ok(upstream) = branch.upstream() {
            if let Some(upstream_oid) = upstream.get().target() {
                match self.inner.graph_ahead_behind(local_oid, upstream_oid) {
                    Ok((ahead, behind)) => return Ok((ahead, behind)),
                    Err(_) => return Ok((0, 0)),
                }
            }
        }
        
        Ok((0, 0))
    }
    
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_branch_info()?;
        Ok(())
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
    
    pub fn total_commit_count(&self) -> Result<usize> {
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

    pub fn load_commits(&mut self, limit: Option<usize>) -> Result<()> {
        tracing::info!("Loading commits from repository");
        let start = std::time::Instant::now();
        
        if self.inner.is_empty().unwrap_or(true) {
            tracing::warn!("Repository is empty, no commits to load");
            return Ok(());
        }
        
        let mut revwalk = self.inner.revwalk()
            .map_err(|e| TwiggyError::Git {
                message: "Failed to create revwalk".to_string(),
                source: e,
            })?;
        
        revwalk.push_head()
            .map_err(|e| TwiggyError::Git {
                message: "Failed to push HEAD".to_string(),
                source: e,
            })?;
        
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
            .map_err(|e| TwiggyError::Git {
                message: "Failed to set sorting".to_string(),
                source: e,
            })?;
        
        let max_commits = limit.unwrap_or(1000);
        let mut commits = Vec::new();
        let mut loaded_count = 0;
        
        for (index, oid) in revwalk.enumerate() {
            if index >= max_commits {
                tracing::debug!("Reached commit limit of {}", max_commits);
                break;
            }
            
            let oid = oid.map_err(|e| TwiggyError::Git {
                message: "Failed to get commit OID".to_string(),
                source: e,
            })?;
            
            match self.parse_commit(oid) {
                Ok(commit) => {
                    self.commit_cache.insert(commit.id, commit.clone());
                    commits.push(commit);
                    loaded_count += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to parse commit {}: {}", oid, e);
                    continue;
                }
            }
            
            if loaded_count % 100 == 0 {
                tracing::debug!("Loaded {} commits so far", loaded_count);
            }
        }
        
        let elapsed = start.elapsed();
        tracing::info!("Loaded {} commits in {:?}", commits.len(), elapsed);
        
        if commits.is_empty() {
            tracing::warn!("No commits were loaded from repository");
        }
        
        self.commits = commits;
        Ok(())
    }
    
    fn parse_commit(&self, oid: git2::Oid) -> Result<Commit> {
        let commit = self.inner.find_commit(oid)
            .map_err(|e| TwiggyError::Git {
                message: format!("Failed to find commit {}", oid),
                source: e,
            })?;
        
        let message = commit.message().unwrap_or("").to_string();
        let summary = commit.summary().unwrap_or("").to_string();
        
        if message.is_empty() && summary.is_empty() {
            tracing::debug!("Commit {} has empty message", oid);
        }
        
        let parents: Vec<CommitId> = commit
            .parent_ids()
            .map(|id| CommitId(id))
            .collect();
        
        let tree_id = commit.tree_id().to_string();
        
        let author = match commit.author().name() {
            Some(_) => Signature::from(&commit.author()),
            None => {
                tracing::debug!("Commit {} has invalid author signature", oid);
                Signature {
                    name: "Unknown".to_string(),
                    email: "".to_string(),
                    time: chrono::Utc::now(),
                }
            }
        };
        
        let committer = match commit.committer().name() {
            Some(_) => Signature::from(&commit.committer()),
            None => {
                tracing::debug!("Commit {} has invalid committer signature", oid);
                Signature {
                    name: "Unknown".to_string(),
                    email: "".to_string(),
                    time: chrono::Utc::now(),
                }
            }
        };
        
        Ok(Commit {
            id: CommitId(oid),
            author,
            committer,
            message,
            summary,
            parents,
            tree_id,
        })
    }

    pub fn get_commits(&self) -> &[Commit] {
        &self.commits
    }

    pub fn get_commit_by_id(&self, id: &CommitId) -> Option<&Commit> {
        self.commit_cache.get(id)
    }

    pub fn commit_count(&self) -> usize {
        self.commits.len()
    }

    pub fn load_commits_lazy(&mut self, start: usize, count: usize) -> Result<Vec<Commit>> {
        tracing::debug!("Loading commits lazily: start={}, count={}", start, count);
        
        if start < self.commits.len() {
            let end = (start + count).min(self.commits.len());
            return Ok(self.commits[start..end].to_vec());
        }
        
        let mut revwalk = self.inner.revwalk()
            .map_err(|e| TwiggyError::Git {
                message: "Failed to create revwalk for lazy loading".to_string(),
                source: e,
            })?;
        
        revwalk.push_head()
            .map_err(|e| TwiggyError::Git {
                message: "Failed to push HEAD for lazy loading".to_string(),
                source: e,
            })?;
        
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
            .map_err(|e| TwiggyError::Git {
                message: "Failed to set sorting for lazy loading".to_string(),
                source: e,
            })?;
        
        let mut commits = Vec::new();
        for (index, oid) in revwalk.enumerate() {
            if index < start {
                continue;
            }
            if index >= start + count {
                break;
            }
            
            let oid = oid.map_err(|e| TwiggyError::Git {
                message: "Failed to get commit OID in lazy loading".to_string(),
                source: e,
            })?;
            
            if let Some(cached_commit) = self.commit_cache.get(&CommitId(oid)) {
                commits.push(cached_commit.clone());
            } else {
                let commit = self.parse_commit(oid)?;
                self.commit_cache.insert(commit.id, commit.clone());
                commits.push(commit);
            }
        }
        
        Ok(commits)
    }

    pub fn refresh_commits(&mut self, limit: Option<usize>) -> Result<()> {
        tracing::info!("Refreshing commit data");
        self.commits.clear();
        self.load_commits(limit)
    }

    pub fn clear_commit_cache(&mut self) {
        tracing::debug!("Clearing commit cache");
        self.commit_cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.commit_cache.len()
    }

    pub fn find_commit_by_hash(&self, hash: &str) -> Result<Option<Commit>> {
        tracing::debug!("Searching for commit by hash: {}", hash);
        
        let oid = git2::Oid::from_str(hash)
            .map_err(|e| TwiggyError::Git {
                message: format!("Invalid commit hash: {}", hash),
                source: e,
            })?;
        
        let commit_id = CommitId(oid);
        
        if let Some(cached_commit) = self.commit_cache.get(&commit_id) {
            return Ok(Some(cached_commit.clone()));
        }
        
        match self.inner.find_commit(oid) {
            Ok(_) => {
                let commit = self.parse_commit(oid)?;
                Ok(Some(commit))
            }
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(TwiggyError::Git {
                message: format!("Failed to find commit {}", hash),
                source: e,
            }),
        }
    }

    pub fn load_commits_for_branch(&mut self, branch_name: &str, limit: Option<usize>) -> Result<Vec<Commit>> {
        tracing::info!("Loading commits for branch: {}", branch_name);
        
        let mut revwalk = self.inner.revwalk()
            .map_err(|e| TwiggyError::Git {
                message: "Failed to create revwalk for branch".to_string(),
                source: e,
            })?;
        
        let branch_ref = format!("refs/heads/{}", branch_name);
        let oid = self.inner.refname_to_id(&branch_ref)
            .map_err(|e| TwiggyError::Git {
                message: format!("Failed to find branch: {}", branch_name),
                source: e,
            })?;
        
        revwalk.push(oid)
            .map_err(|e| TwiggyError::Git {
                message: format!("Failed to push branch {} to revwalk", branch_name),
                source: e,
            })?;
        
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
            .map_err(|e| TwiggyError::Git {
                message: "Failed to set sorting for branch commits".to_string(),
                source: e,
            })?;
        
        let max_commits = limit.unwrap_or(1000);
        let mut commits = Vec::new();
        
        for (index, oid) in revwalk.enumerate() {
            if index >= max_commits {
                break;
            }
            
            let oid = oid.map_err(|e| TwiggyError::Git {
                message: "Failed to get commit OID for branch".to_string(),
                source: e,
            })?;
            
            if let Some(cached_commit) = self.commit_cache.get(&CommitId(oid)) {
                commits.push(cached_commit.clone());
            } else {
                let commit = self.parse_commit(oid)?;
                self.commit_cache.insert(commit.id, commit.clone());
                commits.push(commit);
            }
        }
        
        Ok(commits)
    }

    pub fn load_commits_range(&mut self, from: &str, to: &str, limit: Option<usize>) -> Result<Vec<Commit>> {
        tracing::info!("Loading commits in range: {}..{}", from, to);
        
        let from_oid = git2::Oid::from_str(from)
            .map_err(|e| TwiggyError::Git {
                message: format!("Invalid from commit hash: {}", from),
                source: e,
            })?;
        
        let to_oid = git2::Oid::from_str(to)
            .map_err(|e| TwiggyError::Git {
                message: format!("Invalid to commit hash: {}", to),
                source: e,
            })?;
        
        let mut revwalk = self.inner.revwalk()
            .map_err(|e| TwiggyError::Git {
                message: "Failed to create revwalk for range".to_string(),
                source: e,
            })?;
        
        revwalk.push(to_oid)
            .map_err(|e| TwiggyError::Git {
                message: format!("Failed to push to commit {} to revwalk", to),
                source: e,
            })?;
        
        revwalk.hide(from_oid)
            .map_err(|e| TwiggyError::Git {
                message: format!("Failed to hide from commit {} in revwalk", from),
                source: e,
            })?;
        
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
            .map_err(|e| TwiggyError::Git {
                message: "Failed to set sorting for range commits".to_string(),
                source: e,
            })?;
        
        let max_commits = limit.unwrap_or(1000);
        let mut commits = Vec::new();
        
        for (index, oid) in revwalk.enumerate() {
            if index >= max_commits {
                break;
            }
            
            let oid = oid.map_err(|e| TwiggyError::Git {
                message: "Failed to get commit OID for range".to_string(),
                source: e,
            })?;
            
            if let Some(cached_commit) = self.commit_cache.get(&CommitId(oid)) {
                commits.push(cached_commit.clone());
            } else {
                let commit = self.parse_commit(oid)?;
                self.commit_cache.insert(commit.id, commit.clone());
                commits.push(commit);
            }
        }
        
        Ok(commits)
    }

    pub fn search_commits(&self, query: &str) -> Vec<&Commit> {
        tracing::debug!("Searching commits with query: {}", query);
        let query_lower = query.to_lowercase();
        
        self.commits
            .iter()
            .filter(|commit| {
                commit.message.to_lowercase().contains(&query_lower) ||
                commit.summary.to_lowercase().contains(&query_lower) ||
                commit.author.name.to_lowercase().contains(&query_lower) ||
                commit.author.email.to_lowercase().contains(&query_lower) ||
                commit.id.as_str().starts_with(&query_lower)
            })
            .collect()
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