use crate::{
    handle_json::{load_todos, save_todos, Todo},
    helpers::{create_popup_area, PopupSize},
    new_task,
    theme::Theme,
};
use chrono::NaiveDate;
use new_task::NewTask;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    style::palette::tailwind::{GREEN, RED},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
};
use std::collections::BTreeMap;

pub struct App<'a> {
    // Used to determine which part of the app is currently in focus
    focus: AppFocus,
    // The last selected index of the todo list before clearing the selection
    last_selected: Option<usize>,
    // Enable preview mode
    preview: bool,
    // The state of the new task popup
    new_task: NewTask<'a>,
    // The state of the todo list
    selected: ListState,
    // Selected theme
    theme: Theme,
    // The list of parsed todos from json
    todos: Vec<Todo>,
    // Preview scroll state
    preview_scroll: (u16, u16),
    preview_length: u16,
    preview_height: u16,
    // Indexes of todos that can be selected ( This eliminates the date headers )
    selectable: Vec<(usize, u128)>,
    // Total number of items in the list
    total: usize,
}

#[derive(PartialEq)]
enum AppFocus {
    NewTask,
    Preview,
    TodoList,
    DeletePrompt,
}

pub const PRIMARY_STYLE: Style = Style::new().fg(Color::Green);
pub const SECONDARY_STYLE: Style = Style::new().fg(Color::Blue);
pub const GREEN_STYLE: Style = Style::new().fg(GREEN.c500);
pub const RED_STYLE: Style = Style::new().fg(RED.c500);

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let footer_text: Line = match self.focus {
            AppFocus::TodoList => {
                let arrows = if self.theme == Theme::Default {
                    " "
                } else {
                    "Up/Down"
                };
                format!(" [{}] Navigate | [q] Quit | [e] Edit Task | [p] Preview Tab | [n] New Task | [d] Delete Task | [Space] Toggle Task ", arrows).into()
            }

            AppFocus::NewTask => self.new_task.footer_text().into(),
            AppFocus::DeletePrompt => "[y] Yes | [n] No".into(),
            AppFocus::Preview => {
                let arrows = if self.theme == Theme::Default {
                    " "
                } else {
                    "Up/Down"
                };
                format!(" [{}] Navigate", arrows).into()
            }
        };

        let footer_height = (1 + footer_text.width().try_into().unwrap_or(0) / area.width).min(3);

        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(footer_height)]).areas(area);

        let [list_area, preview_area] = if self.preview {
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(main_area)
        } else {
            Layout::horizontal([Constraint::Percentage(100), Constraint::Percentage(0)])
                .areas(main_area)
        };

        self.render_footer(footer_area, buf, footer_text);
        self.render_list(list_area, buf);
        self.render_preview(preview_area, buf);

        match self.focus {
            AppFocus::NewTask => {
                let new_task_area =
                    create_popup_area(main_area, PopupSize::Percentage { x: 75, y: 75 });
                self.new_task.render(new_task_area, buf);
            }
            AppFocus::DeletePrompt => {
                crate::confirm::Confirm::new(
                    " Delete Task ".into(),
                    "Are you sure you want to delete the selected task(s)?".into(),
                )
                .render(area, buf);
            }
            AppFocus::Preview => {}
            AppFocus::TodoList => {}
        }
    }
}

impl App<'_> {
    pub fn new() -> Self {
        let todos = load_todos().unwrap_or_else(|_| Vec::new());

        Self {
            focus: AppFocus::TodoList,
            last_selected: None,
            preview: false,
            theme: Theme::Default,
            new_task: NewTask::new(),
            selected: ListState::default(),
            todos,
            preview_scroll: (0, 0),
            preview_length: 0,
            preview_height: 0,
            selectable: Vec::new(),
            total: 0,
        }
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer, line: Line) {
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let style = match self.focus {
            AppFocus::TodoList => PRIMARY_STYLE,
            _ => SECONDARY_STYLE,
        };
        let block = crate::helpers::rounded_block(" Tasks ", style);

        let mut grouped_tasks: BTreeMap<NaiveDate, Vec<&Todo>> = BTreeMap::new();
        for todo in &self.todos {
            let date = NaiveDate::parse_from_str(&todo.date, "%Y-%m-%d").unwrap();
            grouped_tasks.entry(date).or_default().push(todo);
        }

        let mut items: Vec<ListItem> = Vec::new();
        self.selectable.clear();

        let today = chrono::Local::now().date_naive();

        for (date, tasks) in grouped_tasks {
            // Format the date header based on its relation to today
            let date_header = if date == today {
                "Today".to_string()
            } else if date == today.succ_opt().unwrap_or(today) {
                "Tomorrow".to_string()
            } else {
                format!("{} {}", date.format("%a"), date.format("%b %d %Y")) // Show day name with month, date, year
            };

            // Add a date header (Non-selectable)
            items.push(
                ListItem::new(format!(" {}", date_header))
                    .style(Style::default().blue().add_modifier(Modifier::BOLD)),
            );

            // Add tasks under the date
            for todo in tasks {
                let index = items.len(); // Get current index before pushing task
                let title = todo.title.as_str();
                let (icon, style) = if todo.completed {
                    (self.theme.get_completed(), Style::default().dark_gray())
                } else {
                    (self.theme.get_uncompleted(), Style::default().bold())
                };
                let item = ListItem::new(format!(" {} {}", icon, title)).style(style);
                self.selectable.push((index, todo.id)); // Track only tasks as selectable
                items.push(item);
            }
            items.push(ListItem::new("")); // Adds a new line after each date group
        }

        self.total = items.len();
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().fg(Color::Yellow));

        StatefulWidget::render(list, area, buf, &mut self.selected);
    }

    fn render_preview(&mut self, area: Rect, buf: &mut Buffer) {
        let style = match self.focus {
            AppFocus::TodoList => SECONDARY_STYLE,
            _ => PRIMARY_STYLE,
        };
        let block = crate::helpers::rounded_block(" Preview ", style);
        let temp_todo = Todo {
            id: 0,
            title: String::new(),
            date: String::new(),
            description: String::from("No task selected"),
            completed: false,
        };
        let todo = self.get_selected().unwrap_or(temp_todo);
        let preview = todo.description.lines().count() as u16;
        self.preview_length = preview;
        self.preview_height = area.height;

        Paragraph::new(todo.description.as_str())
            .block(block)
            .scroll(self.preview_scroll)
            .wrap(Wrap { trim: true })
            .render(area, buf)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.focus {
            AppFocus::TodoList => match key.code {
                KeyCode::BackTab | KeyCode::Tab => {
                    if self.preview {
                        self.focus = AppFocus::Preview;
                    }
                }
                KeyCode::Down => self.select_next(),
                KeyCode::Up => self.select_previous(),
                KeyCode::Esc => self.select_none(),
                KeyCode::Char(' ') => self.toggle_completed(),
                KeyCode::Char('t') => {
                    self.theme = self.theme.change_theme();
                }
                KeyCode::Char('q') => {
                    save_todos(&self.todos).unwrap();
                    return true;
                }
                KeyCode::Char('d') => self.focus = AppFocus::DeletePrompt,
                KeyCode::Char('e') => {
                    if let Some(todo) = self.get_selected() {
                        self.new_task = NewTask::from(todo);
                        self.focus = AppFocus::NewTask;
                    }
                }
                KeyCode::Char('n') => {
                    self.new_task = NewTask::new();
                    self.new_task.quit = false;
                    self.focus = AppFocus::NewTask;
                }
                KeyCode::Char('p') => self.preview = !self.preview,
                _ => {}
            },
            AppFocus::NewTask => {
                self.new_task.handle_key(key);
                if self.new_task.quit {
                    if self.new_task.completed {
                        let todo = self.new_task.todo.clone();
                        if let Some(index) = self.selected.selected() {
                            if let Some((_, id)) = self.selectable.iter().find(|(i, _)| *i == index)
                            {
                                if let Some(existing_todo) =
                                    self.todos.iter_mut().find(|t| t.id == *id)
                                {
                                    *existing_todo = todo;
                                }
                            }
                        } else {
                            self.todos.push(todo);
                        }
                        save_todos(&self.todos).unwrap();
                        self.new_task = NewTask::new();
                    }
                    self.focus = AppFocus::TodoList;
                }
            }
            AppFocus::Preview => match key.code {
                KeyCode::BackTab | KeyCode::Tab => self.focus = AppFocus::TodoList,
                KeyCode::Down => self.scroll_preview_down(),
                KeyCode::Up => self.scroll_preview_up(),
                _ => {}
            },
            AppFocus::DeletePrompt => match key.code {
                KeyCode::Char('y') => {
                    self.delete_entry();
                    self.focus = AppFocus::TodoList;
                }
                KeyCode::Char('n') => self.focus = AppFocus::TodoList,
                _ => {}
            },
        }
        false
    }

    fn scroll_preview_up(&mut self) {
        self.preview_scroll.0 = self.preview_scroll.0.saturating_sub(1);
    }

    fn scroll_preview_down(&mut self) {
        if self.preview_length - 1 > self.preview_scroll.0 + self.preview_height {
            self.preview_scroll.0 += 1;
        }
    }

    fn get_selected(&self) -> Option<Todo> {
        self.selected
            .selected()
            .and_then(|index| self.selectable.iter().find(|(i, _)| *i == index))
            .and_then(|(_, id)| self.todos.iter().find(|t| t.id == *id))
            .cloned()
    }

    fn get_selected_mut(&mut self) -> Option<&mut Todo> {
        self.selected
            .selected()
            .and_then(|index| self.selectable.iter().find(|(i, _)| *i == index))
            .and_then(|(_, id)| self.todos.iter_mut().find(|t| t.id == *id))
    }

    fn select_last_selected(&mut self) {
        if let Some(index) = self.last_selected.take() {
            self.selected.select(Some(index));
        }
    }

    fn select_next(&mut self) {
        if self.selectable.is_empty() {
            return;
        }
        self.select_last_selected();
        let index = match self.selected.selected() {
            Some(index) => index,
            None => 0,
        };

        let mut next = index + 1;
        while !self.selectable.iter().any(|&(i, _)| i == next) {
            next += 1;
            if next >= self.total {
                next = self.selectable[0].0;
            }
        }
        self.selected.select(Some(next));
    }

    fn select_previous(&mut self) {
        self.select_last_selected();
        if self.selectable.is_empty() {
            return;
        }
        let index: usize = match self.selected.selected() {
            Some(index) => index,
            None => 0,
        };

        let mut next = index.saturating_sub(1);
        while !self.selectable.iter().any(|&(i, _)| i == next) {
            next = next.saturating_sub(1);
            if next == 0 {
                next = self.selectable.last().copied().unwrap_or((0, 0)).0;
            }
        }
        self.selected.select(Some(next));
    }

    fn select_none(&mut self) {
        self.last_selected = self.selected.selected();
        self.selected.select(None);
    }

    fn toggle_completed(&mut self) {
        if let Some(todo) = self.get_selected_mut() {
            todo.completed = !todo.completed;
        }
    }

    fn delete_entry(&mut self) {
        if let Some(todo) = self.get_selected() {
            self.todos.retain(|t| t.id != todo.id);
        }
    }
}
