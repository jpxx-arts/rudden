// src/task.rs

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::str::FromStr;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Status {
    Pending,
    Finished,
}

impl FromStr for Status {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "finished" => Ok(Self::Finished),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            Self::Pending => "pending",
            Self::Finished => "finished",
        };
        write!(f, "{}", status)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Importance {
    Normal,
    Important,
    Urgent,
}

impl FromStr for Importance {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Self::Normal),
            "important" => Ok(Self::Important),
            "urgent" => Ok(Self::Urgent),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Importance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let importance = match self {
            Self::Normal => "normal",
            Self::Important => "important",
            Self::Urgent => "urgent",
        };
        write!(f, "{}", importance)
    }
}

#[derive(Clone)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub status: Status,
    pub importance: Importance,
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

        Ok(Self {
            id,
            name,
            status,
            importance,
        })
    }
}

pub struct ToDoList {
    tasks: Vec<Task>,
}

impl ToDoList {
    pub fn load(path: &str) -> io::Result<Self> {
        match File::open(path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let tasks: Vec<Task> = reader
                    .lines()
                    .map_while(Result::ok)
                    .filter_map(|line| Task::from_str(&line).ok())
                    .collect();

                Ok(Self{ tasks })
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self{ tasks: vec![] }),
            Err(e) => Err(e),
        }
    }

    pub fn save(&self, path: &str) -> io::Result<()> {
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

    fn get_next_id(&self) -> u32 {
        self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }

    pub fn add_task(&mut self, name: String, importance: Importance) -> u32 {
        let new_id = self.get_next_id();

        let new_task = Task {
            id: new_id,
            name,
            status: Status::Pending,
            importance,
        };

        self.tasks.push(new_task);

        new_id
    }

    pub fn update_task(&mut self, id: u32, new_status: Option<Status>, new_importance: Option<Importance>) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            if let Some(status) = new_status {
                task.status = status;
            }
            if let Some(importance) = new_importance {
                task.importance = importance;
            }
            return true;
        }
        false
    }

    pub fn remove_task(&mut self, id: u32) -> bool {
        let initial_len = self.tasks.len();
        self.tasks.retain(|task| task.id != id);
        self.tasks.len() < initial_len
    }

    pub fn update_statuses_from_logs(&mut self, logs_content: &str) -> bool {
        let mut tasks_updated = false;
        for task in &mut self.tasks {
            if task.status == Status::Pending && logs_content.contains(&task.name) {
                println!("Task '{}' finished!", task.name);
                task.status = Status::Finished;
                tasks_updated = true;
            }
        }
        tasks_updated
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

}