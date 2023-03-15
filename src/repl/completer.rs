use std::{sync::{Mutex, Arc}};

use reedline::{Completer, DefaultCompleter};

use super::symbols::Symbols;

pub struct EzCompleter {
    symbols: Arc<Mutex<Symbols>>,
    inner: DefaultCompleter
}

impl EzCompleter {
    pub fn new(symbols: Arc<Mutex<Symbols>>) -> Self {
        Self {
            symbols,
            inner: DefaultCompleter::default()
        }
    }
}

impl Completer for EzCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<reedline::Suggestion> {
        let last = line.split_whitespace().last();

        if let Some(word) = last {
            let symbols = self.symbols.lock().unwrap();
            let defs: Vec<String> = symbols.search(word);
            self.inner.clear();
            self.inner.insert(defs);
    
            self.inner.complete(line, pos)
        }
        else {
            Vec::new()
        }        
    }
}