use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io};

#[cfg(feature = "encryption")]
use crate::auth::get_password;

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
    #[cfg(feature = "encryption")]
    let id = load_tasks_encrypted().unwrap().len() as u16;
    #[cfg(not(feature = "encryption"))]
    let id = load_tasks().unwrap().len() as u16;
    id
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
    let password = get_password();
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    let data = fs::read(format!("{}/tasks", dir.data_dir().to_str().unwrap()));
    match data {
        Ok(tasks) => Ok(crate::auth::decrypt_tasks(&tasks, &password)),
        Err(_) => Ok(Vec::new()),
    }
}

#[cfg(feature = "encryption")]
pub fn save_tasks_ecrypted(tasks: &[Task]) -> io::Result<()> {
    let password = get_password();
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    fs::DirBuilder::new()
        .recursive(true)
        .create(dir.data_dir())
        .unwrap();
    let path = format!("{}/tasks", dir.data_dir().to_str().unwrap());
    let data = crate::auth::encrypt_tasks(&tasks, &password);
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
