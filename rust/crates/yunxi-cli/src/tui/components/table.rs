use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Row, Table as RatatuiTable, TableState};

pub struct Table {
    state: ComponentState,
    columns: Vec<Column>,
    rows: Vec<RowData>,
    sort_column: Option<usize>,
    sort_order: SortOrder,
    focused_row: usize,
    selected_rows: Vec<usize>,
    scroll_offset: usize,
    page_size: usize,
    fixed_columns: usize,
    on_select: Option<Box<dyn Fn(usize) -> ActionResult + Send + Sync>>,
    on_double_click: Option<Box<dyn Fn(usize) -> ActionResult + Send + Sync>>,
    on_sort: Option<Box<dyn Fn(usize, SortOrder) -> ActionResult + Send + Sync>>,
    style: TableStyle,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub width: u16,
    pub alignment: Alignment,
    pub sortable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone)]
pub struct RowData {
    pub cells: Vec<String>,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct TableStyle {
    pub header_bg: Color,
    pub header_fg: Color,
    pub normal_bg: Color,
    pub normal_fg: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub focused_bg: Color,
    pub focused_fg: Color,
    pub border: bool,
    pub show_row_numbers: bool,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            header_bg: Color::Rgb(36, 45, 60),
            header_fg: Color::Rgb(232, 232, 237),
            normal_bg: Color::Rgb(26, 35, 50),
            normal_fg: Color::Rgb(232, 232, 237),
            selected_bg: Color::Rgb(68, 138, 255),
            selected_fg: Color::Rgb(13, 13, 18),
            focused_bg: Color::Rgb(139, 176, 240),
            focused_fg: Color::Rgb(13, 13, 18),
            border: true,
            show_row_numbers: false,
        }
    }
}

impl Column {
    pub fn new(name: impl Into<String>, width: u16) -> Self {
        Self {
            name: name.into(),
            width,
            alignment: Alignment::Left,
            sortable: true,
        }
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }
}

impl RowData {
    pub fn new(cells: Vec<String>) -> Self {
        Self {
            cells,
            visible: true,
        }
    }
}

impl Table {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            state: ComponentState::new(generate_component_id("table")),
            columns,
            rows: Vec::new(),
            sort_column: None,
            sort_order: SortOrder::Ascending,
            focused_row: 0,
            selected_rows: Vec::new(),
            scroll_offset: 0,
            page_size: 10,
            fixed_columns: 0,
            on_select: None,
            on_double_click: None,
            on_sort: None,
            style: TableStyle::default(),
        }
    }

    pub fn with_rows(mut self, rows: Vec<RowData>) -> Self {
        self.rows = rows;
        self
    }

    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    pub fn with_fixed_columns(mut self, fixed_columns: usize) -> Self {
        self.fixed_columns = fixed_columns;
        self
    }

    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize) -> ActionResult + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn with_on_double_click<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize) -> ActionResult + Send + Sync + 'static,
    {
        self.on_double_click = Some(Box::new(callback));
        self
    }

    pub fn with_on_sort<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, SortOrder) -> ActionResult + Send + Sync + 'static,
    {
        self.on_sort = Some(Box::new(callback));
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn add_row(&mut self, row: RowData) {
        self.rows.push(row);
    }

    pub fn get_selected_rows(&self) -> &[usize] {
        &self.selected_rows
    }

    pub fn get_focused_row(&self) -> usize {
        self.focused_row
    }

    pub fn get_rows(&self) -> &[RowData] {
        &self.rows
    }

    pub fn get_columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn clear_selection(&mut self) {
        self.selected_rows.clear();
    }

    pub fn select_all(&mut self) {
        self.selected_rows = (0..self.rows.len()).collect();
    }

    pub fn sort_by_column(&mut self, column_index: usize) {
        if column_index >= self.columns.len() {
            return;
        }

        if Some(column_index) == self.sort_column {
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            self.sort_column = Some(column_index);
            self.sort_order = SortOrder::Ascending;
        }

        self.rows.sort_by(|a, b| {
            match self.sort_order {
                SortOrder::Ascending => {
                    a.cells.get(column_index)
                        .cmp(&b.cells.get(column_index))
                }
                SortOrder::Descending => {
                    b.cells.get(column_index)
                        .cmp(&a.cells.get(column_index))
                }
            }
        });

        if let Some(ref callback) = self.on_sort {
            callback(column_index, self.sort_order);
        }
    }

    pub fn filter<F>(&mut self, predicate: F)
    where
        F: Fn(&RowData) -> bool,
    {
        for row in &mut self.rows {
            row.visible = predicate(row);
        }
    }

    fn get_sort_symbol(&self, column_index: usize) -> &'static str {
        if Some(column_index) == self.sort_column {
            match self.sort_order {
                SortOrder::Ascending => "▲",
                SortOrder::Descending => "▼",
            }
        } else {
            ""
        }
    }
}

impl Component for Table {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let header_cells: Vec<String> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let symbol = self.get_sort_symbol(i);
                format!("{} {}", col.name, symbol)
            })
            .collect();

        let mut visible_rows = Vec::new();
        for (row_idx, row) in self.rows.iter().enumerate().filter(|(_, r)| r.visible) {
            let style = if self.selected_rows.contains(&row_idx) {
                Style::default()
                    .bg(self.style.selected_bg)
                    .fg(self.style.selected_fg)
            } else if row_idx == self.focused_row && self.state.focused {
                Style::default()
                    .bg(self.style.focused_bg)
                    .fg(self.style.focused_fg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .bg(self.style.normal_bg)
                    .fg(self.style.normal_fg)
            };

            let mut cells = row.cells.clone();
            if self.style.show_row_numbers {
                cells.insert(0, format!("{}", row_idx + 1));
            }

            visible_rows.push(Row::new(cells).style(style));
        }

        let column_widths: Vec<u16> = self
            .columns
            .iter()
            .map(|col| col.width)
            .collect();

        let table = RatatuiTable::new(
            vec![Row::new(header_cells)
                .style(Style::default().bg(self.style.header_bg).fg(self.style.header_fg))
                .bottom_margin(0)]
            .into_iter()
            .chain(visible_rows.into_iter()),
            column_widths,
        )
        .block(
            Block::default()
                .borders(if self.style.border {
                    Borders::ALL
                } else {
                    Borders::NONE
                })
                .style(Style::default()),
        )
        .style(Style::default());

        table.render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.state.visible {
            return ActionResult::Ignored;
        }

        let visible_count = self.rows.iter().filter(|r| r.visible).count();

        match event {
            Event::Input(InputEvent::Key(key)) => {
                match key.code {
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.focused_row < self.rows.len().saturating_sub(1) {
                            self.focused_row += 1;
                            if self.focused_row >= self.scroll_offset + self.page_size {
                                self.scroll_offset += 1;
                            }
                        }
                        ActionResult::Handled
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if self.focused_row > 0 {
                            self.focused_row -= 1;
                            if self.focused_row < self.scroll_offset {
                                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                            }
                        }
                        ActionResult::Handled
                    }
                    KeyCode::PageDown => {
                        let page = self.page_size.min(visible_count);
                        self.focused_row = (self.focused_row + page).min(self.rows.len().saturating_sub(1));
                        self.scroll_offset = self.scroll_offset.saturating_add(page).min(self.rows.len().saturating_sub(self.page_size));
                        ActionResult::Handled
                    }
                    KeyCode::PageUp => {
                        let page = self.page_size.min(visible_count);
                        self.focused_row = self.focused_row.saturating_sub(page);
                        self.scroll_offset = self.scroll_offset.saturating_sub(page);
                        ActionResult::Handled
                    }
                    KeyCode::Home => {
                        self.focused_row = 0;
                        self.scroll_offset = 0;
                        ActionResult::Handled
                    }
                    KeyCode::End => {
                        self.focused_row = self.rows.len().saturating_sub(1);
                        self.scroll_offset = self.rows.len().saturating_sub(self.page_size);
                        ActionResult::Handled
                    }
                    KeyCode::Tab => {
                        if let Some(col_idx) = self.sort_column {
                            let new_col = (col_idx + 1).min(self.columns.len() - 1);
                            self.sort_by_column(new_col);
                        } else if !self.columns.is_empty() {
                            self.sort_by_column(0);
                        }
                        ActionResult::Handled
                    }
                    KeyCode::BackTab => {
                        if let Some(col_idx) = self.sort_column {
                            let new_col = col_idx.saturating_sub(1);
                            self.sort_by_column(new_col);
                        } else if !self.columns.is_empty() {
                            self.sort_by_column(self.columns.len() - 1);
                        }
                        ActionResult::Handled
                    }
                    KeyCode::Enter => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            if let Some(ref callback) = self.on_double_click {
                                return callback(self.focused_row);
                            }
                        } else if let Some(ref callback) = self.on_select {
                            return callback(self.focused_row);
                        }
                        ActionResult::Ignored
                    }
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.select_all();
                        ActionResult::Handled
                    }
                    KeyCode::Char(' ') => {
                        if self.selected_rows.contains(&self.focused_row) {
                            self.selected_rows.retain(|&i| i != self.focused_row);
                        } else {
                            self.selected_rows.push(self.focused_row);
                        }
                        ActionResult::Handled
                    }
                    _ => ActionResult::Ignored,
                }
            }
            _ => ActionResult::Ignored,
        }
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    fn on_resize(&mut self, area: Rect) {
        self.state.bounds = area;
        self.page_size = area.height.saturating_sub(2) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_creation() {
        let columns = vec![
            Column::new("Name", 20),
            Column::new("Age", 10),
            Column::new("City", 15),
        ];
        let table = Table::new(columns);
        assert_eq!(table.get_columns().len(), 3);
    }

    #[test]
    fn test_table_sort() {
        let columns = vec![Column::new("Name", 20), Column::new("Age", 10)];
        let mut table = Table::new(columns).with_rows(vec![
            RowData::new(vec!["Alice".to_string(), "30".to_string()]),
            RowData::new(vec!["Bob".to_string(), "25".to_string()]),
        ]);
        table.sort_by_column(1);
        assert_eq!(table.get_rows()[0].cells[0], "Bob");
    }

    #[test]
    fn test_table_selection() {
        let columns = vec![Column::new("Name", 20)];
        let mut table = Table::new(columns).with_rows(vec![
            RowData::new(vec!["Alice".to_string()]),
            RowData::new(vec!["Bob".to_string()]),
        ]);
        table.handle_event(&Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        })));
        assert_eq!(table.get_selected_rows().len(), 1);
    }

    #[test]
    fn test_table_filter() {
        let columns = vec![Column::new("Name", 20)];
        let mut table = Table::new(columns).with_rows(vec![
            RowData::new(vec!["Alice".to_string()]),
            RowData::new(vec!["Bob".to_string()]),
            RowData::new(vec!["Charlie".to_string()]),
        ]);
        table.filter(|row| row.cells[0].starts_with('A'));
        assert_eq!(table.get_rows().iter().filter(|r| r.visible).count(), 1);
    }

    #[test]
    fn test_column_alignment() {
        let column = Column::new("Name", 20)
            .with_alignment(Alignment::Center)
            .with_sortable(false);
        assert_eq!(column.alignment, Alignment::Center);
        assert!(!column.sortable);
    }
}