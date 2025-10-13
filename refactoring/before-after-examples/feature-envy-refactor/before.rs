// src/commands.rs

use crate::cli::{AddArgs, RmArgs, UpdateArgs};
use crate::task::{Importance, Status, Task, ToDoList};
use std::fs;
use std::io::{self, Error, ErrorKind};

pub fn check_tasks(to_do_list: &mut ToDoList, file_path: &str) -> io::Result<()> {
    let logs_path = "./.git/logs/HEAD";
    let logs_content = fs::read_to_string(logs_path)?;

    let mut tasks_updated = false;
    for task in &mut to_do_list.tasks {
        if task.status == Status::Pending && logs_content.contains(&task.name) {
            println!("Task '{}' finished!", task.name);
            task.status = Status::Finished;
            tasks_updated = true;
        }
    }

    if tasks_updated {
        to_do_list.save(file_path)?;
        println!("Tasks updated successfully.");
    } else {
        println!("No tasks to update.");
    }
    Ok(())
}

pub fn add_task(to_do_list: &mut ToDoList, args: AddArgs, file_path: &str) -> io::Result<()> {
    let importance_str = args.importance.unwrap_or_else(|| "normal".to_string());
    let importance = importance_str.parse::<Importance>().map_err(|_| {
        let err_msg = format!(
            "Error: '{}' is not a valid importance. Use 'normal', 'important', or 'urgent'.",
            importance_str
        );
        Error::new(ErrorKind::InvalidInput, err_msg)
    })?;

    let new_id = to_do_list.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;

    let new_task = Task {
        id: new_id,
        name: args.message,
        status: Status::Pending,
        importance,
    };

    to_do_list.tasks.push(new_task);
    to_do_list.save(file_path)?;
    println!("Successfully added task with ID: {}", new_id);
    Ok(())
}

pub fn update_task(to_do_list: &mut ToDoList, args: UpdateArgs, file_path: &str) -> io::Result<()> {
    let task_id_to_update = args.id;
    if let Some(task) = to_do_list.tasks.iter_mut().find(|t| t.id == task_id_to_update) {
        if let Some(new_status_str) = args.status {
            task.status = new_status_str.parse::<Status>().map_err(|_| {
                let err_msg = format!("Error: '{}' is not a valid status.", new_status_str);
                Error::new(ErrorKind::InvalidInput, err_msg)
            })?;
        }
        if let Some(new_importance_str) = args.importance {
            task.importance = new_importance_str.parse::<Importance>().map_err(|_| {
                let err_msg = format!("Error: '{}' is not a valid importance.", new_importance_str);
                Error::new(ErrorKind::InvalidInput, err_msg)
            })?;
        }
        to_do_list.save(file_path)?;
        println!("Successfully updated task with ID: {}", task_id_to_update);
    } else {
        eprintln!("Error: Task with ID {} not found.", task_id_to_update);
    }
    Ok(())
}


pub fn remove_task(to_do_list: &mut ToDoList, args: RmArgs, file_path: &str) -> io::Result<()> {
    let initial_len = to_do_list.tasks.len();
    to_do_list.tasks.retain(|task| task.id != args.id);

    if to_do_list.tasks.len() < initial_len {
        to_do_list.save(file_path)?;
        println!("Successfully removed task with ID: {}", args.id);
    } else {
        eprintln!("Error: Task with ID {} not found.", args.id);
    }
    Ok(())
}

pub fn show_tasks(to_do_list: &ToDoList) {
    if to_do_list.tasks.is_empty() {
        println!("No tasks to show.");
    } else {
        println!("There are {} Tasks:", to_do_list.tasks.len());
        for task in &to_do_list.tasks {
            println!(
                "- [id: {}] {} (Status: {}, Importance: {})",
                task.id, task.name, task.status, task.importance
            );
        }
    }
}