use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Cell, Clear, Row, Table, TableState},
};

use crate::{helpers::PopupSize, tasks::Task};

use super::{PRIMARY_STYLE, SELECTION_STYLE};

pub struct OverDue {
    state: TableState,
    tasks: Vec<Task>,
}

impl Widget for &mut OverDue {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = crate::helpers::create_popup_area(area, &PopupSize::Percentage { x: 50, y: 50 });
        Clear.render(area, buf);
        let block = crate::helpers::rounded_block(" Overdues ", PRIMARY_STYLE);
        let mut rows: Vec<Row> = Vec::new();
        let now = chrono::Local::now().naive_local();
        for task in &self.tasks {
            let title = task.title.as_str();
            let time = NaiveTime::parse_from_str(&task.time, "%H:%M").unwrap_or(now.time());
            let time = time.format("%H:%M").to_string();
            let row = Row::new(vec![
                Cell::from(title),
                Cell::from(task.date.as_str()),
                Cell::from(time),
            ]);
            rows.push(row);
        }
        let headers = Row::new(vec!["Title", "Date", "Time"])
            .style(Style::default().bold().reversed())
            .bottom_margin(1);

        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ],
        )
        .header(headers)
        .row_highlight_style(SELECTION_STYLE)
        .block(block);
        StatefulWidget::render(table, area, buf, &mut self.state);
    }
}

impl OverDue {
    pub fn new(tasks: &[Task]) -> Self {
        let tasks = Self::get_tasks(tasks);
        Self {
            state: TableState::default(),
            tasks,
        }
    }
    pub fn get_tasks(tasks: &[Task]) -> Vec<Task> {
        let now = chrono::Local::now().naive_local();
        let mut tasks: Vec<Task> = tasks
            .iter()
            .filter(|task| {
                let date = NaiveDate::parse_from_str(&task.date, "%d-%m-%Y").unwrap();
                let time = NaiveTime::parse_from_str(&task.time, "%H:%M").unwrap_or(now.time());
                date < now.date() || (date == now.date() && time < now.time())
            })
            .cloned()
            .collect();
        tasks.sort_by_key(|task| {
            NaiveDateTime::parse_from_str(
                format!("{} {}", task.time, task.date).as_str(),
                "%H:%M %d-%m-%Y",
            )
            .unwrap()
        });
        tasks
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true,
            KeyCode::Down => self.state.select_next(),
            KeyCode::Up => self.state.select_previous(),
            _ => {}
        }
        false
    }
}
