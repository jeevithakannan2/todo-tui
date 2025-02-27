use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io};
use uuid::Uuid;

#[derive(Serialize, Clone, PartialEq, Deserialize)]
pub struct Todo {
    pub id: u128,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

impl Todo {
    pub fn new() -> Self {
        Self {
            id: Uuid::now_v7().as_u128(),
            title: String::new(),
            description: String::new(),
            completed: false,
        }
    }

    pub fn from(
        id: Option<u128>,
        title: Option<String>,
        description: Option<String>,
        completed: Option<bool>,
    ) -> Self {
        Self {
            id: id.unwrap_or_else(|| Uuid::now_v7().as_u128()),
            title: title.unwrap_or_else(|| String::new()),
            description: description.unwrap_or_else(|| String::new()),
            completed: completed.unwrap_or_else(|| false),
        }
    }
}

pub fn load_todos() -> io::Result<Vec<Todo>> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    let data = fs::read_to_string(format!("{}/todos.json", dir.data_dir().to_str().unwrap()))
        .unwrap_or_else(|_| "[]".to_string());
    let todos: Vec<Todo> =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(todos)
}

pub fn save_todos(todos: &[Todo]) -> io::Result<()> {
    let dir = ProjectDirs::from("com", "CodeTrenchers", "TodoTUI").unwrap();
    fs::DirBuilder::new()
        .recursive(true)
        .create(dir.data_dir())
        .unwrap();
    let path = format!("{}/todos.json", dir.data_dir().to_str().unwrap());
    let data = serde_json::to_string(todos)?;
    fs::write(path, data)
}
