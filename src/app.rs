use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Flex,
    prelude::*,
    widgets::{Block, BorderType, List, ListItem},
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io;

use crate::new_task;
use new_task::NewTask;

// Define your task type.
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Todo {
    pub title: String,
    pub description: String,
}

fn load_todos() -> io::Result<Vec<Todo>> {
    let data = fs::read_to_string("todos.json").unwrap_or_else(|_| "[]".to_string());
    let todos: Vec<Todo> =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(todos)
}

pub struct App<'a> {
    todos: Vec<Todo>,
    pub focus: AppFocus,
    new_task: NewTask<'a>,
}

#[derive(PartialEq)]
pub enum AppFocus {
    NewTask,
    TodoList,
}

impl App<'_> {
    pub fn new() -> Self {
        let todos = load_todos().unwrap_or_else(|_| Vec::new());
        Self {
            todos,
            focus: AppFocus::TodoList,
            new_task: NewTask::new(),
        }
    }

    // Render the UI.
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if self.focus == AppFocus::NewTask {
            let new_task_area = popup_area(area, 75, 75);
            self.new_task.draw(frame, new_task_area);
        }

        let style = if self.focus == AppFocus::TodoList {
            Style::default().light_blue()
        } else {
            Style::default()
        };

        let items: Vec<ListItem> = self
            .todos
            .iter()
            .map(|todo| ListItem::new(todo.title.clone()))
            .collect();
        let list = List::new(items).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title(Line::from(" To-Do List ").bold().centered())
                .border_style(style),
        );
        frame.render_widget(list, area);
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        match self.focus {
            AppFocus::TodoList => match key.code {
                KeyCode::Char('q') => return true,
                KeyCode::Char('n') => {
                    self.new_task = NewTask::new();
                    self.focus = AppFocus::NewTask;
                }
                _ => {}
            },
            AppFocus::NewTask => {
                self.new_task.on_key(key);
                if self.new_task.completed {
                    let todo = self.new_task.task.todo.clone();
                    self.todos.push(todo);
                    self.focus = AppFocus::TodoList;
                    self.save_todos().unwrap();
                    self.new_task = NewTask::new();
                }
            }
        }
        false
    }

    fn save_todos(&self) -> io::Result<()> {
        let data = serde_json::to_string(&self.todos)?;
        fs::write("todos.json", data)
    }
}

pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
