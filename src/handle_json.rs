use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io};

#[derive(Serialize, Clone, PartialEq, Deserialize)]
pub struct Task {
    pub id: u16,
    pub title: String,
    pub date: String,
    pub description: String,
    pub completed: bool,
}

impl Task {
    pub fn new() -> Self {
        Self {
            id: get_id(),
            title: String::new(),
            date: String::new(),
            description: String::new(),
            completed: false,
        }
    }

    pub fn from(id: u16) -> Self {
        Self {
            id,
            title: String::new(),
            date: String::new(),
            description: String::new(),
            completed: false,
        }
    }
}

fn get_id() -> u16 {
    load_tasks().unwrap().len() as u16
}

pub fn load_tasks() -> io::Result<Vec<Task>> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    let data = fs::read_to_string(format!("{}/tasks.json", dir.data_dir().to_str().unwrap()))
        .unwrap_or_else(|_| "[]".to_string());
    let tasks: Vec<Task> =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(tasks)
}

pub fn save_tasks(tasks: &[Task]) -> io::Result<()> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    fs::DirBuilder::new()
        .recursive(true)
        .create(dir.data_dir())
        .unwrap();
    let path = format!("{}/tasks.json", dir.data_dir().to_str().unwrap());
    let data = serde_json::to_string(tasks)?;
    fs::write(path, data)
}
