use crate::app::Todo;
use std::{fs, io};

pub fn load_todos() -> io::Result<Vec<Todo>> {
    let data = fs::read_to_string("todos.json").unwrap_or_else(|_| "[]".to_string());
    let todos: Vec<Todo> =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(todos)
}

pub fn save_todos(todos: &[Todo]) -> io::Result<()> {
    let data = serde_json::to_string(todos)?;
    fs::write("todos.json", data)
}
