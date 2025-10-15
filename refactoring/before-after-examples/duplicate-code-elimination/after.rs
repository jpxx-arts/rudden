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
    pub tasks: Vec<Task>,
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
}