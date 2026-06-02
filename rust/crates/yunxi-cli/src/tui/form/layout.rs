#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormLayout {
    Vertical,
    Horizontal,
    Grid { columns: usize },
    Tabbed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct FormLayoutConfig {
    pub layout: FormLayout,
    pub alignment: FormAlignment,
    pub field_spacing: u16,
    pub section_spacing: u16,
    pub label_width: u16,
}

impl Default for FormLayoutConfig {
    fn default() -> Self {
        Self {
            layout: FormLayout::Vertical,
            alignment: FormAlignment::Left,
            field_spacing: 1,
            section_spacing: 2,
            label_width: 20,
        }
    }
}

impl FormLayoutConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_layout(mut self, layout: FormLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_alignment(mut self, alignment: FormAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_field_spacing(mut self, spacing: u16) -> Self {
        self.field_spacing = spacing;
        self
    }

    pub fn with_section_spacing(mut self, spacing: u16) -> Self {
        self.section_spacing = spacing;
        self
    }

    pub fn with_label_width(mut self, width: u16) -> Self {
        self.label_width = width;
        self
    }

    pub fn vertical() -> Self {
        Self::default().with_layout(FormLayout::Vertical)
    }

    pub fn horizontal() -> Self {
        Self::default().with_layout(FormLayout::Horizontal)
    }

    pub fn grid(columns: usize) -> Self {
        Self::default().with_layout(FormLayout::Grid { columns })
    }

    pub fn tabbed() -> Self {
        Self::default().with_layout(FormLayout::Tabbed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_layout_config_default() {
        let config = FormLayoutConfig::default();
        assert_eq!(config.layout, FormLayout::Vertical);
        assert_eq!(config.alignment, FormAlignment::Left);
        assert_eq!(config.field_spacing, 1);
        assert_eq!(config.section_spacing, 2);
        assert_eq!(config.label_width, 20);
    }

    #[test]
    fn test_form_layout_config_new() {
        let config = FormLayoutConfig::new();
        assert_eq!(config.layout, FormLayout::Vertical);
        assert_eq!(config.alignment, FormAlignment::Left);
    }

    #[test]
    fn test_form_layout_config_with_layout() {
        let config = FormLayoutConfig::new().with_layout(FormLayout::Horizontal);
        assert_eq!(config.layout, FormLayout::Horizontal);
    }

    #[test]
    fn test_form_layout_config_with_alignment() {
        let config = FormLayoutConfig::new().with_alignment(FormAlignment::Center);
        assert_eq!(config.alignment, FormAlignment::Center);
    }

    #[test]
    fn test_form_layout_config_with_field_spacing() {
        let config = FormLayoutConfig::new().with_field_spacing(3);
        assert_eq!(config.field_spacing, 3);
    }

    #[test]
    fn test_form_layout_config_with_section_spacing() {
        let config = FormLayoutConfig::new().with_section_spacing(4);
        assert_eq!(config.section_spacing, 4);
    }

    #[test]
    fn test_form_layout_config_with_label_width() {
        let config = FormLayoutConfig::new().with_label_width(25);
        assert_eq!(config.label_width, 25);
    }

    #[test]
    fn test_form_layout_config_vertical() {
        let config = FormLayoutConfig::vertical();
        assert_eq!(config.layout, FormLayout::Vertical);
    }

    #[test]
    fn test_form_layout_config_horizontal() {
        let config = FormLayoutConfig::horizontal();
        assert_eq!(config.layout, FormLayout::Horizontal);
    }

    #[test]
    fn test_form_layout_config_grid() {
        let config = FormLayoutConfig::grid(3);
        assert_eq!(config.layout, FormLayout::Grid { columns: 3 });
    }

    #[test]
    fn test_form_layout_config_tabbed() {
        let config = FormLayoutConfig::tabbed();
        assert_eq!(config.layout, FormLayout::Tabbed);
    }

    #[test]
    fn test_form_layout_config_chained() {
        let config = FormLayoutConfig::new()
            .with_layout(FormLayout::Grid { columns: 2 })
            .with_alignment(FormAlignment::Right)
            .with_field_spacing(2)
            .with_section_spacing(3)
            .with_label_width(30);

        assert_eq!(config.layout, FormLayout::Grid { columns: 2 });
        assert_eq!(config.alignment, FormAlignment::Right);
        assert_eq!(config.field_spacing, 2);
        assert_eq!(config.section_spacing, 3);
        assert_eq!(config.label_width, 30);
    }

    #[test]
    fn test_form_layout_partial_eq() {
        let config1 = FormLayoutConfig::new().with_layout(FormLayout::Horizontal);
        let config2 = FormLayoutConfig::new().with_layout(FormLayout::Horizontal);
        assert_eq!(config1.layout, config2.layout);
    }

    #[test]
    fn test_form_layout_clone() {
        let config = FormLayoutConfig::new()
            .with_layout(FormLayout::Grid { columns: 2 })
            .with_alignment(FormAlignment::Center);

        let cloned = config.clone();
        assert_eq!(config.layout, cloned.layout);
        assert_eq!(config.alignment, cloned.alignment);
    }

    #[test]
    fn test_form_alignment_variants() {
        assert_eq!(FormAlignment::Left, FormAlignment::Left);
        assert_eq!(FormAlignment::Center, FormAlignment::Center);
        assert_eq!(FormAlignment::Right, FormAlignment::Right);
        assert_ne!(FormAlignment::Left, FormAlignment::Right);
    }

    #[test]
    fn test_form_layout_variants() {
        assert_eq!(FormLayout::Vertical, FormLayout::Vertical);
        assert_eq!(FormLayout::Horizontal, FormLayout::Horizontal);
        assert_eq!(FormLayout::Tabbed, FormLayout::Tabbed);
        assert_eq!(
            FormLayout::Grid { columns: 2 },
            FormLayout::Grid { columns: 2 }
        );
        assert_ne!(FormLayout::Vertical, FormLayout::Horizontal);
    }
}
