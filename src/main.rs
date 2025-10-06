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
    Update,
    Add,
}

struct Task {
    name: String,
}

fn main() {
    let cli = Cli::parse();

    match cli.mode {
        Mode::Update => {
            let logs_path = "./.git/logs/HEAD";
            let logs_content = fs::read_to_string(logs_path).expect("No local commits yet");

            let rudden_file_path = "./.rudden";
            let rudden_content = fs::read_to_string(rudden_file_path).expect("No rudden file");

            for task_line in rudden_content.lines() {
                for line in logs_content.lines() {
                    if line.contains(task_line) {
                        println!("I found ya!");
                        break;
                    }
                }
            }
        }
        Mode::Add => {
        }
    }
}
