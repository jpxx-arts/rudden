use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::task::{Task, Importance, Status};

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    last_id: u32,
}

fn read_metadata(meta_path: &Path) -> io::Result<Metadata> {
    if !meta_path.exists() {
        return Ok(Metadata { last_id: 0 });
    }
    let file = File::open(meta_path)?;
    let meta: Metadata = serde_json::from_reader(file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(meta)
}

fn write_metadata(meta_path: &Path, meta: &Metadata) -> io::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(meta_path)?;
    serde_json::to_writer_pretty(file, meta)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(())
}

fn get_next_id(meta_path: &Path) -> io::Result<u32> {
    let mut meta = read_metadata(meta_path)?;
    meta.last_id += 1;
    write_metadata(meta_path, &meta)?;
    Ok(meta.last_id)
}

/// Adds a task using an append-only strategy, which is O(1).
pub fn add_task_fast(data_path: &Path, meta_path: &Path, name: String, importance: Importance) -> io::Result<()> {
    let mut file = OpenOptions::new().append(true).create(true).open(data_path)?;
    
    let new_id = get_next_id(meta_path)?;

    let new_task = Task {
        id: new_id,
        name,
        status: Status::Pending,
        importance,
    };

    // This is a simplified `to_csv_line` for direct use.
    let line = format!(
        "{},{},{},{}",
        new_task.id, new_task.name, new_task.status, new_task.importance
    );

    writeln!(file, "{}", line)?;

    Ok(())
}

/// Clears the data files used by the benchmark.
pub fn clear_benchmark_data(slow_path: &Path, fast_path: &Path, meta_path: &Path) -> io::Result<()> {
    if slow_path.exists() {
        std::fs::remove_file(slow_path)?;
    }
    if fast_path.exists() {
        std::fs::remove_file(fast_path)?;
    }
    if meta_path.exists() {
        std::fs::remove_file(meta_path)?;
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_read_metadata_no_file() -> io::Result<()> {
        let dir = tempdir()?;
        let meta_path = dir.path().join("meta.json");
        
        // Arrange & Act
        let meta = read_metadata(&meta_path)?;

        // Assert
        assert_eq!(meta.last_id, 0);
        Ok(())
    }

    #[test]
    fn test_write_and_read_metadata() -> io::Result<()> {
        let dir = tempdir()?;
        let meta_path = dir.path().join("meta.json");

        // Arrange
        let meta_to_write = Metadata { last_id: 42 };

        // Act
        write_metadata(&meta_path, &meta_to_write)?;
        let meta_read = read_metadata(&meta_path)?;

        // Assert
        assert_eq!(meta_read.last_id, 42);
        Ok(())
    }

    #[test]
    fn test_get_next_id_sequentially() -> io::Result<()> {
        let dir = tempdir()?;
        let meta_path = dir.path().join("meta.json");

        // Arrange, Act, Assert
        assert_eq!(get_next_id(&meta_path)?, 1);
        assert_eq!(get_next_id(&meta_path)?, 2);
        assert_eq!(get_next_id(&meta_path)?, 3);

        let meta = read_metadata(&meta_path)?;
        assert_eq!(meta.last_id, 3);
        Ok(())
    }

    #[test]
    fn test_add_task_fast_appends_correctly() -> io::Result<()> {
        let dir = tempdir()?;
        let data_path = dir.path().join("fast.csv");
        let meta_path = dir.path().join("meta.json");

        // Act
        add_task_fast(&data_path, &meta_path, "First task".to_string(), Importance::Urgent)?;
        add_task_fast(&data_path, &meta_path, "Second task, with comma".to_string(), Importance::Normal)?;

        // Assert
        let content = fs::read_to_string(&data_path)?;
        let lines: Vec<&str> = content.trim().split('\n').collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "1,First task,pending,urgent");
        assert_eq!(lines[1], "2,Second task, with comma,pending,normal");
        Ok(())
    }

    #[test]
    fn test_clear_benchmark_data_removes_files() -> io::Result<()> {
        let dir = tempdir()?;
        let slow_path = dir.path().join("slow.csv");
        let fast_path = dir.path().join("fast.csv");
        let meta_path = dir.path().join("meta.json");

        // Arrange
        fs::write(&slow_path, "data")?;
        fs::write(&fast_path, "data")?;
        write_metadata(&meta_path, &Metadata { last_id: 1 })?;

        // Act
        clear_benchmark_data(&slow_path, &fast_path, &meta_path)?;

        // Assert
        assert!(!slow_path.exists());
        assert!(!fast_path.exists());
        assert!(!meta_path.exists());
        Ok(())
    }
}
