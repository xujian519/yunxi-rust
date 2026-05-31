use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

const SPINNER_BRAILLE: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const SPINNER_DOTS: [&str; 10] = ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷", "⠿", "⡿"];
const SPINNER_LINE: [&str; 4] = ["|", "/", "—", "\\"];
const SPINNER_ARROW: [&str; 8] = ["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"];
const SPINNER_MOON: [&str; 8] = ["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpinnerStyle {
    Braille,
    Dots,
    Line,
    Arrow,
    Moon,
}

impl SpinnerStyle {
    fn frames(self) -> &'static [&'static str] {
        match self {
            SpinnerStyle::Braille => &SPINNER_BRAILLE,
            SpinnerStyle::Dots => &SPINNER_DOTS,
            SpinnerStyle::Line => &SPINNER_LINE,
            SpinnerStyle::Arrow => &SPINNER_ARROW,
            SpinnerStyle::Moon => &SPINNER_MOON,
        }
    }

    pub(crate) fn glyph(self, frame: usize) -> &'static str {
        let frames = self.frames();
        frames[frame % frames.len()]
    }
}

pub(crate) fn spinner_span(style: SpinnerStyle, frame: usize, label: &str) -> Span<'static> {
    let glyph = style.glyph(frame);
    Span::styled(
        format!("{glyph} {label}"),
        Style::default()
            .fg(Color::Indexed(183))
            .add_modifier(Modifier::ITALIC),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShimmerPhase {
    Active,
    Done,
    Failed,
}

pub(crate) fn shimmer_span(phase: ShimmerPhase, text: &str, frame: usize) -> Span<'static> {
    let (glyph, color): (&str, Color) = match phase {
        ShimmerPhase::Active => {
            let frames = &SPINNER_BRAILLE;
            (frames[frame % frames.len()], Color::Indexed(183))
        }
        ShimmerPhase::Done => ("✓", Color::Green),
        ShimmerPhase::Failed => ("✗", Color::Red),
    };

    Span::styled(format!("{glyph} {text}"), Style::default().fg(color))
}

pub(crate) fn progress_bar(label: &str, current: u32, total: u32) -> ratatui::text::Line<'static> {
    use ratatui::text::Span;

    let pct = if total > 0 {
        (current as f64 / total as f64 * 20.0) as usize
    } else {
        0
    };

    let filled = "█".repeat(pct);
    let empty = "░".repeat(20usize.saturating_sub(pct));

    ratatui::text::Line::from(vec![
        Span::styled(
            format!("{label} "),
            Style::default().fg(Color::Indexed(245)),
        ),
        Span::styled(filled, Style::default().fg(Color::Indexed(183))),
        Span::styled(empty, Style::default().fg(Color::Indexed(237))),
        Span::styled(
            format!(" {current}/{total}"),
            Style::default().fg(Color::Indexed(245)),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_glyph_cycles() {
        let style = SpinnerStyle::Braille;
        let g0 = style.glyph(0);
        let g10 = style.glyph(10);
        assert_eq!(g0, g10);
    }

    #[test]
    fn shimmer_done_shows_checkmark() {
        let span = shimmer_span(ShimmerPhase::Done, "completed", 0);
        let s = span.content.as_ref();
        assert!(s.contains('✓'));
    }

    #[test]
    fn progress_bar_fills_correctly() {
        let line = progress_bar("test", 5, 10);
        let s = line.to_string();
        assert!(s.contains("5/10"));
        assert!(s.contains('█'));
        assert!(s.contains('░'));
    }
}
