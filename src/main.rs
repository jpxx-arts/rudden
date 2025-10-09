use clap::{Args, Parser, Subcommand};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Error, ErrorKind, Write};
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    Check,
    Add(AddArgs),
    Update(UpdateArgs),
    Rm(RmArgs),
    Show,
}

#[derive(Args, Debug)]
struct AddArgs {
    #[arg(short, long)]
    message: String,
    #[arg(short, long)]
    importance: Option<String>,
}

#[derive(Args, Debug)]
struct UpdateArgs {
    id: u32,
    #[arg(short, long)]
    status: Option<String>,
    #[arg(short, long)]
    importance: Option<String>,
}

#[derive(Args, Debug)]
struct RmArgs {
    id: u32,
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Status {
    Pending,
    Finished,
}

impl FromStr for Status {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Status::Pending),
            "finished" => Ok(Status::Finished),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            Status::Pending => "pending",
            Status::Finished => "finished",
        };
        write!(f, "{}", status)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Importance {
    Normal,
    Important,
    Urgent,
}

impl FromStr for Importance {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Importance::Normal),
            "important" => Ok(Importance::Important),
            "urgent" => Ok(Importance::Urgent),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Importance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let importance = match self {
            Importance::Normal => "normal",
            Importance::Important => "important",
            Importance::Urgent => "urgent",
        };
        write!(f, "{}", importance)
    }
}

struct ToDoList {
    tasks: Vec<Task>,
}

impl ToDoList {
    fn load(path: &str) -> io::Result<ToDoList> {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let tasks: Vec<Task> = reader
                    .lines()
                    .map_while(Result::ok)
                    .filter_map(|line| Task::from_str(&line).ok())
                    .collect();

                Ok(ToDoList { tasks })
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(ToDoList { tasks: vec![] }),
            Err(e) => Err(e),
        }
    }

    fn save(&self, path: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        for task in &self.tasks {
            writeln!(file, "{}", task.to_csv_line())?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Task {
    id: u32,
    name: String,
    status: Status,
    importance: Importance,
}

impl Task {
    fn to_csv_line(&self) -> String {
        format!(
            "{},{},{},{}",
            self.id, self.name, self.status, self.importance
        )
    }
}

impl FromStr for Task {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(4, ',').collect();
        if parts.len() != 4 {
            return Err("Incorrect line format".to_string());
        }

        let id = parts[0].parse::<u32>().map_err(|e| e.to_string())?;
        let name = parts[1].to_string();
        let status = Status::from_str(parts[2]).map_err(|_| "Invalid Status".to_string())?;
        let importance =
            Importance::from_str(parts[3]).map_err(|_| "Invalid Importance".to_string())?;

        Ok(Task {
            id,
            name,
            status,
            importance,
        })
    }
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let rudden_dir = "./.rudden";
    fs::create_dir_all(rudden_dir)?;
    let rudden_file_path = rudden_dir.to_string() + "/.rudden";

    let mut to_do_list = ToDoList::load(&rudden_file_path)?;

    match cli.mode {
        Mode::Check => {
            let logs_path = "./.git/logs/HEAD";
            let logs_content = match fs::read_to_string(logs_path) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading git logs: {}", e);
                    return Err(e);
                }
            };

            // REFACTOR:
            /*
             * This flag may seem unnecessary, but it serves to solve two problems:
             *
             * 1. (Borrow Checker) The primary reason: The `for` loop holds a mutable borrow
             * on `to_do_list`. Calling `to_do_list.save()` inside the loop would require a
             * simultaneous immutable borrow, which Rust's ownership rules forbid.
             * The flag allows us to defer the save operation until after the mutable borrow's scope has ended.
             *
             * 2. (Performance) It ensures the potentially expensive file I/O from `.save()`
             * is performed at most once, and only if at least one task was actually modified.
             */
            let mut tasks_updated = false;
            for task in &mut to_do_list.tasks {
                if task.status == Status::Pending && logs_content.contains(&task.name) {
                    println!("Task '{}' finished!", task.name);
                    task.status = Status::Finished;
                    tasks_updated = true;
                }
            }

            if tasks_updated {
                to_do_list.save(&rudden_file_path)?;
                println!("Tasks updated successfully.");
            } else {
                println!("No tasks to update.");
            }
        }

        Mode::Add(args) => {
            let importance_str = args.importance.unwrap_or_else(|| "normal".to_string());
            let importance = match importance_str.parse::<Importance>() {
                Ok(imp) => imp,
                Err(_) => {
                    let err_msg = format!(
                        "Error: '{}' is not a valid importance. Use 'normal', 'important', or 'urgent'.",
                        importance_str
                    );
                    return Err(Error::new(ErrorKind::InvalidInput, err_msg));
                }
            };

            let new_id = to_do_list.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;

            let new_task = Task {
                id: new_id,
                name: args.message,
                status: Status::Pending,
                importance,
            };

            to_do_list.tasks.push(new_task);
            to_do_list.save(&rudden_file_path)?;
            println!("Successfully added task with ID: {}", new_id);
        }

        Mode::Update(args) => {
            let mut task_found = false;
            for task in to_do_list.tasks.iter_mut() {
                if task.id == args.id {
                    // REFACTOR:
                    /*
                     * This flag may seem unnecessary, but it serves to solve two problems:
                     *
                     * 1. (Borrow Checker) The primary reason: The `for` loop holds a mutable borrow
                     * on `to_do_list`. Calling `to_do_list.save()` inside the loop would require a
                     * simultaneous immutable borrow, which Rust's ownership rules forbid.
                     * The flag allows us to defer the save operation until after the mutable borrow's scope has ended.
                     *
                     * 2. (Performance) It ensures the potentially expensive file I/O from `.save()`
                     * is performed at most once, and only if at least one task was actually modified.
                     */
                    task_found = true;
                    if let Some(new_status_str) = args.status {
                        match new_status_str.parse::<Status>() {
                            Ok(new_status) => task.status = new_status,
                            Err(_) => {
                                let err_msg = format!(
                                    "Error: '{}' is not a valid status. Use 'pending' or 'finished'.",
                                    new_status_str
                                );
                                return Err(Error::new(ErrorKind::InvalidInput, err_msg));
                            }
                        }
                    }
                    if let Some(new_importance_str) = args.importance {
                        match new_importance_str.parse::<Importance>() {
                            Ok(new_importance) => task.importance = new_importance,
                            Err(_) => {
                                let err_msg = format!(
                                    "Error: '{}' is not a valid importance. Use 'normal', 'important', or 'urgent'.",
                                    new_importance_str
                                );
                                return Err(Error::new(ErrorKind::InvalidInput, err_msg));
                            }
                        }
                    }
                    break;
                }
            }

            if task_found {
                to_do_list.save(&rudden_file_path)?;
                println!("Successfully updated task with ID: {}", args.id);
            } else {
                eprintln!("Error: Task with ID {} not found.", args.id);
            }
        }

        Mode::Show => {
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

        Mode::Rm(args) => {
            let initial_len = to_do_list.tasks.len();
            to_do_list.tasks.retain(|task| task.id != args.id);

            if to_do_list.tasks.len() < initial_len {
                to_do_list.save(&rudden_file_path)?;
                println!("Successfully removed task with ID: {}", args.id);
            } else {
                eprintln!("Error: Task with ID {} not found.", args.id);
            }
        }
    }

    Ok(())
}
