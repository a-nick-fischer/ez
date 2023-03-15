use reedline::{Highlighter, StyledText};

// This os the only place where we have to use nu_ansi_term 'cause
// reedline depends on it
lazy_static! {
    static ref DEFAULT_STYLE: Style = Style::new().fg(Color::White);
    static ref UNKNOWN_STYLE: Style = Style::new().fg(Color::Red);
    static ref CALL_STYLE: Style = Style::new().fg(Color::Cyan).bold();
    static ref STRING_STYLE: Style = Style::new().fg(Color::Green);
    static ref NUMBER_STYLE: Style = Style::new().fg(Color::Magenta);
}

pub struct EzHighlighter;

impl Highlighter for EzHighlighter {
    fn highlight(&self, line: &str, cursor: usize) -> reedline::StyledText {
        let text = StyledText::new();
        
        text
    }
}