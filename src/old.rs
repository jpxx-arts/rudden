/*
use std::fmt;
use std::str::FromStr;
use clap::{Parser, Subcommand, Args};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

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
    Update,
    Rm,
    Show,
}

#[derive(Args, Debug)]
struct AddArgs {
    #[arg(short, long)]
    message: String,

    #[arg(short, long)]
    importance: Option<String>,
}

#[derive(PartialEq, Eq, Debug)]
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

#[derive(PartialEq, Eq, Debug)]
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
    tasks_number: u32,
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

                Ok(ToDoList { tasks_number: tasks.len() as u32, tasks  })
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(ToDoList { tasks: vec![], tasks_number: 0 }),
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
            self.id,
            self.name,
            self.status,
            self.importance
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

    let rudden_file_path = "./.rudden/.rudden";
    let mut to_do_list = ToDoList::load(rudden_file_path)?;

    match cli.mode {
        Mode::Check => {
            let logs_path = "./.git/logs/HEAD";
            let logs_content = fs::read_to_string(logs_path).expect("No local commits yet");

            for task in &mut to_do_list.tasks {
                if task.status == Status::Pending && logs_content.contains(&task.name) {
                    println!("-> Task '{}' finished!", task.name);
                    task.status = Status::Finished;
                }
            }

            to_do_list.save(rudden_file_path)?;
        }

        Mode::Add(args) => {
            println!("-> Modo 'Add' ativado!");

            let commit = &args.message;
            println!("Mensagem da tarefa: '{}'", commit);

            let importance = if let Some(imp) = &args.importance {
                match imp.parse::<Importance>() {
                    Ok(importance) => importance,
                    Err(_) => {
                        panic!("This importance is not defined")
                    }
                }
            } else {
                println!("Importância não foi definida");
                Importance::Normal
            };

            let mut tasks = OpenOptions::new()
                .append(true)
                .open(rudden_file_path)?;
            
            writeln!(tasks, "{},{},pending,{}", to_do_list.tasks_number, commit, importance).expect("Couldn't write in .rudden");
        }

        Mode::Update => {}

        Mode::Show => {
            println!("There are {} Tasks:", to_do_list.tasks_number);
            for task in to_do_list.tasks {
                println!(
                    "- [id: {}] {} (Status: {:?}, Importance: {:?})",
                    task.id, task.name, task.status, task.importance
                );
            }
        }

        Mode::Rm => {}
    }

    Ok(())
}
*/
