#[cfg(test)]
mod commit_reading_tests {
    use std::path::PathBuf;
    use twiggy::git::repository::GitRepository;
    use twiggy::git::types::{Commit, CommitId};

    fn get_test_repo_path() -> Option<PathBuf> {
        let current_dir = std::env::current_dir().ok()?;
        if current_dir.join(".git").exists() {
            Some(current_dir)
        } else {
            None
        }
    }

    #[test]
    fn test_commit_loading_basic() {
        let repo_path = match get_test_repo_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: No git repository found in current directory");
                return;
            }
        };

        let mut repo = GitRepository::open(&repo_path)
            .expect("Failed to open test repository");

        let result = repo.load_commits(Some(10));
        assert!(result.is_ok(), "Failed to load commits: {:?}", result.err());

        let commits = repo.get_commits();
        assert!(!commits.is_empty(), "No commits were loaded");
        assert!(commits.len() <= 10, "Loaded more commits than requested");

        println!("Successfully loaded {} commits", commits.len());
    }

    #[test]
    fn test_commit_caching() {
        let repo_path = match get_test_repo_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: No git repository found in current directory");
                return;
            }
        };

        let mut repo = GitRepository::open(&repo_path)
            .expect("Failed to open test repository");

        repo.load_commits(Some(5)).expect("Failed to load commits");
        let initial_cache_size = repo.cache_size();
        
        assert!(initial_cache_size > 0, "Cache should contain commits after loading");

        repo.load_commits(Some(5)).expect("Failed to reload commits");
        let final_cache_size = repo.cache_size();
        
        assert_eq!(initial_cache_size, final_cache_size, "Cache size should remain stable on reload");

        println!("Cache working correctly with {} commits", final_cache_size);
    }

    #[test]
    fn test_commit_data_integrity() {
        let repo_path = match get_test_repo_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: No git repository found in current directory");
                return;
            }
        };

        let mut repo = GitRepository::open(&repo_path)
            .expect("Failed to open test repository");

        repo.load_commits(Some(3)).expect("Failed to load commits");
        let commits = repo.get_commits();

        for commit in commits {
            assert!(!commit.id.as_str().is_empty(), "Commit ID should not be empty");
            assert!(!commit.author.name.is_empty(), "Author name should not be empty");
            assert!(!commit.tree_id.is_empty(), "Tree ID should not be empty");
            
            println!("Commit {}: {} by {}", 
                commit.id.as_str()[..8].to_string(),
                commit.summary,
                commit.author.name
            );
        }
    }

    #[test]
    fn test_commit_search_functionality() {
        let repo_path = match get_test_repo_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: No git repository found in current directory");
                return;
            }
        };

        let mut repo = GitRepository::open(&repo_path)
            .expect("Failed to open test repository");

        repo.load_commits(Some(50)).expect("Failed to load commits");
        
        let all_commits = repo.get_commits();
        if all_commits.is_empty() {
            println!("Skipping search test: No commits available");
            return;
        }

        let first_commit = &all_commits[0];
        let search_term = &first_commit.author.name[..3];
        
        let search_results = repo.search_commits(search_term);
        assert!(!search_results.is_empty(), "Search should find at least one commit");

        println!("Search for '{}' found {} commits", search_term, search_results.len());
    }

    #[test]
    fn test_performance_large_load() {
        let repo_path = match get_test_repo_path() {
            Some(path) => path,
            None => {
                println!("Skipping test: No git repository found in current directory");
                return;
            }
        };

        let mut repo = GitRepository::open(&repo_path)
            .expect("Failed to open test repository");

        let start_time = std::time::Instant::now();
        let result = repo.load_commits(Some(1000));
        let elapsed = start_time.elapsed();

        assert!(result.is_ok(), "Failed to load large number of commits");
        
        let commits = repo.get_commits();
        println!("Performance test: Loaded {} commits in {:?}", commits.len(), elapsed);
        
        assert!(elapsed.as_secs() < 10, "Loading should complete within 10 seconds");
    }

    #[test]
    fn test_empty_repository_handling() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let repo_path = temp_dir.path();
        
        git2::Repository::init(repo_path).expect("Failed to initialize empty repository");
        
        let mut repo = GitRepository::open(repo_path)
            .expect("Failed to open empty repository");

        let result = repo.load_commits(Some(10));
        assert!(result.is_ok(), "Loading commits from empty repository should not fail");
        
        let commits = repo.get_commits();
        assert!(commits.is_empty(), "Empty repository should have no commits");
        
        println!("Empty repository handling works correctly");
    }
}