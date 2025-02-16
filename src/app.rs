use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Flex,
    prelude::*,
    style::palette::tailwind::GREEN,
    widgets::{
        Block, BorderType, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Paragraph,
        Wrap,
    },
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
    TaskView,
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

    // Render the UI.
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        match self.focus {
            AppFocus::NewTask => {
                let new_task_area = popup_area(main_area, 75, 75);
                self.new_task.draw(frame, new_task_area);
            }

            AppFocus::TaskView => {
                let task_view_area = popup_area(main_area, 75, 75);
                frame.render_widget(Clear, task_view_area);
                let todo = self.todos.get(self.selected.selected().unwrap()).unwrap();
                let title = format!(" {} ", todo.title.clone());
                let description = todo.description.clone();
                let task_view = Block::bordered()
                    .title(Line::from(title).centered().reset_style())
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::LightBlue));
                frame.render_widget(&task_view, task_view_area);
                let descrition_area = task_view.inner(task_view_area);
                let description = Paragraph::new(description)
                    .style(Style::reset())
                    .wrap(Wrap { trim: true });
                frame.render_widget(description, descrition_area);
            }
            _ => {}
        }

        let style = if self.focus == AppFocus::TodoList {
            Style::default().light_blue()
        } else {
            Style::default()
        };

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
            .block(
                Block::default()
                    .border_type(BorderType::Rounded)
                    .title(Line::from(" To-Do List ").bold().centered())
                    .border_style(style)
                    .borders(Borders::all()),
            )
            .highlight_style(Style::default().reset().reversed())
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, main_area, &mut self.selected);

        let footer =
            Paragraph::new(" [q] Quit | [n] New Task | [Enter] Open Task | [Space] Toggle Task ")
                .style(Style::default().fg(Color::LightBlue))
                .alignment(Alignment::Center)
                .block(Block::default());

        frame.render_widget(footer, footer_area);
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        match self.focus {
            AppFocus::TaskView => {
                if key.code == KeyCode::Char('q') {
                    self.focus = AppFocus::TodoList;
                }
            }
            AppFocus::TodoList => match key.code {
                KeyCode::Down => self.select_next(),
                KeyCode::Up => self.select_previous(),
                KeyCode::Enter => self.open_selected(),
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

    fn open_selected(&mut self) {
        self.focus = AppFocus::TaskView;
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
