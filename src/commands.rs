use crate::cli::{AddArgs, RmArgs, UpdateArgs};
use crate::task::{Importance, Status, ToDoList};
use std::io;
use std::path::Path;
use std::process::Command;

/// Executes `git log` to get a string containing all commit subjects.
fn get_git_log_subjects(repo_path: &Path) -> io::Result<String> {
    let output = Command::new("git")
        .arg("log")
        .arg("--pretty=format:%s") // Get only the subject line of each commit
        .current_dir(repo_path) // Run the command in the specified directory
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to execute git log: {}", stderr),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Adds a new task to the list. Takes a reference to AddArgs.
pub fn add_task(to_do_list: &mut ToDoList, args: &AddArgs) -> Result<String, String> {
    let importance_str = args.importance.as_deref().unwrap_or("normal");
    let importance = match importance_str.parse::<Importance>() {
        Ok(imp) => imp,
        Err(_) => return Err(format!("'{}' is not a valid importance. Use 'normal', 'important', or 'urgent'.", importance_str)),
    };

    let new_id = to_do_list.add_task(args.message.clone(), importance);
    Ok(format!("Successfully added task with ID: {}", new_id))
}

/// Updates an existing task's status or importance. Takes a reference to UpdateArgs.
pub fn update_task(to_do_list: &mut ToDoList, args: &UpdateArgs) -> Result<String, String> {
    let status = match args.status.as_deref().map(|s| s.parse::<Status>()).transpose() {
        Ok(s) => s,
        Err(_) => return Err("Invalid status provided. Use 'pending' or 'finished'.".to_string()),
    };

    let importance = match args.importance.as_deref().map(|s| s.parse::<Importance>()).transpose() {
        Ok(i) => i,
        Err(_) => return Err("Invalid importance provided. Use 'normal', 'important', or 'urgent'.".to_string()),
    };

    if to_do_list.update_task(args.id, status, importance) {
        Ok(format!("Successfully updated task with ID: {}", args.id))
    } else {
        Err(format!("Task with ID {} not found.", args.id))
    }
}

/// Removes a task from the list. Takes a reference to RmArgs.
pub fn remove_task(to_do_list: &mut ToDoList, args: &RmArgs) -> Result<String, String> {
    if to_do_list.remove_task(args.id) {
        Ok(format!("Successfully removed task with ID: {}", args.id))
    } else {
        Err(format!("Task with ID {} not found.", args.id))
    }
}

/// Generates a string displaying all tasks.
pub fn show_tasks(to_do_list: &ToDoList) -> Result<String, String> {
    if to_do_list.is_empty() {
        return Ok("No tasks to show.".to_string());
    }

    let mut output = format!("There are {} Tasks:\n", to_do_list.tasks().len());
    for task in to_do_list.tasks() {
        let line = format!(
            "- [id: {}] {} (Status: {}, Importance: {})\n",
            task.id, task.name, task.status, task.importance
        );
        output.push_str(&line);
    }
    Ok(output.trim_end().to_string())
}

/// Checks git logs and updates task statuses by calling `git log`.
pub fn check_tasks(to_do_list: &mut ToDoList, repo_path: &Path) -> io::Result<String> {
    let logs_content = match get_git_log_subjects(repo_path) {
        Ok(content) => content,
        Err(e) => return Err(e),
    };

    if to_do_list.update_statuses_from_logs(&logs_content) {
        Ok("Tasks updated successfully based on git logs.".to_string())
    } else {
        Ok("No tasks to update from git logs.".to_string())
    }
}