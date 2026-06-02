use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, InputEvent};
use crate::tui::form::{FormLayoutConfig, ValidatorSet};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    TextField,
    NumberField,
    BooleanField,
    SelectField,
    MultiSelectField,
    TextArea,
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub id: String,
    pub label: String,
    pub field_type: FieldType,
    pub value: String,
    pub placeholder: String,
    pub options: Vec<String>,
    pub required: bool,
    pub validators: ValidatorSet<String>,
    pub error_message: Option<String>,
    pub help_text: Option<String>,
    pub disabled: bool,
    pub editable: bool,
}

impl FormField {
    pub fn new(id: impl Into<String>, label: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            field_type,
            value: String::new(),
            placeholder: String::new(),
            options: Vec::new(),
            required: false,
            validators: ValidatorSet::new(),
            error_message: None,
            help_text: None,
            disabled: false,
            editable: true,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.options = options;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_validator<V>(mut self, validator: V) -> Self
    where
        V: crate::tui::form::Validator<String> + 'static + Send + Sync,
    {
        self.validators = self.validators.add(validator);
        self
    }

    pub fn with_help_text(mut self, help_text: impl Into<String>) -> Self {
        self.help_text = Some(help_text.into());
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn validate(&mut self) -> Result<(), String> {
        if self.disabled {
            return Ok(());
        }

        if self.required && self.validators.is_empty() {
            let validator = crate::tui::form::RequiredValidator;
            if let Err(e) = validator.validate(&self.value) {
                self.error_message = Some(e);
                return Err(e);
            }
        }

        if let Err(e) = self.validators.validate(&self.value) {
            self.error_message = Some(e);
            return Err(e);
        }

        self.error_message = None;
        Ok(())
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn is_valid(&self) -> bool {
        self.error_message.is_none()
    }
}

pub struct Form {
    state: ComponentState,
    fields: Vec<FormField>,
    selected_field: usize,
    layout_config: FormLayoutConfig,
    show_validation: bool,
    submit_callback: Option<Box<dyn Fn(&[FormField]) -> ActionResult + Send + Sync>>,
    cancel_callback: Option<Box<dyn Fn() -> ActionResult + Send + Sync>>,
}

impl Form {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("form")),
            fields: Vec::new(),
            selected_field: 0,
            layout_config: FormLayoutConfig::default(),
            show_validation: true,
            submit_callback: None,
            cancel_callback: None,
        }
    }

    pub fn with_layout(mut self, config: FormLayoutConfig) -> Self {
        self.layout_config = config;
        self
    }

    pub fn with_validation(mut self, show: bool) -> Self {
        self.show_validation = show;
        self
    }

    pub fn with_submit<F>(mut self, callback: F) -> Self
    where
        F: Fn(&[FormField]) -> ActionResult + Send + Sync + 'static,
    {
        self.submit_callback = Some(Box::new(callback));
        self
    }

    pub fn with_cancel<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> ActionResult + Send + Sync + 'static,
    {
        self.cancel_callback = Some(Box::new(callback));
        self
    }

    pub fn add_field(&mut self, field: FormField) {
        self.fields.push(field);
    }

    pub fn remove_field(&mut self, id: &str) {
        if let Some(pos) = self.fields.iter().position(|f| f.id == id) {
            self.fields.remove(pos);
            if self.selected_field >= self.fields.len() && !self.fields.is_empty() {
                self.selected_field = self.fields.len() - 1;
            }
        }
    }

    pub fn get_field(&self, id: &str) -> Option<&FormField> {
        self.fields.iter().find(|f| f.id == id)
    }

    pub fn get_field_mut(&mut self, id: &str) -> Option<&mut FormField> {
        self.fields.iter_mut().find(|f| f.id == id)
    }

    pub fn set_field_value(&mut self, id: &str, value: impl Into<String>) {
        if let Some(field) = self.get_field_mut(id) {
            field.value = value.into();
            field.clear_error();
        }
    }

    pub fn validate_all(&mut self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for field in &mut self.fields {
            if let Err(e) = field.validate() {
                errors.push(format!("{}: {}", field.label, e));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn is_valid(&self) -> bool {
        self.fields.iter().all(|f| f.is_valid())
    }

    pub fn submit(&mut self) -> ActionResult {
        if self.show_validation {
            if let Err(errors) = self.validate_all() {
                let error_msg = errors.join("; ");
                return ActionResult::Action(Action::Custom(format!(
                    "form_validation_error: {}",
                    error_msg
                )));
            }
        }

        if let Some(callback) = &self.submit_callback {
            callback(&self.fields)
        } else {
            ActionResult::Action(Action::Custom("form_submit".to_string()))
        }
    }

    pub fn cancel(&mut self) -> ActionResult {
        if let Some(callback) = &self.cancel_callback {
            callback()
        } else {
            ActionResult::Action(Action::Close)
        }
    }

    fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) -> ActionResult {
        match key {
            KeyCode::Esc => self.cancel(),
            KeyCode::Enter => {
                if modifiers.contains(KeyModifiers::ALT) {
                    self.submit()
                } else {
                    ActionResult::Handled
                }
            }
            KeyCode::Tab => {
                if !self.fields.is_empty() {
                    self.selected_field = (self.selected_field + 1) % self.fields.len();
                }
                ActionResult::Handled
            }
            KeyCode::BackTab => {
                if !self.fields.is_empty() {
                    self.selected_field = if self.selected_field == 0 {
                        self.fields.len() - 1
                    } else {
                        self.selected_field - 1
                    };
                }
                ActionResult::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_field > 0 {
                    self.selected_field -= 1;
                }
                ActionResult::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_field < self.fields.len().saturating_sub(1) {
                    self.selected_field += 1;
                }
                ActionResult::Handled
            }
            KeyCode::Char(c)
                if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(field) = self.fields.get_mut(self.selected_field) {
                    if field.editable && !field.disabled {
                        match field.field_type {
                            FieldType::TextField | FieldType::TextArea => {
                                field.value.push(c);
                                field.clear_error();
                                ActionResult::Handled
                            }
                            FieldType::BooleanField => {
                                field.value = if field.value == "true" {
                                    "false".to_string()
                                } else {
                                    "true".to_string()
                                };
                                ActionResult::Handled
                            }
                            _ => ActionResult::Ignored,
                        }
                    } else {
                        ActionResult::Ignored
                    }
                } else {
                    ActionResult::Ignored
                }
            }
            KeyCode::Backspace => {
                if let Some(field) = self.fields.get_mut(self.selected_field) {
                    if field.editable && !field.disabled {
                        match field.field_type {
                            FieldType::TextField | FieldType::TextArea => {
                                field.value.pop();
                                ActionResult::Handled
                            }
                            _ => ActionResult::Ignored,
                        }
                    } else {
                        ActionResult::Ignored
                    }
                } else {
                    ActionResult::Ignored
                }
            }
            _ => ActionResult::Ignored,
        }
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Form {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(2)])
            .split(area);

        let label_width = self.layout_config.label_width;
        let field_spacing = self.layout_config.field_spacing;

        let mut y_offset = 0;

        for (i, field) in self.fields.iter().enumerate() {
            if y_offset >= chunks[0].height {
                break;
            }

            let is_selected = i == self.selected_field;
            let has_error = field.error_message.is_some();

            let label_style = if field.required {
                Style::default()
                    .fg(Color::Rgb(240, 139, 139))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Rgb(139, 176, 240))
                    .add_modifier(Modifier::BOLD)
            };

            let field_style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(50, 60, 80))
            } else if field.disabled {
                Style::default()
                    .fg(Color::Rgb(100, 100, 100))
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default()
            };

            let error_style = Style::default().fg(Color::Rgb(255, 100, 100));

            let label_text = format!("{}{}", field.label, if field.required { " *" } else { "" });

            let value_display = match field.field_type {
                FieldType::TextField | FieldType::TextArea => {
                    if field.value.is_empty() {
                        field.placeholder.clone()
                    } else {
                        field.value.clone()
                    }
                }
                FieldType::BooleanField => {
                    if field.value == "true" { "是" } else { "否" }.to_string()
                }
                FieldType::SelectField => field
                    .options
                    .get(field.value.parse::<usize>().unwrap_or(0))
                    .map(|s| s.clone())
                    .unwrap_or_else(|| field.value.clone()),
                FieldType::MultiSelectField => {
                    format!(
                        "已选择 {} 项",
                        field.value.split(',').filter(|s| !s.is_empty()).count()
                    )
                }
                FieldType::NumberField => field.value.clone(),
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:width$}", label_text, width = label_width as usize),
                    label_style,
                ),
                Span::raw(" "),
                Span::styled(value_display, field_style),
            ]);

            let paragraph = Paragraph::new(line).wrap(Wrap { trim: true });

            let field_area = Rect::new(chunks[0].x, chunks[0].y + y_offset, chunks[0].width, 1);

            paragraph.render(field_area, buf);

            if let Some(ref error) = field.error_message {
                if self.show_validation {
                    let error_line = Line::from(vec![
                        Span::raw(format!("{:width$}", "", width = label_width as usize)),
                        Span::raw(" "),
                        Span::styled(format!("! {}", error), error_style),
                    ]);

                    let error_paragraph = Paragraph::new(error_line).wrap(Wrap { trim: true });

                    let error_area =
                        Rect::new(chunks[0].x, chunks[0].y + y_offset + 1, chunks[0].width, 1);

                    error_paragraph.render(error_area, buf);
                    y_offset += 1;
                }
            }

            if let Some(ref help) = field.help_text {
                if y_offset + 1 < chunks[0].height {
                    let help_style = Style::default().fg(Color::Rgb(150, 150, 150));
                    let help_line = Line::from(vec![
                        Span::raw(format!("{:width$}", "", width = label_width as usize)),
                        Span::raw(" "),
                        Span::styled(format!("ℹ {}", help), help_style),
                    ]);

                    let help_paragraph = Paragraph::new(help_line).wrap(Wrap { trim: true });

                    let help_area =
                        Rect::new(chunks[0].x, chunks[0].y + y_offset + 1, chunks[0].width, 1);

                    help_paragraph.render(help_area, buf);
                    y_offset += 1;
                }
            }

            y_offset += field_spacing + 1;
        }

        let help_text = format!("[↑/↓]选择 [Enter]确认 [Alt+Enter]提交 [Tab]下一字段 [ESC]取消",);

        let help_style = Style::default().fg(Color::Rgb(150, 150, 150));
        let help_line = Line::from(Span::styled(help_text, help_style));

        let help_paragraph = Paragraph::new(help_line).alignment(Alignment::Center);

        help_paragraph.render(chunks[1], buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(KeyEvent {
                code,
                modifiers,
                kind: _,
                state: _,
            })) => self.handle_key_event(*code, *modifiers),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::form::{FormLayout, LengthValidator};

    #[test]
    fn test_form_creation() {
        let form = Form::new();
        assert!(form.fields.is_empty());
        assert_eq!(form.selected_field, 0);
        assert!(form.show_validation);
    }

    #[test]
    fn test_form_with_layout() {
        let config = FormLayoutConfig::horizontal();
        let form = Form::new().with_layout(config);
        assert_eq!(form.layout_config.layout, FormLayout::Horizontal);
    }

    #[test]
    fn test_form_field_creation() {
        let field = FormField::new("name", "姓名", FieldType::TextField);
        assert_eq!(field.id, "name");
        assert_eq!(field.label, "姓名");
        assert_eq!(field.field_type, FieldType::TextField);
        assert!(!field.required);
    }

    #[test]
    fn test_form_field_builder() {
        let field = FormField::new("email", "邮箱", FieldType::TextField)
            .with_value("user@example.com")
            .with_placeholder("请输入邮箱")
            .with_required(true)
            .with_help_text("用于接收通知");

        assert_eq!(field.value, "user@example.com");
        assert_eq!(field.placeholder, "请输入邮箱");
        assert!(field.required);
        assert_eq!(field.help_text, Some("用于接收通知".to_string()));
    }

    #[test]
    fn test_form_field_with_validator() {
        let field = FormField::new("username", "用户名", FieldType::TextField)
            .with_validator(LengthValidator::new(3, 20))
            .with_required(true);

        assert_eq!(field.validators.len(), 2);
    }

    #[test]
    fn test_form_field_validate_required() {
        let mut field = FormField::new("name", "姓名", FieldType::TextField).with_required(true);
        assert!(field.validate().is_err());
        assert!(field.error_message.is_some());

        field.value = "张三".to_string();
        assert!(field.validate().is_ok());
        assert!(field.error_message.is_none());
    }

    #[test]
    fn test_form_field_validate_length() {
        let mut field = FormField::new("code", "代码", FieldType::TextField)
            .with_validator(LengthValidator::new(3, 10))
            .with_required(true);

        field.value = "ab".to_string();
        assert!(field.validate().is_err());

        field.value = "abc".to_string();
        assert!(field.validate().is_ok());

        field.value = "abcdefghijk".to_string();
        assert!(field.validate().is_err());
    }

    #[test]
    fn test_form_add_field() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField));
        assert_eq!(form.fields.len(), 1);
    }

    #[test]
    fn test_form_remove_field() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField));
        form.add_field(FormField::new("email", "邮箱", FieldType::TextField));

        form.remove_field("name");
        assert_eq!(form.fields.len(), 1);
        assert_eq!(form.fields[0].id, "email");
    }

    #[test]
    fn test_form_get_field() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField));

        let field = form.get_field("name");
        assert!(field.is_some());
        assert_eq!(field.unwrap().id, "name");

        let none_field = form.get_field("invalid");
        assert!(none_field.is_none());
    }

    #[test]
    fn test_form_set_field_value() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField));

        form.set_field_value("name", "张三");
        assert_eq!(form.get_field("name").unwrap().value, "张三");
    }

    #[test]
    fn test_form_validate_all() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField).with_required(true));
        form.add_field(FormField::new("email", "邮箱", FieldType::TextField).with_required(true));

        let result = form.validate_all();
        assert!(result.is_err());

        form.set_field_value("name", "张三");
        form.set_field_value("email", "user@example.com");

        let result = form.validate_all();
        assert!(result.is_ok());
    }

    #[test]
    fn test_form_is_valid() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField).with_required(true));

        assert!(!form.is_valid());

        form.set_field_value("name", "张三");
        assert!(form.is_valid());
    }

    #[test]
    fn test_form_handle_navigation() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField));
        form.add_field(FormField::new("email", "邮箱", FieldType::TextField));

        form.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Down,
            KeyModifiers::NONE,
        ))));
        assert_eq!(form.selected_field, 1);

        form.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Up,
            KeyModifiers::NONE,
        ))));
        assert_eq!(form.selected_field, 0);
    }

    #[test]
    fn test_form_handle_tab_navigation() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField));
        form.add_field(FormField::new("email", "邮箱", FieldType::TextField));

        form.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::NONE,
        ))));
        assert_eq!(form.selected_field, 1);

        form.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::NONE,
        ))));
        assert_eq!(form.selected_field, 0);
    }

    #[test]
    fn test_form_handle_char_input() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField));

        form.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('张'),
            KeyModifiers::NONE,
        ))));
        form.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('三'),
            KeyModifiers::NONE,
        ))));

        assert_eq!(form.get_field("name").unwrap().value, "张三");
    }

    #[test]
    fn test_form_handle_backspace() {
        let mut form = Form::new();
        form.add_field(FormField::new("name", "姓名", FieldType::TextField));
        form.set_field_value("name", "张三");

        form.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Backspace,
            KeyModifiers::NONE,
        ))));
        assert_eq!(form.get_field("name").unwrap().value, "张");
    }

    #[test]
    fn test_form_boolean_field_toggle() {
        let mut form = Form::new();
        form.add_field(FormField::new("active", "启用", FieldType::BooleanField));

        form.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::NONE,
        ))));
        assert_eq!(form.get_field("active").unwrap().value, "true");

        form.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::NONE,
        ))));
        assert_eq!(form.get_field("active").unwrap().value, "false");
    }

    #[test]
    fn test_form_with_callbacks() {
        let submit_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancel_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let submit_clone = submit_called.clone();
        let cancel_clone = cancel_called.clone();

        let mut form = Form::new()
            .with_submit(move |_| {
                submit_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                ActionResult::Handled
            })
            .with_cancel(move || {
                cancel_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                ActionResult::Handled
            });

        form.submit();
        assert!(submit_called.load(std::sync::atomic::Ordering::SeqCst));

        form.cancel();
        assert!(cancel_called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_form_field_clear_error() {
        let mut field = FormField::new("name", "姓名", FieldType::TextField).with_required(true);
        field.validate();
        assert!(field.error_message.is_some());

        field.clear_error();
        assert!(field.error_message.is_none());
    }

    #[test]
    fn test_form_field_disabled() {
        let mut field = FormField::new("name", "姓名", FieldType::TextField).with_disabled(true);
        field.validate();
        assert!(field.validate().is_ok());
    }

    #[test]
    fn test_form_select_field() {
        let field = FormField::new("role", "角色", FieldType::SelectField)
            .with_options(vec![
                "管理员".to_string(),
                "用户".to_string(),
                "访客".to_string(),
            ])
            .with_value("1");

        assert_eq!(field.options.len(), 3);
        assert_eq!(field.value, "1");
    }

    #[test]
    fn test_form_id_generation() {
        let form = Form::new();
        assert!(form.get_state().id.starts_with("form_"));
    }

    #[test]
    fn test_form_state_update() {
        let mut form = Form::new();
        form.on_focus(true);
        assert!(form.state.focused);

        let area = Rect::new(10, 10, 80, 20);
        form.on_resize(area);
        assert_eq!(form.state.bounds, area);
    }
}
