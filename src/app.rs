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

use crate::{
    handle_json::{load_todos, save_todos},
    new_task,
};

use new_task::NewTask;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Todo {
    pub title: String,
    pub description: String,
    pub completed: bool,
}

pub struct App<'a> {
    todos: Vec<Todo>,
    pub focus: AppFocus,
    new_task: NewTask<'a>,
    selected: ListState,
    last_selected: Option<usize>,
}

#[derive(PartialEq)]
pub enum AppFocus {
    NewTask,
    TodoList,
}

pub const SECONDARY_STYLE: Style = Style::new().fg(Color::LightBlue);

impl App<'_> {
    pub fn new() -> Self {
        let todos = load_todos().unwrap_or_else(|_| Vec::new());
        Self {
            todos,
            focus: AppFocus::TodoList,
            selected: ListState::default(),
            new_task: NewTask::new(),
            last_selected: None,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let [hero_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let style = if self.focus != AppFocus::NewTask {
            SECONDARY_STYLE
        } else {
            Style::default()
        };

        let [list_area, preview_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(hero_area);

        let list_block = Block::bordered()
            .title(" Todo List ")
            .title_alignment(Alignment::Center)
            .title_style(Style::default().reset().bold())
            .borders(Borders::BOTTOM | Borders::TOP | Borders::LEFT)
            .border_type(BorderType::Rounded)
            .border_style(style);

        let items: Vec<ListItem> = self
            .todos
            .iter()
            .map(|todo| {
                if todo.completed {
                    ListItem::new(format!("  {}", todo.title.as_str()))
                        .style(Style::default().fg(GREEN.c500))
                } else {
                    ListItem::new(format!("  {}", todo.title.as_str()))
                }
            })
            .collect();
        let list = List::new(items)
            .highlight_symbol("")
            .block(list_block)
            .highlight_spacing(HighlightSpacing::WhenSelected);

        frame.render_stateful_widget(list, list_area, &mut self.selected);

        let preview_block = Block::bordered()
            .title(" Preview ")
            .title_alignment(Alignment::Center)
            .title_style(Style::default().reset().bold())
            .borders(Borders::BOTTOM | Borders::TOP | Borders::RIGHT)
            .border_type(BorderType::Rounded)
            .border_style(style);

        frame.render_widget(&preview_block, preview_area);

        // Sometimes the ratatui list selection goes todos.len() + 1 so we need to clamp it
        let todo = match self.selected.selected() {
            Some(index) => match self
                .todos
                .get(index.min(self.todos.len().saturating_sub(1)))
            {
                Some(todo) => todo,
                None => &Todo {
                    title: String::new(),
                    description: "No tasks selected".to_string(),
                    completed: false,
                },
            },
            None => &Todo {
                title: String::new(),
                description: "No tasks selected".to_string(),
                completed: false,
            },
        };

        // `preview_inner_block` is used to print the separation line between the list and the preview
        let preview_inner_block = Block::bordered().borders(Borders::LEFT).border_style(style);
        let description = Paragraph::new(todo.description.as_str())
            .style(Style::reset())
            .block(preview_inner_block)
            .wrap(Wrap { trim: true });

        frame.render_widget(description, preview_block.inner(preview_area));

        let footer =
            Paragraph::new(" [q] Quit | [n] New Task | [Enter] Open Task | [Space] Toggle Task ")
                .style(Style::default())
                .alignment(Alignment::Center)
                .block(Block::default());

        frame.render_widget(footer, footer_area);

        if self.focus == AppFocus::NewTask {
            let new_task_area = popup_area(hero_area, 75, 75);
            self.new_task.draw(frame, new_task_area);
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
        match self.focus {
            AppFocus::TodoList => match key.code {
                KeyCode::Down => self.select_next(),
                KeyCode::Up => self.select_previous(),
                KeyCode::Esc => self.select_none(),
                KeyCode::Char(' ') => self.toggle_completed(),
                KeyCode::Char('q') => {
                    save_todos(&self.todos).unwrap();
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
                        save_todos(&self.todos).unwrap();
                        self.new_task = NewTask::new();
                    }
                    self.focus = AppFocus::TodoList;
                }
            }
        }
        false
    }

    fn select_next(&mut self) {
        if let Some(index) = self.last_selected {
            self.selected.select(Some(index));
            self.last_selected = None;
            return;
        }

        self.selected.select_next();
    }

    fn select_previous(&mut self) {
        if let Some(index) = self.last_selected {
            self.selected.select(Some(index));
            self.last_selected = None;
            return;
        }
        self.selected.select_previous();
    }

    fn select_none(&mut self) {
        self.last_selected = self.selected.selected();
        self.selected.select(None);
    }

    fn toggle_completed(&mut self) {
        let index = match self.selected.selected() {
            Some(index) => index,
            None => return,
        };
        let todo = self.todos.get(index).unwrap();
        let mut todos = self.todos.clone();
        todos[index] = Todo {
            completed: !todo.completed,
            ..todo.clone()
        };
        self.todos = todos;
    }
}

pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
