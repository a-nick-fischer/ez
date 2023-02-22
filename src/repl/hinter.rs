use reedline::{History, Hinter};
use yansi::Style;


pub struct AutocompletionHinter {
    style: Style,
    current_hint: String,
    min_chars: usize,
    symbols: Vec<String>
}

impl AutocompletionHinter {
    fn new() -> Self {
        
    }
}

impl Hinter for AutocompletionHinter {
    fn handle(
        &mut self,
        line: &str,
        _pos: usize,
        _history: &dyn History,
        use_ansi_coloring: bool,
    ) -> String {
        self.current_hint = if line.chars().count() >= self.min_chars {
            
        } else {
            String::new()
        };

        if use_ansi_coloring && !self.current_hint.is_empty() {
            self.style.paint(&self.current_hint).to_string()
        } else {
            self.current_hint.clone()
        }
    }

    fn complete_hint(&self) -> String {
        self.current_hint.clone()
    }

    fn next_hint_token(&self) -> String {
        let mut reached_content = false;
        let result: String = self
            .current_hint
            .chars()
            .take_while(|c| match (c.is_whitespace(), reached_content) {
                (true, true) => false,
                (true, false) => true,
                (false, true) => true,
                (false, false) => {
                    reached_content = true;
                    true
                }
            })
            .collect();
        result
    }
}