use crate::{
    handle_json::{load_todos, save_todos, Todo},
    new_task,
};
use new_task::NewTask;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Flex,
    prelude::*,
    style::palette::tailwind::{GREEN, RED},
    widgets::{
        Block, BorderType, Borders, Clear, HighlightSpacing, List, ListItem, ListState, Paragraph,
        Wrap,
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
                if self.selected_entries.contains(&todo) {
                    ListItem::new(format!("  {}", todo.title.as_str()))
                        .style(Style::default().fg(RED.c500))
                } else if todo.completed {
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
            Paragraph::new(" [q] Quit | [n] New Task | [d] Delete Task | [v] Multi select | [Enter] Open Task | [Space] Toggle Task ")
                .style(Style::default())
                .alignment(Alignment::Center)
                .block(Block::default());

        frame.render_widget(footer, footer_area);

        match self.focus {
            AppFocus::NewTask => {
                let new_task_area = popup_area(hero_area, 75, 75);
                self.new_task.draw(frame, new_task_area);
            }
            AppFocus::DeletePrompt => {
                confirm_prompt(
                    frame,
                    hero_area,
                    " Delete Task ",
                    "Are you sure you want to delete the selected task(s)?",
                );
            }
            _ => {}
        }
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
                        let todo = self.new_task.todo.clone();
                        self.todos.push(todo);
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

    fn get_selected(&self) -> Option<Todo> {
        self.selected
            .selected()
            .and_then(|index| self.todos.get(index).cloned())
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
            return;
        } else {
            for todo in self.selected_entries.iter() {
                self.todos.retain(|t| t != todo);
            }
        }
    }
}

pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

pub fn confirm_prompt(frame: &mut Frame, area: Rect, title: &str, description: &str) {
    let popup_area = popup_area(area, 30, 25);
    let block = Block::bordered()
        .title(Line::from(title).bold().centered())
        .border_type(BorderType::Rounded)
        .border_style(SECONDARY_STYLE)
        .title_bottom(vec![
            " [ ".into(),
            Span::styled("Y", Style::default().green().bold()),
            " ] ".into(),
        ])
        .title_bottom(vec![
            " [ ".into(),
            Span::styled("N", Style::default().red().bold()),
            " ] ".into(),
        ])
        .title_alignment(Alignment::Center)
        .title_style(Style::default().reset());
    let confirm = Paragraph::new(Line::from(description).centered())
        .wrap(Wrap { trim: true })
        .block(block);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(confirm, popup_area);
}
