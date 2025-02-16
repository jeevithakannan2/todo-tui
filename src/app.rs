use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Flex,
    prelude::*,
    style::palette::tailwind::GREEN,
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, Wrap,
    },
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io;

use crate::new_task;
use new_task::NewTask;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Todo {
    pub title: String,
    pub description: String,
    pub completed: bool,
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
    selected: ListState,
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
            selected: ListState::default().with_selected(Some(0)),
            new_task: NewTask::new(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let style = if self.focus != AppFocus::NewTask {
            Style::default().light_blue()
        } else {
            Style::default()
        };

        let border_block = Block::default()
            .border_type(BorderType::Rounded)
            .title(Line::from(" To-Do List ").bold().centered())
            .border_style(style)
            .borders(Borders::all());

        frame.render_widget(&border_block, main_area);

        let mut list_area = border_block.inner(main_area);

        let [area_left, area_right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(list_area);
        list_area = area_left;
        let todo = match self.todos.get(self.selected.selected().unwrap_or(0)) {
            Some(todo) => todo,
            None => &Todo {
                title: "No tasks".to_string(),
                description: "No tasks".to_string(),
                completed: false,
            },
        };

        let description = todo.description.clone();
        let description_block = Block::bordered().borders(Borders::LEFT).border_style(style);
        let description = Paragraph::new(description)
            .style(Style::reset())
            .block(description_block)
            .wrap(Wrap { trim: true });

        frame.render_widget(description, area_right);

        match self.focus {
            AppFocus::NewTask => {
                let new_task_area = popup_area(main_area, 75, 75);
                self.new_task.draw(frame, new_task_area);
            }
            _ => {}
        }

        let items: Vec<ListItem> = self
            .todos
            .iter()
            .map(|todo| {
                if todo.completed {
                    ListItem::new(format!("  {}", todo.title.clone()))
                        .style(Style::default().fg(GREEN.c500))
                } else {
                    ListItem::new(format!("  {}", todo.title.clone()))
                }
            })
            .collect();
        let list = List::new(items)
            .highlight_symbol("")
            .highlight_spacing(HighlightSpacing::WhenSelected);

        frame.render_stateful_widget(list, list_area, &mut self.selected);

        let footer =
            Paragraph::new(" [q] Quit | [n] New Task | [Enter] Open Task | [Space] Toggle Task ")
                .style(Style::default())
                .alignment(Alignment::Center)
                .block(Block::default());

        frame.render_widget(footer, footer_area);
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        match self.focus {
            AppFocus::TodoList => match key.code {
                KeyCode::Down => self.select_next(),
                KeyCode::Up => self.select_previous(),
                KeyCode::Char(' ') => self.toggle_completed(),
                KeyCode::Char('q') => {
                    self.save_todos().unwrap();
                    return true;
                }
                KeyCode::Char('n') => {
                    self.new_task.completed = false;
                    self.new_task.quit = false;
                    self.focus = AppFocus::NewTask;
                }
                _ => {}
            },
            AppFocus::NewTask => {
                self.new_task.on_key(key);
                if self.new_task.quit {
                    if self.new_task.completed {
                        let todo = self.new_task.task.todo.clone();
                        self.todos.push(todo);
                        self.save_todos().unwrap();
                    }
                    self.focus = AppFocus::TodoList;
                }
            }
        }
        false
    }

    fn select_next(&mut self) {
        self.selected.select_next();
    }

    fn select_previous(&mut self) {
        self.selected.select_previous();
    }

    fn toggle_completed(&mut self) {
        let index = self.selected.selected().unwrap();
        let todo = self.todos.get(index).unwrap();
        let mut todos = self.todos.clone();
        todos[index] = Todo {
            completed: !todo.completed,
            ..todo.clone()
        };
        self.todos = todos;
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
