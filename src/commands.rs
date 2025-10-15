use crate::cli::{AddArgs, RmArgs, UpdateArgs};
use crate::task::{Importance, Status, ToDoList};
use std::fs;
use std::io::{self, Error, ErrorKind};

pub fn check_tasks(to_do_list: &mut ToDoList, file_path: &str) -> io::Result<()> {
    let logs_path = "./.git/logs/HEAD";
    let logs_content = fs::read_to_string(logs_path)?;

    if to_do_list.update_statuses_from_logs(&logs_content) {
        to_do_list.save(file_path)?;
        println!("Tasks updated successfully.");
    } else {
        println!("No tasks to update.");
    }
    Ok(())
}

pub fn add_task(to_do_list: &mut ToDoList, args: AddArgs, file_path: &str) -> io::Result<()> {
    let importance_str = args.importance.unwrap_or_else(|| "normal".to_string());
    let importance = importance_str.parse::<Importance>().map_err(|()| {
        let err_msg = format!("Error: '{importance_str}' is not a valid importance.");
        Error::new(ErrorKind::InvalidInput, err_msg)
    })?;

    let new_id = to_do_list.add_task(args.message, importance);
    
    to_do_list.save(file_path)?;
    println!("Successfully added task with ID: {new_id}");
    Ok(())
}

pub fn update_task(to_do_list: &mut ToDoList, args: UpdateArgs, file_path: &str) -> io::Result<()> {
    let status = args.status
        .map(|s| s.parse::<Status>())
        .transpose()
        .map_err(|()| Error::new(ErrorKind::InvalidInput, "Invalid status provided."))?;

    let importance = args.importance
        .map(|s| s.parse::<Importance>())
        .transpose()
        .map_err(|()| Error::new(ErrorKind::InvalidInput, "Invalid importance provided."))?;

    if to_do_list.update_task(args.id, status, importance) {
        to_do_list.save(file_path)?;
        println!("Successfully updated task with ID: {}", args.id);
    } else {
        eprintln!("Error: Task with ID {} not found.", args.id);
    }
    Ok(())
}

pub fn remove_task(to_do_list: &mut ToDoList, args: RmArgs, file_path: &str) -> io::Result<()> {
    if to_do_list.remove_task(args.id) {
        to_do_list.save(file_path)?;
        println!("Successfully removed task with ID: {}", args.id);
    } else {
        eprintln!("Error: Task with ID {} not found.", args.id);
    }
    Ok(())
}

pub fn show_tasks(to_do_list: &ToDoList) {
    if to_do_list.is_empty() {
        println!("No tasks to show.");
    } else {
        println!("There are {} Tasks:", to_do_list.tasks().len());
        for task in to_do_list.tasks() {
            println!(
                "- [id: {}] {} (Status: {}, Importance: {})",
                task.id, task.name, task.status, task.importance
            );
        }
    }
}
