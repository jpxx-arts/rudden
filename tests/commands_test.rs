use rudden::cli::{AddArgs, RmArgs, UpdateArgs};
use rudden::commands;
use rudden::task::{Importance, Status, ToDoList};
use std::fs;

// Test 1: Successfully add a task
#[test]
fn test_add_task_success() {
    // Arrange
    let mut to_do_list = ToDoList::default();
    let args = AddArgs {
        message: "Test this function".to_string(),
        importance: Some("urgent".to_string()),
    };

    // Act
    let result = commands::add_task(&mut to_do_list, &args);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Successfully added task with ID: 1");
    assert_eq!(to_do_list.tasks().len(), 1);
    let task = &to_do_list.tasks()[0];
    assert_eq!(task.name, "Test this function");
    assert_eq!(task.importance, Importance::Urgent);
}

// Test 2: Fail to add a task due to invalid importance
#[test]
fn test_add_task_invalid_importance() {
    // Arrange
    let mut to_do_list = ToDoList::default();
    let args = AddArgs {
        message: "A task".to_string(),
        importance: Some("critical".to_string()),
    };

    // Act
    let result = commands::add_task(&mut to_do_list, &args);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "'critical' is not a valid importance. Use 'normal', 'important', or 'urgent'.");
    assert!(to_do_list.is_empty());
}

// Test 3: Successfully update a task
#[test]
fn test_update_task_success() {
    // Arrange
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("Initial Task".to_string(), Importance::Normal); // ID will be 1
    let args = UpdateArgs {
        id: 1,
        status: Some("finished".to_string()),
        importance: Some("urgent".to_string()),
    };

    // Act
    let result = commands::update_task(&mut to_do_list, &args);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Successfully updated task with ID: 1");
    let task = &to_do_list.tasks()[0];
    assert_eq!(task.status, Status::Finished);
    assert_eq!(task.importance, Importance::Urgent);
}

// Test 4: Fail to update a non-existent task
#[test]
fn test_update_task_not_found() {
    // Arrange
    let mut to_do_list = ToDoList::default();
    let args = UpdateArgs {
        id: 99,
        status: Some("pending".to_string()),
        importance: None,
    };

    // Act
    let result = commands::update_task(&mut to_do_list, &args);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "Task with ID 99 not found.");
}

// Test 5: Successfully remove a task
#[test]
fn test_remove_task_success() {
    // Arrange
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("To be deleted".to_string(), Importance::Normal); // ID will be 1
    let args = RmArgs { id: 1 };

    // Act
    let result = commands::remove_task(&mut to_do_list, &args);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Successfully removed task with ID: 1");
    assert!(to_do_list.is_empty());
}

// Test 6: Fail to remove a non-existent task
#[test]
fn test_remove_task_not_found() {
    // Arrange
    let mut to_do_list = ToDoList::default();
    let args = RmArgs { id: 99 };

    // Act
    let result = commands::remove_task(&mut to_do_list, &args);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "Task with ID 99 not found.");
}

// Test 7: Show tasks when the list is empty
#[test]
fn test_show_tasks_empty() {
    // Arrange
    let to_do_list = ToDoList::default();

    // Act
    let result = commands::show_tasks(&to_do_list);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "No tasks to show.");
}

// Test 8: Show tasks when the list has items
#[test]
fn test_show_tasks_with_items() {
    // Arrange
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("First task".to_string(), Importance::Normal);
    to_do_list.add_task("Second task".to_string(), Importance::Urgent);

    // Act
    let result = commands::show_tasks(&to_do_list);
    let expected_output = "There are 2 Tasks:\n- [id: 1] First task (Status: pending, Importance: normal)\n- [id: 2] Second task (Status: pending, Importance: urgent)";

    // Assert
    assert!(result.is_ok());
    let output = result.unwrap();
    // Normalize line endings for cross-platform compatibility
    let normalized_output = output.replace("\r\n", "\n");
    assert_eq!(normalized_output, expected_output.replace("\r\n", "\n"));
}

// Test 9: Fail to update a task due to invalid status
#[test]
fn test_update_task_invalid_status() {
    // Arrange
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("Initial Task".to_string(), Importance::Normal);
    let args = UpdateArgs {
        id: 1,
        status: Some("in-progress".to_string()),
        importance: None,
    };

    // Act
    let result = commands::update_task(&mut to_do_list, &args);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "Invalid status provided. Use 'pending' or 'finished'.");
}

// Test 10: Fail to update a task due to invalid importance
#[test]
fn test_update_task_invalid_importance() {
    // Arrange
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("Initial Task".to_string(), Importance::Normal);
    let args = UpdateArgs {
        id: 1,
        status: None,
        importance: Some("low".to_string()),
    };

    // Act
    let result = commands::update_task(&mut to_do_list, &args);

    // Assert
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), "Invalid importance provided. Use 'normal', 'important', or 'urgent'.");
}


/// Helper function to create a temporary git repo for testing `check_tasks`
fn setup_test_repo(log_content: &str) -> tempfile::TempDir {
    let temp_dir = tempfile::Builder::new().prefix("rudden-test-").tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git").join("logs");
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(git_dir.join("HEAD"), log_content).unwrap();
    temp_dir
}

// Test 11: check_tasks successfully updates a task
#[test]
fn test_check_tasks_updates_task() {
    // Arrange
    let temp_dir = setup_test_repo("commit abc\nfeat: Implement the new parser");
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("feat: Implement the new parser".to_string(), Importance::Important);

    // Act
    let result = commands::check_tasks(&mut to_do_list, temp_dir.path());

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Tasks updated successfully based on git logs.");
    assert_eq!(to_do_list.tasks()[0].status, Status::Finished);
}

// Test 12: check_tasks finds no tasks to update
#[test]
fn test_check_tasks_no_updates() {
    // Arrange
    let temp_dir = setup_test_repo("commit def\nfix: A minor bug");
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("A completely different task".to_string(), Importance::Normal);

    // Act
    let result = commands::check_tasks(&mut to_do_list, temp_dir.path());

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "No tasks to update from git logs.");
    assert_eq!(to_do_list.tasks()[0].status, Status::Pending);
}

// Test 13: check_tasks handles a non-existent .git directory gracefully
#[test]
fn test_check_tasks_no_repo() {
    // Arrange
    let temp_dir = tempfile::Builder::new().prefix("rudden-no-repo-").tempdir().unwrap();
    let mut to_do_list = ToDoList::default();
    to_do_list.add_task("Some task".to_string(), Importance::Normal);

    // Act
    let result = commands::check_tasks(&mut to_do_list, temp_dir.path());

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "No git repository found, can't check logs.");
}
