use std::{sync::{Arc, Mutex}};

use reedline::{History, Hinter};

use yansi::{Style, Color};

use super::symbols::Symbols;

lazy_static! {
    static ref HINTER_STYLE: Style = Style::new(Color::Fixed(7)).dimmed();
}

pub struct AutocompletionHinter {
    current_hint: String,
    symbols: Arc<Mutex<Symbols>>
}

impl AutocompletionHinter {
    pub fn new(symbols: Arc<Mutex<Symbols>>) -> Self {
        Self {
            current_hint: String::new(),
            symbols
        }
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
        let last_word = line.split_whitespace().last();

        self.current_hint = if let Some(word) = last_word {
            self.symbols
                .lock()
                .unwrap()
                .search(word)
                .first()
                .map(|completion|
                    completion.chars()
                        .skip(word.len())
                        .collect()
                )
                .unwrap_or_default()
        }
        else {
            String::new()
        };

        if use_ansi_coloring && !self.current_hint.is_empty() {
            HINTER_STYLE.paint(&self.current_hint).to_string()
        } 
        else {
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