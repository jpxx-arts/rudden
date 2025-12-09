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

#[derive(PartialEq, Eq, Debug, Clone)]
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
        // We split from the right, because status and importance don't contain commas.
        let mut parts: Vec<&str> = s.rsplitn(3, ',').collect();
        if parts.len() != 3 {
            return Err("Incorrect line format: couldn't split into 3 parts from right".to_string());
        }
        // Because we used rsplit, the vector is reversed. Let's fix that.
        parts.reverse();
        // Now, parts[0] is "id,name", parts[1] is "status", parts[2] is "importance"
        let id_and_name = parts[0];
        let status_str = parts[1];
        let importance_str = parts[2];

        let id_name_parts: Vec<&str> = id_and_name.splitn(2, ',').collect();
        if id_name_parts.len() != 2 {
            return Err("Incorrect line format: couldn't split id and name".to_string());
        }

        let id = id_name_parts[0].parse::<u32>().map_err(|e| e.to_string())?;
        let name = id_name_parts[1].to_string();
        let status = Status::from_str(status_str).map_err(|_| "Invalid Status".to_string())?;
        let importance =
            Importance::from_str(importance_str).map_err(|_| "Invalid Importance".to_string())?;

        Ok(Self {
            id,
            name,
            status,
            importance,
        })
    }
}

#[derive(Default)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    // Test 1: Successful parsing of a valid task line
    #[test]
    fn test_task_from_str_success() {
        // Arrange
        let line = "1,Refactor the CLI module,pending,urgent";
        
        // Act
        let task = Task::from_str(line).unwrap();

        // Assert
        assert_eq!(task.id, 1);
        assert_eq!(task.name, "Refactor the CLI module");
        assert_eq!(task.status, Status::Pending);
        assert_eq!(task.importance, Importance::Urgent);
    }

    // Test 2: Failure on parsing a malformed task line
    #[test]
    fn test_task_from_str_malformed() {
        // Arrange
        let line = "1,Just one field";

        // Act
        let result = Task::from_str(line);

        // Assert
        assert!(result.is_err());
    }

    // Test 3: Failure on parsing a task with an invalid status
    #[test]
    fn test_task_from_str_invalid_status() {
        // Arrange
        let line = "2,New feature,in_progress,normal";

        // Act
        let result = Task::from_str(line);

        // Assert
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Invalid Status");
    }

    // Test 4: Add a task and verify it's in the list
    #[test]
    fn test_todolist_add_task() {
        // Arrange
        let mut list = ToDoList { tasks: vec![] };
        let name = "Write unit tests".to_string();

        // Act
        let new_id = list.add_task(name.clone(), Importance::Important);

        // Assert
        assert_eq!(new_id, 1);
        assert_eq!(list.tasks.len(), 1);
        assert_eq!(list.tasks[0].name, name);
        assert_eq!(list.tasks[0].status, Status::Pending);
        assert_eq!(list.tasks[0].importance, Importance::Important);
    }

    // Test 5: Get next ID from an empty list
    #[test]
    fn test_todolist_get_next_id_empty() {
        // Arrange
        let list = ToDoList { tasks: vec![] };

        // Act
        let next_id = list.get_next_id();

        // Assert
        assert_eq!(next_id, 1);
    }

    // Test 6: Get next ID from a populated list
    #[test]
    fn test_todolist_get_next_id_populated() {
        // Arrange
        let tasks = vec![
            Task { id: 1, name: "t1".into(), status: Status::Pending, importance: Importance::Normal },
            Task { id: 5, name: "t5".into(), status: Status::Finished, importance: Importance::Urgent },
        ];
        let list = ToDoList { tasks };

        // Act
        let next_id = list.get_next_id();

        // Assert
        assert_eq!(next_id, 6);
    }

    // Test 7: Successfully update a task's status and importance
    #[test]
    fn test_todolist_update_task_success() {
        // Arrange
        let tasks = vec![
            Task { id: 1, name: "t1".into(), status: Status::Pending, importance: Importance::Normal },
        ];
        let mut list = ToDoList { tasks };

        // Act
        let result = list.update_task(1, Some(Status::Finished), Some(Importance::Urgent));

        // Assert
        assert!(result);
        assert_eq!(list.tasks[0].status, Status::Finished);
        assert_eq!(list.tasks[0].importance, Importance::Urgent);
    }

    // Test 8: Attempt to update a non-existent task
    #[test]
    fn test_todolist_update_task_not_found() {
        // Arrange
        let mut list = ToDoList { tasks: vec![] };

        // Act
        let result = list.update_task(99, Some(Status::Finished), None);

        // Assert
        assert!(!result);
    }

    // Test 9: Successfully remove an existing task
    #[test]
    fn test_todolist_remove_task_success() {
        // Arrange
        let tasks = vec![
            Task { id: 1, name: "t1".into(), status: Status::Pending, importance: Importance::Normal },
        ];
        let mut list = ToDoList { tasks };

        // Act
        let result = list.remove_task(1);

        // Assert
        assert!(result);
        assert!(list.tasks.is_empty());
    }

    // Test 10: Attempt to remove a non-existent task
    #[test]
    fn test_todolist_remove_task_not_found() {
        // Arrange
        let tasks = vec![
            Task { id: 1, name: "t1".into(), status: Status::Pending, importance: Importance::Normal },
        ];
        let mut list = ToDoList { tasks };

        // Act
        let result = list.remove_task(99);

        // Assert
        assert!(!result);
        assert_eq!(list.tasks.len(), 1);
    }

    // Test 11: Task status changes when name is in logs
    #[test]
    fn test_update_statuses_from_logs_task_updated() {
        // Arrange
        let task_name = "feat: Implement the core logic".to_string();
        let tasks = vec![
            Task { id: 1, name: task_name.clone(), status: Status::Pending, importance: Importance::Normal },
        ];
        let mut list = ToDoList { tasks };
        let logs_content = "commit 1234\nAuthor: a\nDate: now\n\n    feat: Implement the core logic\n";

        // Act
        let updated = list.update_statuses_from_logs(logs_content);

        // Assert
        assert!(updated);
        assert_eq!(list.tasks[0].status, Status::Finished);
    }

    // Test 12: Task status does not change if already finished
    #[test]
    fn test_update_statuses_from_logs_already_finished() {
        // Arrange
        let task_name = "fix: A critical bug".to_string();
        let tasks = vec![
            Task { id: 1, name: task_name.clone(), status: Status::Finished, importance: Importance::Urgent },
        ];
        let mut list = ToDoList { tasks };
        let logs_content = "commit 5678\nAuthor: b\nDate: past\n\n    fix: A critical bug\n";

        // Act
        let updated = list.update_statuses_from_logs(logs_content);

        // Assert
        assert!(!updated);
        assert_eq!(list.tasks[0].status, Status::Finished);
    }

    // Test 13: Task status does not change if name is not in logs
    #[test]
    fn test_update_statuses_from_logs_not_in_logs() {
        // Arrange
        let tasks = vec![
            Task { id: 1, name: "docs: Update README".into(), status: Status::Pending, importance: Importance::Normal },
        ];
        let mut list = ToDoList { tasks };
        let logs_content = "commit 9012\nAuthor: c\nDate: future\n\n    chore: Release new version\n";

        // Act
        let updated = list.update_statuses_from_logs(logs_content);
        
        // Assert
        assert!(!updated);
        assert_eq!(list.tasks[0].status, Status::Pending);
    }

        // Test 14: Save and load a list of tasks

        #[test]

        fn test_todolist_save_and_load() {

            // Arrange

            let mut list_to_save = ToDoList::default();

            list_to_save.add_task("First".to_string(), Importance::Normal);

            list_to_save.add_task("Second, with comma".to_string(), Importance::Urgent);

    

            let temp_file = tempfile::NamedTempFile::new().unwrap();

            let path_str = temp_file.path().to_str().unwrap();

    

            // Act

            let save_result = list_to_save.save(path_str);

            let loaded_list = ToDoList::load(path_str).unwrap();

    

            // Assert

            assert!(save_result.is_ok());

            assert_eq!(loaded_list.tasks().len(), 2);

            assert_eq!(loaded_list.tasks()[0].id, 1);

            assert_eq!(loaded_list.tasks()[0].name, "First");

            assert_eq!(loaded_list.tasks()[1].id, 2);

            assert_eq!(loaded_list.tasks()[1].name, "Second, with comma");

            assert_eq!(loaded_list.tasks()[1].importance, Importance::Urgent);

        }

    

        // --- NOVOS TESTES ADICIONADOS ---

    

        // Test 15: Parse a task with commas in its name

        #[test]

        fn test_task_from_str_with_commas_in_name() {

            // Arrange

            let line = "10,My task, with, commas,pending,normal";

            

            // Act

            let task = Task::from_str(line).unwrap();

    

            // Assert

            assert_eq!(task.id, 10);

            assert_eq!(task.name, "My task, with, commas");

            assert_eq!(task.status, Status::Pending);

            assert_eq!(task.importance, Importance::Normal);

        }

    

        // Test 16: Failure on parsing a task with an invalid ID

        #[test]

        fn test_task_from_str_invalid_id() {

            // Arrange

            let line = "not-a-number,A task,pending,normal";

    

            // Act

            let result = Task::from_str(line);

    

            // Assert

            assert!(result.is_err());

        }

        

        // Test 17: Update only the importance of a task

        #[test]

        fn test_todolist_update_task_importance_only() {

            // Arrange

            let tasks = vec![Task { id: 1, name: "t1".into(), status: Status::Pending, importance: Importance::Normal }];

            let mut list = ToDoList { tasks };

    

            // Act

            let result = list.update_task(1, None, Some(Importance::Urgent));

    

            // Assert

            assert!(result);

            assert_eq!(list.tasks[0].status, Status::Pending); // Status should not change

            assert_eq!(list.tasks[0].importance, Importance::Urgent);

        }

    

        // Test 18: Add multiple tasks and check ID assignment

        #[test]

        fn test_todolist_add_multiple_tasks() {

            // Arrange

            let mut list = ToDoList::default();

    

            // Act

            let id1 = list.add_task("Task 1".to_string(), Importance::Normal);

            let id2 = list.add_task("Task 2".to_string(), Importance::Important);

            let id3 = list.add_task("Task 3".to_string(), Importance::Urgent);

    

            // Assert

            assert_eq!(id1, 1);

            assert_eq!(id2, 2);

            assert_eq!(id3, 3);

            assert_eq!(list.tasks.len(), 3);

        }

    

        // Test 19: Load from a non-existent file should succeed with an empty list

        #[test]

        fn test_todolist_load_non_existent_file() {

            // Arrange

            let path = "non_existent_file_for_testing.csv";

    

            // Act

            let list = ToDoList::load(path).unwrap();

    

            // Assert

            assert!(list.is_empty());

        }

    

        // Test 20: Load from an empty file should result in an empty list

        #[test]

        fn test_todolist_load_empty_file() {

            // Arrange

            let temp_file = tempfile::NamedTempFile::new().unwrap();

            let path_str = temp_file.path().to_str().unwrap();

    

            // Act

            let list = ToDoList::load(path_str).unwrap();

    

            // Assert

            assert!(list.is_empty());

        }

    

        // Test 21: Load from a file with mixed valid and invalid lines

        #[test]

        fn test_todolist_load_mixed_lines() {

            // Arrange

            let temp_file = tempfile::NamedTempFile::new().unwrap();

            let path_str = temp_file.path().to_str().unwrap();

            let mut file = File::create(path_str).unwrap();

            writeln!(file, "1,Valid task,pending,normal").unwrap();

            writeln!(file, "this is a malformed line").unwrap();

            writeln!(file, "2,Another valid task,finished,urgent").unwrap();

            writeln!(file, "3,Task with,commas,pending,important").unwrap();

    

            // Act

            let list = ToDoList::load(path_str).unwrap();

    

            // Assert

            assert_eq!(list.tasks.len(), 3);

            assert_eq!(list.tasks[0].id, 1);

            assert_eq!(list.tasks[1].id, 2);

            assert_eq!(list.tasks[2].id, 3);

        }

    

        // Test 22: Failure on parsing a task line with insufficient parts

        #[test]

        fn test_task_from_str_insufficient_parts() {

            // Arrange

            let line = "1,missing_status_and_importance";

    

            // Act

            let result = Task::from_str(line);

    

            // Assert

            assert!(result.is_err());

            assert_eq!(result.err().unwrap(), "Incorrect line format: couldn't split into 3 parts from right");

        }

    }

    