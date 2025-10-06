use clap::{Parser, ValueEnum};
use std::fs;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Check,
    Add,
}

#[derive(PartialEq, Eq)]
enum Status {
    Pending,
    Finished
}

#[derive(PartialEq, Eq)]
enum Importance {
    Normal,
    Important,
    Urgent
}

struct Task {
    id: u32,
    name: String,
    status: Status,
    importance: Importance
}

impl Task {
    fn new() -> Task {
        Task {
            id: 0,
            name: String::from("None"),
            status: Status::Pending,
            importance: Importance::Normal,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Check => {
            let logs_path = "./.git/logs/HEAD";
            let logs_content = fs::read_to_string(logs_path).expect("No local commits yet");

            let rudden_file_path = "./.rudden";
            let rudden_content = fs::read_to_string(rudden_file_path).expect("No rudden file");

            for rudden_line in rudden_content.lines() {
                let task_vect: Vec<&str> = rudden_line.split(',').collect();
                let mut task = Task::new();

                task.id = task_vect[0]
                    .parse()
                    .expect("Failed to parse task id to u32");
                task.name = String::from(task_vect[1]);
                task.status = match task_vect[2] {
                    "pending" => Status::Pending,
                    "finished" => Status::Finished,
                    _ => Status::Finished,
                };
                task.importance = match task_vect[3] {
                    "normal" => Importance::Normal,
                    "important" => Importance::Important,
                    "urgent" => Importance::Urgent,
                    _ => Importance::Normal,
                };

                if task.status == Status::Pending {
                    println!("id = {}, name = {}", task.id, task.name);
                }

                for log_line in logs_content.lines() {
                    if task.status == Status::Pending && log_line.contains(&task.name) {
                        task.status = Status::Finished;

                        break;
                    }
                }
            }
        }
        Mode::Add => {}
    }
}
