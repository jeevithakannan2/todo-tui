use crate::{
    handle_json::{load_todos, save_todos, Todo},
    new_task,
};
use new_task::NewTask;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    style::palette::tailwind::{GREEN, RED},
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, Wrap,
    },
};

pub struct App<'a> {
    // Used to determine which part of the app is currently in focus
    focus: AppFocus,
    // The last selected index of the todo list before clearing the selection
    last_selected: Option<usize>,
    // Multi select mode
    multi_select: bool,
    // The state of the new task popup
    new_task: NewTask<'a>,
    // The state of the todo list
    selected: ListState,
    // Selected entries
    selected_entries: Vec<Todo>,
    // The list of parsed todos from json
    todos: Vec<Todo>,
}

#[derive(PartialEq)]
enum AppFocus {
    NewTask,
    TodoList,
    DeletePrompt,
}

pub const SECONDARY_STYLE: Style = Style::new().fg(Color::Blue);
pub const GREEN_STYLE: Style = Style::new().fg(GREEN.c500);
pub const RED_STYLE: Style = Style::new().fg(RED.c500);

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let [list_area, preview_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(main_area);

        self.render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_preview(preview_area, buf);

        match self.focus {
            AppFocus::TodoList => {}
            AppFocus::NewTask => {
                let new_task_area = crate::confirm::popup_area(main_area, 75, 75);
                self.new_task.render(new_task_area, buf);
            }
            AppFocus::DeletePrompt => {
                crate::confirm::Confirm::new(
                    " Delete Task ".into(),
                    "Are you sure you want to delete the selected task(s)?".into(),
                )
                .render(area, buf);
            }
        }
    }
}

impl App<'_> {
    pub fn new() -> Self {
        let todos = load_todos().unwrap_or_else(|_| Vec::new());
        Self {
            focus: AppFocus::TodoList,
            last_selected: None,
            multi_select: false,
            new_task: NewTask::new(),
            selected: ListState::default(),
            selected_entries: Vec::new(),
            todos,
        }
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let footer_text = match self.focus {
            AppFocus::TodoList => " [ ] Navigate | [q] Quit | [e] Edit Task | [n] New Task | [d] Delete Task | [v] Multi select | [Space] Toggle Task ",
            AppFocus::NewTask => self.new_task.footer_text(),
            AppFocus::DeletePrompt => "[y] Yes | [n] No",
        };

        Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Todo List ")
            .title_alignment(Alignment::Center)
            .title_style(Style::reset().bold())
            .borders(Borders::BOTTOM | Borders::TOP | Borders::LEFT)
            .border_type(BorderType::Rounded)
            .border_style(SECONDARY_STYLE);

        let items: Vec<ListItem> = self
            .todos
            .iter()
            .map(|todo| {
                if self.selected_entries.contains(&todo) {
                    ListItem::new(format!("  {}", todo.title.as_str()))
                        .style(RED_STYLE)
                } else if todo.completed {
                    ListItem::new(format!("  {}", todo.title.as_str()))
                        .style(GREEN_STYLE)
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
        let block = Block::bordered()
            .title(" Preview ")
            .title_alignment(Alignment::Center)
            .title_style(Style::reset().bold())
            .borders(Borders::BOTTOM | Borders::TOP | Borders::RIGHT)
            .border_type(BorderType::Rounded)
            .border_style(SECONDARY_STYLE);

        let inner_area = block.inner(area);
        block.render(area, buf);

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

        // `preview_inner_block` is used to print the separation line between the list and the preview
        let preview_inner_block = Block::bordered()
            .borders(Borders::LEFT)
            .border_style(SECONDARY_STYLE);
        Paragraph::new(todo.description.as_str())
            .style(Style::reset())
            .block(preview_inner_block)
            .wrap(Wrap { trim: true })
            .render(inner_area, buf);

    }

    pub fn on_key(&mut self, key: KeyEvent) -> bool {
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
                _ => {}
            },
            AppFocus::NewTask => {
                self.new_task.on_key(key);
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
                self.selected_entries.push(todo.clone());
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
