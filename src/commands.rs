use crate::cli::{AddArgs, RmArgs, UpdateArgs, BenchArgs};
use crate::task::{Importance, Status, ToDoList};
use crate::persistence;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;

/// Adds a new task to the list. Takes a reference to AddArgs.
pub fn add_task(to_do_list: &mut ToDoList, args: &AddArgs) -> Result<String, String> {
    let importance_str = args.importance.as_deref().unwrap_or("normal");
    let importance = match importance_str.parse::<Importance>() {
        Ok(imp) => imp,
        Err(_) => return Err(format!("'{}' is not a valid importance. Use 'normal', 'important', or 'urgent'.", importance_str)),
    };

    // We need to clone the message because args is a reference.
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
    // Remove the final newline for a cleaner output
    Ok(output.trim_end().to_string())
}

/// Checks git logs and updates task statuses.
pub fn check_tasks(to_do_list: &mut ToDoList, repo_path: &Path) -> io::Result<String> {
    let logs_path = repo_path.join(".git").join("logs").join("HEAD");
    let logs_content = match fs::read_to_string(logs_path) {
        Ok(content) => content,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok("No git repository found, can't check logs.".to_string());
        }
        Err(e) => return Err(e),
    };

    if to_do_list.update_statuses_from_logs(&logs_content) {
        Ok("Tasks updated successfully based on git logs.".to_string())
    } else {
        Ok("No tasks to update from git logs.".to_string())
    }
}

/// Runs a benchmark comparison between the slow (read/write) and fast (append-only) add operations.
pub fn run_benchmark(args: &BenchArgs) -> Result<String, String> {
    let num_tasks = args.tasks;
    let slow_path = Path::new(".rudden_slow_bench.csv");
    let fast_path = Path::new(".rudden_fast_bench.csv");
    let meta_path = Path::new(".rudden_meta.json");

    println!("Starting benchmark with {} tasks...", num_tasks);

    // --- SLOW METHOD (Read-Modify-Write) ---
    persistence::clear_benchmark_data(slow_path, fast_path, meta_path).map_err(|e| e.to_string())?;
    let mut slow_list = ToDoList::default();
    let slow_start = Instant::now();
    for i in 0..num_tasks {
        slow_list.add_task(format!("Task {}", i), Importance::Normal);
        // This save call is the O(N) bottleneck
        slow_list.save(slow_path.to_str().unwrap()).map_err(|e| e.to_string())?;
    }
    let slow_duration = slow_start.elapsed();

    println!("Slow method finished.");

    // --- FAST METHOD (Append-Only) ---
    let fast_start = Instant::now();
    for i in 0..num_tasks {
        persistence::add_task_fast(fast_path, meta_path, format!("Task {}", i), Importance::Normal)
            .map_err(|e| e.to_string())?;
    }
    let fast_duration = fast_start.elapsed();
    
    println!("Fast method finished.");

    // --- CLEANUP ---
    persistence::clear_benchmark_data(slow_path, fast_path, meta_path).map_err(|e| e.to_string())?;

    // --- REPORT ---
    let report = format!(
        "\n--- Benchmark Results ---\n\
        Tasks Added: {}\n\n\
        Slow Method (O(N) Read/Write): {:?}\n\
        Fast Method (O(1) Append-Only): {:?}\n\n\
        Conclusion: The append-only method was {:.2}x faster.",
        num_tasks,
        slow_duration,
        fast_duration,
        slow_duration.as_secs_f64() / fast_duration.as_secs_f64()
    );
    
    Ok(report)
}
