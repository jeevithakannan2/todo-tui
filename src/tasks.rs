use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Serialize, Clone, PartialEq, Deserialize)]
pub struct Task {
    pub id: u128,
    pub title: String,
    pub date: String,
    pub time: String,
    pub description: String,
    pub status: TaskStatus,
}

#[derive(Serialize, Clone, PartialEq, Deserialize)]
pub enum TaskStatus {
    Completed,
    Pending,
    OverDue,
}

impl Task {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::now_v7().as_u128(),
            title: String::new(),
            date: String::new(),
            time: String::new(),
            description: String::new(),
            status: TaskStatus::Pending,
        }
    }

    pub fn from(id: u128) -> Self {
        Self {
            id,
            title: String::new(),
            date: String::new(),
            time: String::new(),
            description: String::new(),
            status: TaskStatus::Pending,
        }
    }

    pub fn is_overdue(&self) -> bool {
        let now = chrono::Local::now().naive_local();
        let date_time_str = format!("{} {}", self.time, self.date);
        let date_time = NaiveDateTime::parse_from_str(&date_time_str, "%H %M %d %m %Y").unwrap();
        self.status == TaskStatus::OverDue || self.status == TaskStatus::Pending && date_time < now
    }
}


pub fn update_overdue(tasks: &mut [Task]) {
    for task in tasks.iter_mut() {
        if task.is_overdue() {
            task.status = TaskStatus::OverDue;
        }
    }
}

fn get_data_dir() -> PathBuf {
    crate::helpers::get_project_dir().data_dir().to_path_buf()
}

pub fn reset() -> io::Result<()> {
    let dir = get_data_dir();
    let _ = fs::remove_file(dir.join("tasks.enc"));
    Ok(())
}

pub fn load() -> io::Result<Vec<Task>> {
    let dir = get_data_dir();
    let data = fs::read_to_string(dir.join("tasks.json"));
    match data {
        Ok(tasks) => Ok(serde_json::from_str(&tasks)?),
        Err(_) => Ok(Vec::new()),
    }
}

pub fn save(tasks: &[Task]) -> io::Result<()> {
    let dir = get_data_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join("tasks.json");
    let data = serde_json::to_string_pretty(tasks)?;
    fs::write(path, data)
}

pub fn load_encrypted() -> io::Result<Vec<Task>> {
    let dir = get_data_dir();
    let data = fs::read(dir.join("tasks.enc"));
    match data {
        Ok(tasks) => Ok(crate::auth::decrypt_tasks(&tasks)),
        Err(_) => Ok(Vec::new()),
    }
}

pub fn save_encrypted(tasks: &[Task]) -> io::Result<()> {
    let dir = get_data_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join("tasks.enc");
    let data = crate::auth::encrypt_tasks(&tasks);
    fs::write(path, data)
}
