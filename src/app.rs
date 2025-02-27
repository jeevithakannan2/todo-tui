use crate::{
    handle_json::{load_todos, save_todos, Todo},
    new_task,
    // settings::{self, load_settings, save_settings, Mode, Settings, SettingsState},
    settings::{load_settings, save_settings, NewSettings, Settings},
};
use new_task::NewTask;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    style::palette::tailwind::{GREEN, RED},
    widgets::{HighlightSpacing, List, ListItem, ListState, Paragraph, Wrap},
};

pub struct App<'a> {
    // Used to determine which part of the app is currently in focus
    focus: AppFocus,
    // The last selected index of the todo list before clearing the selection
    last_selected: Option<usize>,
    // Enable preview mode
    preview: bool,
    // Multi select mode
    multi_select: bool,
    // The state of the new task popup
    new_task: NewTask<'a>,
    // The state of the todo list
    selected: ListState,
    // Selected entries
    selected_entries: Vec<Todo>,
    // New Settings
    new_settings: NewSettings<'a>,
    // Settings
    settings: Settings,
    // The list of parsed todos from json
    todos: Vec<Todo>,
}

#[derive(PartialEq)]
enum AppFocus {
    NewTask,
    TodoList,
    Settings,
    DeletePrompt,
}

pub const SECONDARY_STYLE: Style = Style::new().fg(Color::Blue);
pub const GREEN_STYLE: Style = Style::new().fg(GREEN.c500);
pub const RED_STYLE: Style = Style::new().fg(RED.c500);

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let footer_text: Line = match self.focus {
            AppFocus::TodoList => " [ ] Navigate | [q] Quit | [e] Edit Task | [p] Preview Tab | [n] New Task | [d] Delete Task | [v] Multi select | [Space] Toggle Task ".into(),
            AppFocus::NewTask => self.new_task.footer_text().into(),
            AppFocus::DeletePrompt => "[y] Yes | [n] No".into(),
            AppFocus::Settings => self.new_settings.footer_text().into(),
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
                let new_task_area = crate::helpers::popup_area(main_area, 75, 75);
                self.new_task.render(new_task_area, buf);
            }
            AppFocus::DeletePrompt => {
                crate::confirm::Confirm::new(
                    " Delete Task ".into(),
                    "Are you sure you want to delete the selected task(s)?".into(),
                )
                .render(area, buf);
            }
            AppFocus::Settings => {
                let settings_area = crate::helpers::popup_fixed_height(main_area, 75, 8);
                self.new_settings.render(settings_area, buf);
            }
            _ => {}
        }
    }
}

impl App<'_> {
    pub fn new() -> Self {
        let todos = load_todos().unwrap_or_else(|_| Vec::new());
        let settings = load_settings().unwrap_or_else(|_| Settings::default());

        Self {
            focus: AppFocus::TodoList,
            last_selected: None,
            preview: false,
            new_settings: NewSettings::from(&settings),
            settings,
            multi_select: false,
            new_task: NewTask::new(),
            selected: ListState::default(),
            selected_entries: Vec::new(),
            todos,
        }
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer, line: Line) {
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = crate::helpers::rounded_block(" Tasks ");
        let items: Vec<ListItem> = self
            .todos
            .iter()
            .map(|todo| {
                if self.selected_entries.contains(&todo) {
                    ListItem::new(format!("  {}", todo.title.as_str())).style(RED_STYLE)
                } else if todo.completed {
                    ListItem::new(format!("  {}", todo.title.as_str())).style(GREEN_STYLE)
                } else {
                    ListItem::new(format!("  {}", todo.title.as_str()))
                }
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol("")
            .highlight_spacing(HighlightSpacing::WhenSelected);

        StatefulWidget::render(list, area, buf, &mut self.selected);
    }

    fn render_preview(&self, area: Rect, buf: &mut Buffer) {
        let block = crate::helpers::rounded_block(" Preview ");

        // Sometimes the ratatui list selection goes todos.len() + 1 so we need to clamp it
        let todo = match self.selected.selected() {
            Some(index) => match self
                .todos
                .get(index.min(self.todos.len().saturating_sub(1)))
            {
                Some(todo) => todo,
                None => &Todo::from(Some(0), Some("No task selected".to_string()), None, None),
            },
            None => &Todo::from(Some(0), Some("No task selected".to_string()), None, None),
        };

        Paragraph::new(todo.description.as_str())
            .block(block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.focus {
            AppFocus::TodoList => match key.code {
                KeyCode::Down => self.select_next(),
                KeyCode::Up => self.select_previous(),
                KeyCode::Esc => self.select_none(),
                KeyCode::Char(' ') => {
                    if self.multi_select {
                        self.toggle_delete();
                    } else {
                        self.toggle_completed();
                    }
                }
                KeyCode::Char('v') => {
                    self.multi_select = !self.multi_select;
                    self.selected_entries.clear();
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
                KeyCode::Char('s') => {
                    self.new_settings.quit = false;
                    self.focus = AppFocus::Settings;
                }
                _ => {}
            },
            AppFocus::NewTask => {
                self.new_task.handle_key(key);
                if self.new_task.quit {
                    if self.new_task.completed {
                        let todo = self.new_task.todo.clone();
                        if let Some(index) = self.find_entry(todo.id) {
                            self.todos[index] = todo;
                        } else {
                            self.todos.push(todo);
                        }
                        save_todos(&self.todos).unwrap();
                        self.new_task = NewTask::new();
                    }
                    self.focus = AppFocus::TodoList;
                }
            }
            AppFocus::DeletePrompt => match key.code {
                KeyCode::Char('y') => {
                    self.delete_entry();
                    self.focus = AppFocus::TodoList;
                }
                KeyCode::Char('n') => self.focus = AppFocus::TodoList,
                _ => {}
            },
            AppFocus::Settings => {
                self.new_settings.handle_key(key);
                if self.new_settings.quit {
                    let settings = self.new_settings.to_settings();
                    save_settings(&settings).unwrap();
                    self.focus = AppFocus::TodoList;
                }
            }
        }
        false
    }

    fn find_entry(&self, id: u128) -> Option<usize> {
        self.todos.iter().position(|t| t.id == id)
    }

    fn get_selected(&self) -> Option<Todo> {
        self.selected
            .selected()
            .and_then(|index| self.todos.get(index).cloned())
    }

    fn select_last_selected(&mut self) {
        if let Some(index) = self.last_selected {
            self.selected.select(Some(index));
        }
    }

    fn toggle_delete(&mut self) {
        if let Some(todo) = self.get_selected() {
            if self.selected_entries.contains(&todo) {
                self.selected_entries.retain(|t| *t != todo);
            } else {
                self.selected_entries.push(todo);
            }
        }
    }

    fn select_next(&mut self) {
        self.select_last_selected();
        self.selected.select_next();
    }

    fn select_previous(&mut self) {
        self.select_last_selected();
        self.selected.select_previous();
    }

    fn select_none(&mut self) {
        self.last_selected = self.selected.selected();
        self.selected.select(None);
    }

    fn toggle_completed(&mut self) {
        if let Some(index) = self.selected.selected() {
            if let Some(todo) = self.todos.get_mut(index) {
                todo.completed = !todo.completed;
            }
        }
    }

    fn delete_entry(&mut self) {
        if self.selected_entries.is_empty() {
            if let Some(index) = self.selected.selected() {
                self.todos.remove(index);
            }
        } else {
            for todo in self.selected_entries.iter() {
                self.todos.retain(|t| t != todo);
            }
        }
    }
}
