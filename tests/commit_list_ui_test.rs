#[cfg(test)]
mod commit_list_ui_tests {
    use crate::git::types::{Commit, CommitId, Signature};
    use crate::ui::components::commit_list::CommitListComponent;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;

    fn create_test_commit(id: &str, summary: &str, author: &str, timestamp: i64) -> Commit {
        let commit_id = CommitId::from_str(id).unwrap_or_else(|_| CommitId::from_str("0000000000000000000000000000000000000000").unwrap());
        
        Commit {
            id: commit_id,
            summary: summary.to_string(),
            message: format!("{}\n\nDetailed commit message", summary),
            author: Signature {
                name: author.to_string(),
                email: format!("{}@example.com", author.to_lowercase()),
                time: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now()),
            },
            committer: Signature {
                name: author.to_string(),
                email: format!("{}@example.com", author.to_lowercase()),
                time: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now()),
            },
            parents: vec![],
            tree_id: "tree123".to_string(),
            is_merge: false,
        }
    }

    fn create_test_commits(count: usize) -> Vec<Commit> {
        let mut commits = Vec::new();
        let base_timestamp = 1695000000;
        
        for i in 0..count {
            let commit = create_test_commit(
                &format!("{:040x}", i),
                &format!("feat: implement feature {}", i + 1),
                &format!("Developer{}", (i % 5) + 1),
                base_timestamp + (i as i64 * 3600),
            );
            commits.push(commit);
        }
        
        commits
    }

    #[test]
    fn test_commit_list_creation() {
        let commit_list = CommitListComponent::new();
        assert_eq!(commit_list.selected_commit, None);
        assert_eq!(commit_list.hovered_commit, None);
        assert_eq!(commit_list.scroll_offset, 0.0);
        assert!(commit_list.item_height > 0.0);
    }

    #[test]
    fn test_commit_selection() {
        let mut commit_list = CommitListComponent::new();
        let commits = create_test_commits(5);
        
        let first_commit_id = commits[0].id;
        commit_list.select_commit(Some(first_commit_id));
        
        assert_eq!(commit_list.selected_commit, Some(first_commit_id));
    }

    #[test]
    fn test_commit_navigation() {
        let mut commit_list = CommitListComponent::new();
        let commits = create_test_commits(10);
        
        commit_list.select_commit(Some(commits[0].id));
        
        commit_list.select_next(&commits);
        assert_eq!(commit_list.selected_commit, Some(commits[1].id));
        
        commit_list.select_previous(&commits);
        assert_eq!(commit_list.selected_commit, Some(commits[0].id));
        
        commit_list.select_previous(&commits);
        assert_eq!(commit_list.selected_commit, Some(commits[0].id));
    }

    #[test]
    fn test_commit_navigation_bounds() {
        let mut commit_list = CommitListComponent::new();
        let commits = create_test_commits(3);
        
        commit_list.select_commit(Some(commits[2].id));
        
        commit_list.select_next(&commits);
        assert_eq!(commit_list.selected_commit, Some(commits[2].id));
        
        commit_list.select_first(&commits);
        assert_eq!(commit_list.selected_commit, Some(commits[0].id));
        
        commit_list.select_last(&commits);
        assert_eq!(commit_list.selected_commit, Some(commits[2].id));
    }

    #[test]
    fn test_empty_commit_list() {
        let mut commit_list = CommitListComponent::new();
        let empty_commits: Vec<Commit> = vec![];
        
        commit_list.select_next(&empty_commits);
        assert_eq!(commit_list.selected_commit, None);
        
        commit_list.select_previous(&empty_commits);
        assert_eq!(commit_list.selected_commit, None);
        
        commit_list.select_first(&empty_commits);
        assert_eq!(commit_list.selected_commit, None);
        
        commit_list.select_last(&empty_commits);
        assert_eq!(commit_list.selected_commit, None);
    }

    #[test]
    fn test_large_commit_list_performance() {
        let start_time = std::time::Instant::now();
        let commits = create_test_commits(1000);
        let creation_time = start_time.elapsed();
        
        assert!(creation_time.as_millis() < 100, "Creating 1000 commits should take less than 100ms");
        
        let mut commit_list = CommitListComponent::new();
        
        let selection_start = std::time::Instant::now();
        commit_list.select_commit(Some(commits[500].id));
        let selection_time = selection_start.elapsed();
        
        assert!(selection_time.as_millis() < 10, "Selecting a commit should take less than 10ms");
        
        let navigation_start = std::time::Instant::now();
        for _ in 0..100 {
            commit_list.select_next(&commits);
        }
        let navigation_time = navigation_start.elapsed();
        
        assert!(navigation_time.as_millis() < 50, "100 navigation operations should take less than 50ms");
    }

    #[test]
    fn test_commit_list_state_consistency() {
        let mut commit_list = CommitListComponent::new();
        let commits = create_test_commits(5);
        
        commit_list.select_commit(Some(commits[2].id));
        assert_eq!(commit_list.selected_commit, Some(commits[2].id));
        
        commit_list.select_commit(None);
        assert_eq!(commit_list.selected_commit, None);
        
        commit_list.select_commit(Some(commits[4].id));
        assert_eq!(commit_list.selected_commit, Some(commits[4].id));
    }

    #[test]
    fn test_commit_list_with_duplicates() {
        let mut commit_list = CommitListComponent::new();
        let mut commits = create_test_commits(3);
        commits.push(commits[0].clone());
        
        commit_list.select_commit(Some(commits[0].id));
        assert_eq!(commit_list.selected_commit, Some(commits[0].id));
        
        commit_list.select_next(&commits);
        assert_eq!(commit_list.selected_commit, Some(commits[1].id));
    }

    #[test]
    fn test_commit_metadata_integrity() {
        let commits = create_test_commits(3);
        
        assert_eq!(commits[0].summary, "feat: implement feature 1");
        assert_eq!(commits[1].summary, "feat: implement feature 2");
        assert_eq!(commits[2].summary, "feat: implement feature 3");
        
        assert_eq!(commits[0].author.name, "Developer1");
        assert_eq!(commits[1].author.name, "Developer2");
        assert_eq!(commits[2].author.name, "Developer3");
        
        assert!(commits[0].author.time < commits[1].author.time);
        assert!(commits[1].author.time < commits[2].author.time);
    }

    #[test]
    fn test_commit_list_scroll_behavior() {
        let mut commit_list = CommitListComponent::new();
        
        assert_eq!(commit_list.scroll_offset, 0.0);
        
        commit_list.scroll_offset = 100.0;
        assert_eq!(commit_list.scroll_offset, 100.0);
        
        commit_list.scroll_offset = -50.0;
        assert_eq!(commit_list.scroll_offset, -50.0);
    }
}