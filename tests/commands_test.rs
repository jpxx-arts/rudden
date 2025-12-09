// tests/commands_test.rs

use rudden::cli::{AddArgs, RmArgs, UpdateArgs};
use rudden::commands;
use rudden::task::{Importance, Status, ToDoList};
use std::process::Command;

// --- Basic Command Tests ---

#[test]
fn test_add_task_success() {
    let mut to_do_list = ToDoList::default();
    let args = AddArgs {
        message: "Test this function".to_string(),
        importance: Some("urgent".to_string()),
    };
    let result = commands::add_task(&mut to_do_list, &args);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Successfully added task with ID: 1");
    assert_eq!(to_do_list.tasks().len(), 1);
}

#[test]
fn test_add_task_invalid_importance() {
    let mut to_do_list = ToDoList::default();
    let args = AddArgs {
        message: "A task".to_string(),
        importance: Some("critical".to_string()),
    };
    let result = commands::add_task(&mut to_do_list, &args);
    assert!(result.is_err());
}

#[test]
fn test_update_task_success() {
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("Initial Task".to_string(), Importance::Normal);
    let args = UpdateArgs {
        id: 1,
        status: Some("finished".to_string()),
        importance: Some("urgent".to_string()),
    };
    let result = commands::update_task(&mut to_do_list, &args);
    assert!(result.is_ok());
    let task = &to_do_list.tasks()[0];
    assert_eq!(task.status, Status::Finished);
    assert_eq!(task.importance, Importance::Urgent);
}

#[test]
fn test_update_task_not_found() {
    let mut to_do_list = ToDoList::default();
    let args = UpdateArgs { id: 99, status: None, importance: None };
    let result = commands::update_task(&mut to_do_list, &args);
    assert!(result.is_err());
}

#[test]
fn test_remove_task_success() {
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("To be deleted".to_string(), Importance::Normal);
    let args = RmArgs { id: 1 };
    let result = commands::remove_task(&mut to_do_list, &args);
    assert!(result.is_ok());
    assert!(to_do_list.is_empty());
}

#[test]
fn test_remove_task_not_found() {
    let mut to_do_list = ToDoList::default();
    let args = RmArgs { id: 99 };
    let result = commands::remove_task(&mut to_do_list, &args);
    assert!(result.is_err());
}

#[test]
fn test_show_tasks_empty() {
    let to_do_list = ToDoList::default();
    let result = commands::show_tasks(&to_do_list);
    assert_eq!(result.unwrap(), "No tasks to show.");
}

#[test]
fn test_show_tasks_with_items() {
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("First task".to_string(), Importance::Normal);
    to_do_list.add_task("Second task".to_string(), Importance::Urgent);
    let result = commands::show_tasks(&to_do_list).unwrap(); // Unwrap once
    assert!(result.contains("First task"));
    assert!(result.contains("Second task"));
}

// --- check_tasks Tests with real git repos ---

/// Helper struct to manage a temporary git repository for testing.
struct GitTestRepo {
    _temp_dir: tempfile::TempDir,
}

impl GitTestRepo {
    /// Creates a new temporary directory, initializes a git repo, and makes commits.
    fn new(commit_messages: &[&str]) -> Self {
        let temp_dir = tempfile::Builder::new().prefix("rudden-git-test-").tempdir().unwrap();
        let repo_path = temp_dir.path();

        Command::new("git").arg("init").current_dir(&repo_path).output().expect("Failed to init git repo");
        Command::new("git").args(&["config", "user.name", "Test User"]).current_dir(&repo_path).output().expect("Failed to set git user.name");
        Command::new("git").args(&["config", "user.email", "test@example.com"]).current_dir(&repo_path).output().expect("Failed to set git user.email");

        for msg in commit_messages {
            Command::new("git").args(&["commit", "--allow-empty", "-m", msg]).current_dir(&repo_path).output().expect("Failed to create empty commit");
        }

        GitTestRepo { _temp_dir: temp_dir }
    }

    fn path(&self) -> &std::path::Path {
        self._temp_dir.path()
    }
}

#[test]
fn test_check_tasks_updates_task_with_real_git_log() {
    // Arrange
    let repo = GitTestRepo::new(&["feat: Implement the new parser", "fix: A previous bug"]);
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("feat: Implement the new parser".to_string(), Importance::Important);
    to_do_list.add_task("An unrelated task".to_string(), Importance::Normal);
    
    // Act
    let result = commands::check_tasks(&mut to_do_list, repo.path());
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Tasks updated successfully based on git logs.");
    assert_eq!(to_do_list.tasks()[0].status, Status::Finished);
    assert_eq!(to_do_list.tasks()[1].status, Status::Pending);
}

#[test]
fn test_check_tasks_no_updates_with_real_git_log() {
    // Arrange
    let repo = GitTestRepo::new(&["fix: A minor bug"]);
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("A completely different task".to_string(), Importance::Normal);

    // Act
    let result = commands::check_tasks(&mut to_do_list, repo.path());

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "No tasks to update from git logs.");
    assert_eq!(to_do_list.tasks()[0].status, Status::Pending);
}

#[test]
fn test_check_tasks_no_repo_fails_git_command() {
    // Arrange
    let temp_dir = tempfile::Builder::new().prefix("rudden-no-repo-").tempdir().unwrap();
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("Some task".to_string(), Importance::Normal);

    // Act
    let result = commands::check_tasks(&mut to_do_list, temp_dir.path());
    
    // Assert
    assert!(result.is_err());
    let error_message = result.err().unwrap().to_string();
    assert!(error_message.contains("Failed to execute git log"));
}
