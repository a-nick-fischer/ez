use std::{sync::{Arc, Mutex}};

use reedline::{History, Hinter};
use weighted_trie::WeightedTrie;
use yansi::{Style, Color};

const MIN_CHARS: usize = 2;

lazy_static! {
    static ref HINTER_STYLE: Style = Style::new(Color::Fixed(7)).dimmed();
}

pub struct Symbols(WeightedTrie);

impl Symbols {
    pub fn new<'a>(new_symbols: impl Iterator<Item = &'a String>) -> Self {
        let mut trie = WeightedTrie::new();
        
        for symbol in new_symbols {
            trie.insert(symbol.clone(), 1);
        }

        Symbols(trie)
    }

    fn search(&self, prefix: &str) -> Vec<String> {
        self.0.search(prefix)
    }
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
        self.current_hint = if line.chars().count() >= MIN_CHARS {
            self.symbols
                .lock()
                .unwrap()
                .search(line)
                .get(0)
                .cloned()
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