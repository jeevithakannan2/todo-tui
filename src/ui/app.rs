use chrono::{NaiveDate, NaiveTime};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::*,
    widgets::{Cell, Clear, List, ListState, Paragraph, Row, Table, TableState, Wrap},
};
use std::collections::BTreeMap;
use tui_textarea::TextArea;

use crate::{
    config::Config,
    helpers::{PopupSize, rounded_block},
    tasks::{Task, TaskStatus},
    theme::Theme,
};

use super::{Confirm, NewTask, OverDue};

pub struct App<'a> {
    theme: Theme,
    tasks: Tasks,
    right_area: RightArea,
    new_task: NewTask<'a>,
    new_task_save: Option<NewTask<'a>>,
    focus: AppFocus,
    state_save: Option<TableState>,
    state: TableState,
    total: usize,
    preview_scroll: (u16, u16),
    search: TextArea<'a>,
    config: Config,
    over_due: OverDue,
    filter: Filter,
}

struct Tasks {
    list: Vec<Task>,
    grouped: BTreeMap<NaiveDate, Vec<Task>>,
    selectable: Vec<(usize, u128)>,
}

#[derive(PartialEq)]
enum RightArea {
    Preview,
    NewTask,
    EditTask,
}

#[derive(PartialEq)]
enum AppFocus {
    LeftArea,
    RightArea,
    DeletePrompt,
    Search,
    FirstTimeSetup,
    ToggleEnc,
    OverDue,
    Filter,
}

struct Filter {
    state: ListState,
    mode: Option<TaskStatus>,
}

enum ScrollDirection {
    Up,
    Down,
}

pub const PRIMARY_STYLE: Style = Style::new().fg(Color::Rgb(166, 227, 161));
pub const SECONDARY_STYLE: Style = Style::new().fg(Color::Rgb(137, 180, 250));
pub const GREEN_STYLE: Style = Style::new().fg(Color::Rgb(0, 255, 0));
pub const RED_STYLE: Style = Style::new().fg(Color::Rgb(255, 0, 0));
pub const SELECTION_STYLE: Style = Style::new().fg(Color::Rgb(249, 226, 175));

impl App<'_> {
    pub fn new(new: bool, config: Config) -> Self {
        let mut tasks = if config.encryption {
            crate::tasks::load_encrypted().unwrap_or_else(|_| Vec::new())
        } else {
            crate::tasks::load().unwrap_or_else(|_| Vec::new())
        };

        crate::tasks::update_overdue(&mut tasks);

        let display_list = Self::get_display_tasks(&tasks, &None);
        let group = Self::group_date_tasks(&display_list);

        let mut text_area = TextArea::default();
        text_area.set_placeholder_text(
            "Press / to search by title or date range (DD MM YYYY - DD MM YYYY)",
        );
        text_area.set_cursor_line_style(Style::default());

        let overdue_tasks = OverDue::get_tasks(&tasks);

        let focus = if new {
            AppFocus::FirstTimeSetup
        } else if tasks.is_empty() {
            AppFocus::RightArea
        } else if !overdue_tasks.is_empty() {
            AppFocus::OverDue
        } else {
            AppFocus::LeftArea
        };

        Self {
            focus,
            state_save: None,
            state: TableState::default(),
            total: group.2,
            preview_scroll: (0, 0),
            theme: Theme::Default,
            over_due: OverDue::new(overdue_tasks),
            tasks: Tasks {
                list: tasks,
                grouped: group.1,
                selectable: group.0,
            },
            new_task: NewTask::new(),
            new_task_save: None,
            right_area: RightArea::NewTask,
            search: text_area,
            filter: Filter {
                state: ListState::default().with_selected(Some(0)),
                mode: None,
            },
            config,
        }
    }

    fn get_display_tasks(tasks: &[Task], display_status: &Option<TaskStatus>) -> Vec<Task> {
        match display_status {
            Some(status) => tasks
                .iter()
                .filter(|t| t.status == *status)
                .cloned()
                .collect(),
            None => tasks.to_vec(),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Clear, area);
        // The right area determination should always be first so that the footer text get correctly
        if self.state.selected().is_none() {
            self.right_area = RightArea::NewTask
        } else if self.right_area != RightArea::EditTask {
            self.right_area = RightArea::Preview
        }

        let footer_text: Line = self.get_footer_text().into();
        let footer_height =
            (1 + footer_text.width().try_into().unwrap_or(0) / (area.width + 1)).min(3);

        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(footer_height)]).areas(area);

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(main_area);

        // Left area
        let [search_area, list_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(left_area);

        // Render Search
        let border_style = self.get_border_style(AppFocus::Search);
        let cursor_style = if border_style == PRIMARY_STYLE {
            Style::default().reversed()
        } else {
            Style::default()
        };
        self.search.set_cursor_style(cursor_style);
        self.search
            .set_block(rounded_block(" Search ".into(), border_style));
        frame.render_widget(&self.search, search_area);

        // Render Task list
        let style = self.get_border_style(AppFocus::LeftArea);
        let title = match self.filter.mode {
            Some(TaskStatus::Completed) => " Tasks - Completed ",
            Some(TaskStatus::OverDue) => " Tasks - Overdue ",
            Some(TaskStatus::Pending) => " Tasks - Pending ",
            None => " Tasks ",
        };
        let block = crate::helpers::rounded_block(title.into(), style);

        let mut rows: Vec<Row> = Vec::with_capacity(self.total);
        let now = chrono::Local::now().naive_local();

        for (date, tasks) in &self.tasks.grouped {
            // Format the date header based on its relation to today
            let date_header = if *date == now.date() {
                "Today".to_string()
            } else if *date == now.date().succ_opt().unwrap_or(now.date()) {
                "Tomorrow".to_string()
            } else {
                format!("{} {}", date.format("%a"), date.format("%b %d %Y"))
            };

            // Add a date header row (Non-selectable)
            let header_row = Row::new(vec![Cell::from(date_header).style(SECONDARY_STYLE.bold())]);

            rows.push(header_row);
            // Add tasks under the date
            for (i, task) in tasks.iter().enumerate() {
                let title = task.title.as_str();

                let (icon, style) = if task.status == TaskStatus::Completed {
                    (self.theme.get_completed(), Style::default().dark_gray())
                } else if task.is_overdue() {
                    (self.theme.get_uncompleted(), Style::default().red().bold())
                } else {
                    (self.theme.get_uncompleted(), Style::default().bold())
                };

                let mut task_row =
                    Row::new(vec![Cell::from(format!("{} {}", icon, title)).style(style)]);

                // Last task of the date add a extra line to separate the next date
                if i == tasks.len() - 1 {
                    task_row = task_row.bottom_margin(1);
                }

                rows.push(task_row);
            }
        }

        let table = Table::new(rows, &[Constraint::Fill(1)])
            .block(block)
            .row_highlight_style(SELECTION_STYLE);

        frame.render_stateful_widget(table, list_area, &mut self.state);

        // Right area
        // Render Border
        let style = self.get_border_style(AppFocus::RightArea);

        let title = match self.right_area {
            RightArea::Preview => " Preview ",
            RightArea::NewTask => " New Task ",
            RightArea::EditTask => " Edit Task ",
        };

        let block = crate::helpers::rounded_block(title.into(), style);
        frame.render_widget(&block, right_area);

        // Render Preview or NewTask
        let right_area = block.inner(right_area);
        if self.right_area == RightArea::Preview {
            let task = self.get_selected().unwrap();
            let description = task.description.as_str();
            self.verify_preview_scroll(description.lines().count() as u16, area);
            let text = tui_markdown::from_str(description).style(Style::default());
            let preview = Paragraph::new(text)
                .scroll(self.preview_scroll)
                .wrap(Wrap { trim: true });
            frame.render_widget(preview, right_area);
        } else {
            frame.render_widget(&mut self.new_task, right_area);
        }

        // Render Footer
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(footer, footer_area);

        // Render popup prompts
        match self.focus {
            AppFocus::OverDue => frame.render_widget(&mut self.over_due, main_area),
            AppFocus::DeletePrompt => {
                let confirm = Confirm::new(
                    " Delete Task ".into(),
                    "Delete the selected task?".into(),
                    PopupSize::Percentage { x: 20, y: 15 },
                );
                frame.render_widget(confirm, main_area);
            }
            AppFocus::ToggleEnc => {
                let (title, body) = if self.config.encryption {
                    (
                        " Disable Encryption ",
                        "Would you like to disable encryption?",
                    )
                } else {
                    (
                        " Enable Encryption ",
                        "Would you like to enable encryption?",
                    )
                };
                let confirm = Confirm::new(
                    title.into(),
                    body.into(),
                    PopupSize::Percentage { x: 20, y: 15 },
                );
                frame.render_widget(confirm, main_area);
            }
            AppFocus::FirstTimeSetup => {
                let widget = Confirm::new(
                    " Welcome to TodoTUI ".into(),
                    "This is a one time setup. \n Would you like to enable encryption?".into(),
                    PopupSize::Percentage { x: 20, y: 20 },
                );
                frame.render_widget(widget, main_area);
            }
            AppFocus::Filter => {
                let popup_area = crate::helpers::create_popup_area(
                    main_area,
                    &PopupSize::Fixed {
                        width: 17,
                        height: 6,
                    },
                );
                let block =
                    crate::helpers::rounded_block(Line::from(" Filter ").centered(), PRIMARY_STYLE);
                let list = vec![
                    Line::from("All").centered(),
                    Line::from("Completed").centered(),
                    Line::from("Pending").centered(),
                    Line::from("Overdue").centered(),
                ];
                let list = List::new(list)
                    .block(block)
                    .highlight_style(SELECTION_STYLE);
                frame.render_widget(Clear, popup_area);
                frame.render_stateful_widget(list, popup_area, &mut self.filter.state);
            }
            _ => {}
        }
    }

    fn get_filtered_tasks(&self) -> Vec<Task> {
        let search_text = &self.search.lines()[0];
        if search_text.is_empty() {
            return self.tasks.list.clone();
        }

        let dates: Vec<&str> = search_text.split(" - ").collect();
        if dates.len() == 2 {
            let date1 = NaiveDate::parse_from_str(dates[0].trim(), "%d %m %Y").ok();
            let date2 = NaiveDate::parse_from_str(dates[1].trim(), "%d %m %Y").ok();

            if let (Some(date1), Some(date2)) = (date1, date2) {
                return self
                    .tasks
                    .list
                    .iter()
                    .filter(|t| {
                        let task_date = NaiveDate::parse_from_str(&t.date, "%d %m %Y").unwrap();
                        task_date >= date1 && task_date <= date2
                    })
                    .cloned()
                    .collect();
            }
        }

        self.tasks
            .list
            .iter()
            .filter(|t| t.title.contains(search_text))
            .cloned()
            .collect()
    }

    fn get_border_style(&self, focus: AppFocus) -> Style {
        if self.focus == focus {
            PRIMARY_STYLE
        } else {
            SECONDARY_STYLE
        }
    }

    fn group_date_tasks(
        tasks: &[Task],
    ) -> (Vec<(usize, u128)>, BTreeMap<NaiveDate, Vec<Task>>, usize) {
        let mut grouped_tasks: BTreeMap<NaiveDate, Vec<Task>> = BTreeMap::new();
        for task in tasks {
            let date = NaiveDate::parse_from_str(&task.date, "%d %m %Y").unwrap();
            grouped_tasks.entry(date).or_default().push(task.clone());
        }

        // Sort by time
        for (_, task_list) in grouped_tasks.iter_mut() {
            task_list.sort_by_key(|task| NaiveTime::parse_from_str(&task.time, "%H %M").unwrap());
        }

        let mut selectable: Vec<(usize, u128)> = Vec::new();
        let mut idx = 0;

        for (_, tasks) in &grouped_tasks {
            idx += 1;
            for task in tasks {
                selectable.push((idx, task.id));
                idx += 1;
            }
        }
        (selectable, grouped_tasks, idx)
    }

    // Verify that the preview scroll is within bounds
    fn verify_preview_scroll(&mut self, preview_lines: u16, preview_area: Rect) {
        let preview_area_height = preview_area.height;
        self.preview_scroll.0 = if preview_lines < self.preview_scroll.0 + preview_area_height {
            preview_lines.saturating_sub(preview_area_height)
        } else {
            self.preview_scroll.0
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if matches!(self.focus, AppFocus::LeftArea | AppFocus::RightArea) {
            if key.code == KeyCode::Char('q') {
                return true;
            }
        }
        match self.focus {
            AppFocus::LeftArea => match key.code {
                KeyCode::BackTab | KeyCode::Tab => {
                    self.focus = AppFocus::RightArea;
                    if self.right_area != RightArea::Preview {
                        self.new_task.quit = false;
                    }
                }
                KeyCode::Down => self.scroll(ScrollDirection::Down),
                KeyCode::Up => self.scroll(ScrollDirection::Up),
                KeyCode::Esc => self.select_none(),
                KeyCode::Char(' ') => self.toggle_completed(),
                KeyCode::Char('t') => self.theme = self.theme.change_theme(),
                KeyCode::Char('d') => {
                    if let Some(_) = self.get_selected() {
                        self.focus = AppFocus::DeletePrompt
                    }
                }
                KeyCode::Char('e') => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        self.focus = AppFocus::ToggleEnc;
                        return false;
                    }
                    if let Some(task) = self.get_selected() {
                        self.new_task = NewTask::from(task);
                        self.focus = AppFocus::RightArea;
                        self.save_new_task_state();
                        self.right_area = RightArea::EditTask;
                    }
                }
                KeyCode::Char('p') => self.right_area = RightArea::Preview,
                KeyCode::Char('f') => self.focus = AppFocus::Filter,
                KeyCode::Char('n') => {
                    self.restore_new_task_state();
                    self.new_task.quit = false;
                    self.focus = AppFocus::RightArea;
                    self.right_area = RightArea::NewTask;
                    self.select_none();
                }
                KeyCode::Char('/') => self.focus = AppFocus::Search,
                _ => {}
            },
            AppFocus::RightArea => {
                if self.right_area != RightArea::Preview {
                    self.new_task.handle_key(key);
                    if self.new_task.quit {
                        if self.new_task.completed {
                            self.add_or_modify_task();
                            self.search.select_all();
                            self.search.delete_newline();
                            self.select_added_task(self.new_task.get_task().id);
                            self.right_area = RightArea::Preview;
                            self.new_task = NewTask::new();
                        } else {
                            self.save_new_task_state();
                            self.select_last_selected();
                        }
                        self.focus = AppFocus::LeftArea;
                    }
                } else {
                    match key.code {
                        KeyCode::BackTab | KeyCode::Tab => self.focus = AppFocus::LeftArea,
                        KeyCode::Down => self.scroll_preview_down(),
                        KeyCode::Up => self.scroll_preview_up(),
                        _ => {}
                    }
                }
            }
            AppFocus::DeletePrompt => match key.code {
                KeyCode::Char('y') => {
                    self.delete_entry();
                    self.focus = AppFocus::LeftArea;
                }
                KeyCode::Char('n') => self.focus = AppFocus::LeftArea,
                _ => {}
            },
            AppFocus::Search => match key.code {
                KeyCode::Esc => self.focus = AppFocus::LeftArea,
                KeyCode::Enter => self.focus = AppFocus::LeftArea,
                KeyCode::BackTab | KeyCode::Tab => self.focus = AppFocus::LeftArea,
                _ => {
                    self.search.input(key);
                    let searched_tasks = self.get_filtered_tasks();
                    let group = Self::group_date_tasks(&searched_tasks);
                    self.tasks.selectable = group.0;
                    self.tasks.grouped = group.1;
                    self.total = group.2;
                    self.state.select(None);
                }
            },
            AppFocus::FirstTimeSetup => match key.code {
                KeyCode::Char('y') => {
                    self.config.encryption = true;
                    self.config.save();
                    crate::auth::generate_key();
                    self.update_task_list();
                    self.focus = AppFocus::LeftArea;
                }
                KeyCode::Char('n') => {
                    self.config.save();
                    self.focus = AppFocus::LeftArea;
                }
                _ => {}
            },
            AppFocus::OverDue => {
                if self.over_due.handle_key(key) {
                    self.focus = AppFocus::LeftArea;
                }
            }
            AppFocus::ToggleEnc => match key.code {
                KeyCode::Char('y') => {
                    self.config.encryption = !self.config.encryption;
                    self.config.save();
                    self.focus = AppFocus::LeftArea
                }
                KeyCode::Char('n') => self.focus = AppFocus::LeftArea,
                _ => {}
            },
            AppFocus::Filter => match key.code {
                KeyCode::Enter => {
                    self.filter.state.selected().map(|i| {
                        self.filter.mode = match i {
                            0 => None,
                            1 => Some(TaskStatus::Completed),
                            2 => Some(TaskStatus::Pending),
                            3 => Some(TaskStatus::OverDue),
                            _ => None,
                        };
                    });
                    self.update_task_list();
                    self.state.select(None);
                    self.focus = AppFocus::LeftArea;
                }
                KeyCode::Esc | KeyCode::Char('q') => self.focus = AppFocus::LeftArea,
                KeyCode::Down => self.filter.state.select_next(),
                KeyCode::Up => self.filter.state.select_previous(),
                _ => {}
            },
        }
        false
    }

    fn select_added_task(&mut self, task_id: u128) {
        let idx = self
            .tasks
            .selectable
            .iter()
            .find(|(_, id)| *id == task_id)
            .unwrap();
        self.state.select(Some(idx.0));
    }

    fn add_or_modify_task(&mut self) {
        let task = self.new_task.get_task().clone();
        if self.right_area == RightArea::EditTask {
            if let Some(selected_task) = self.get_selected_mut() {
                *selected_task = task;
            }
        } else {
            self.tasks.list.push(task);
        }
        self.update_task_list();
    }

    fn scroll_preview_up(&mut self) {
        self.preview_scroll.0 = self.preview_scroll.0.saturating_sub(1);
    }

    fn scroll_preview_down(&mut self) {
        self.preview_scroll.0 += 1;
    }

    fn get_selected(&self) -> Option<Task> {
        self.state
            .selected()
            .and_then(|index| self.tasks.selectable.iter().find(|(i, _)| *i == index))
            .and_then(|(_, id)| self.tasks.list.iter().find(|t| t.id == *id))
            .cloned()
    }

    fn get_selected_mut(&mut self) -> Option<&mut Task> {
        self.state
            .selected()
            .and_then(|index| self.tasks.selectable.iter().find(|(i, _)| *i == index))
            .and_then(|(_, id)| self.tasks.list.iter_mut().find(|t| t.id == *id))
    }

    fn select_last_selected(&mut self) -> bool {
        if let Some(old_selection) = self.state_save.take() {
            self.state = old_selection;
            return true;
        }
        false
    }

    fn restore_new_task_state(&mut self) {
        self.new_task = if let Some(save) = self.new_task_save.take() {
            save.clone()
        } else {
            NewTask::new()
        };
    }

    fn save_new_task_state(&mut self) {
        if self.right_area == RightArea::NewTask {
            self.new_task_save = Some(self.new_task.clone());
        }
    }

    fn scroll(&mut self, scroll_direction: ScrollDirection) {
        if self.tasks.selectable.is_empty() || self.select_last_selected() {
            return;
        }
        let index = match self.state.selected() {
            Some(index) => index,
            None => 0,
        };
        let mut next = index;
        match scroll_direction {
            ScrollDirection::Up => {
                next = next.saturating_sub(1);
                while !self.tasks.selectable.iter().any(|&(i, _)| i == next) {
                    next = next.saturating_sub(1);
                    if next <= 0 {
                        next = self.tasks.selectable.last().copied().unwrap_or((0, 0)).0;
                    }
                }
            }
            ScrollDirection::Down => {
                next += 1;
                while !self.tasks.selectable.iter().any(|&(i, _)| i == next) {
                    next += 1;
                    if next >= self.total {
                        next = self.tasks.selectable[0].0;
                    }
                }
            }
        }

        self.state.select(Some(next));

        if self.right_area == RightArea::EditTask {
            self.new_task = NewTask::from(self.get_selected().unwrap());
        }
    }

    fn select_none(&mut self) {
        self.state_save = Some(self.state.clone());
        self.state.select(None);
    }

    fn update_task_list(&mut self) {
        let display_list = Self::get_display_tasks(&self.tasks.list, &self.filter.mode);
        let grouped_tasks = Self::group_date_tasks(&display_list);
        self.tasks.selectable = grouped_tasks.0;
        self.tasks.grouped = grouped_tasks.1;
        self.total = grouped_tasks.2;
        if self.config.encryption {
            crate::tasks::save_encrypted(&self.tasks.list).unwrap();
        } else {
            crate::tasks::save(&self.tasks.list).unwrap();
        }
    }

    fn toggle_completed(&mut self) {
        if let Some(task) = self.get_selected_mut() {
            if task.status == TaskStatus::Completed {
                if task.is_overdue() {
                    task.status = TaskStatus::OverDue;
                } else {
                    task.status = TaskStatus::Pending;
                }
            } else {
                task.status = TaskStatus::Completed;
            }
            self.update_task_list();
        }
    }

    fn delete_entry(&mut self) {
        if let Some(task) = self.get_selected() {
            self.tasks.list.retain(|t| t.id != task.id);
            self.update_task_list();
            if let Some(_) = self.state.selected() {
                self.scroll(ScrollDirection::Down);
            }
        }
        if self.tasks.list.is_empty() {
            self.state.select(None);
        }
    }

    fn get_footer_text(&self) -> String {
        let arrows = if self.theme == Theme::Default {
            "[ ] Navigate"
        } else {
            "[Up/Down] Navigate"
        };

        let mut footer_text = Vec::new();
        match self.focus {
            AppFocus::LeftArea => {
                footer_text.push(arrows);
                if self.state.selected().is_some() {
                    footer_text.extend_from_slice(&[
                        "[e] Edit Task",
                        "[d] Delete Task",
                        "[Space] Toggle Completed",
                    ]);
                }
                footer_text.push("[f] Filter");
                footer_text.push("[c] Show completed");
                footer_text.push("[n] New Task");
                footer_text.push("[t] Compatibility Mode");
                if self.right_area != RightArea::Preview && self.state.selected().is_some() {
                    footer_text.push("[p] Preview");
                }
                let title = match self.right_area {
                    RightArea::EditTask => "[Tab] Focus Edit Task",
                    RightArea::NewTask => "[Tab] Focus New Task",
                    RightArea::Preview => "[Tab] Focus Preview",
                };
                let enc = if self.config.encryption {
                    "[C-e] Disable Encryption"
                } else {
                    "[C-e] Enable Encryption"
                };
                footer_text.extend_from_slice(&[title, enc, "[q] Quit"]);
            }
            AppFocus::RightArea => {
                if self.right_area != RightArea::Preview {
                    footer_text = self.new_task.footer_text().to_vec();
                } else {
                    footer_text.extend_from_slice(&[arrows, "[Tab] Focus Tasks", "[q] Quit"]);
                }
            }
            AppFocus::DeletePrompt | AppFocus::FirstTimeSetup | AppFocus::ToggleEnc => {
                footer_text.extend_from_slice(&["[y] Yes", "[n] No"]);
            }
            AppFocus::Search => {
                footer_text.extend_from_slice(&["[Esc] Exit Search", "[Enter] Exit Search"]);
            }
            AppFocus::OverDue => {
                footer_text.extend_from_slice(&[arrows, "[q] Quit"]);
            }
            AppFocus::Filter => {
                footer_text.extend_from_slice(&[arrows, "[Enter] Apply", "[Esc][q] Cancel"]);
            }
        }
        footer_text.join(" | ")
    }
}
