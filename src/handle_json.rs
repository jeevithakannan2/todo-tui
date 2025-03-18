use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io};

#[derive(Serialize, Clone, PartialEq, Deserialize)]
pub struct Task {
    pub id: u128,
    pub title: String,
    pub date: String,
    pub description: String,
    pub completed: bool,
}

impl Task {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::now_v7().as_u128(),
            title: String::new(),
            date: String::new(),
            description: String::new(),
            completed: false,
        }
    }

    pub fn from(id: u128) -> Self {
        Self {
            id,
            title: String::new(),
            date: String::new(),
            description: String::new(),
            completed: false,
        }
    }
}

#[cfg(feature = "encryption")]
pub fn reset() -> io::Result<()> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    let _ = fs::remove_file(format!("{}/tasks", dir.data_dir().to_str().unwrap()));
    Ok(())
}

#[cfg(not(feature = "encryption"))]
pub fn load_tasks() -> io::Result<Vec<Task>> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    let data = fs::read_to_string(format!("{}/tasks.json", dir.data_dir().to_str().unwrap()));
    match data {
        Ok(tasks) => Ok(serde_json::from_str(&tasks)?),
        Err(_) => Ok(Vec::new()),
    }
}

#[cfg(feature = "encryption")]
pub fn load_tasks_encrypted() -> io::Result<Vec<Task>> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    let data = fs::read(format!("{}/tasks", dir.data_dir().to_str().unwrap()));
    match data {
        Ok(tasks) => Ok(crate::auth::decrypt_tasks(&tasks)),
        Err(_) => Ok(Vec::new()),
    }
}

#[cfg(feature = "encryption")]
pub fn save_tasks_ecrypted(tasks: &[Task]) -> io::Result<()> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    fs::DirBuilder::new()
        .recursive(true)
        .create(dir.data_dir())
        .unwrap();
    let path = format!("{}/tasks", dir.data_dir().to_str().unwrap());
    let data = crate::auth::encrypt_tasks(&tasks);
    fs::write(path, data)
}

#[cfg(not(feature = "encryption"))]
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
