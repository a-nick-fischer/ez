mod hinter;

use std::sync::{Mutex, Arc};

use reedline::{DefaultPrompt, Reedline, Signal};

use crate::{codegen::jit::Jit, config::Config};

use self::hinter::{AutocompletionHinter, Symbols};

pub struct Repl {
    line_editor: Reedline,
    jit: Jit,
    current_symbols: Arc<Mutex<Symbols>>,
    config: Config,
}

impl Repl {
    pub fn new(config: Config) -> Self {
        let jit = Jit::new();

        let symbols = Symbols::new(jit.defined_symbols());

        let current_symbols = Arc::new(Mutex::new(symbols));
        let hinter = Box::new(AutocompletionHinter::new(current_symbols.clone()));

        let line_editor = Reedline::create().with_hinter(hinter);

        Repl {
            line_editor,
            jit,
            current_symbols,
            config
        }
    }

    pub fn start(&mut self){
        let prompt = DefaultPrompt::default();
    
        loop {
            let sig = self.line_editor.read_line(&prompt);
            
            match sig {
                Ok(Signal::Success(buffer)) if buffer.trim().is_empty() => 
                    continue,

                Ok(Signal::Success(buffer)) => 
                    self.run(buffer),

                Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                    println!("\nAborted!");
                    break;
                },

                _ => continue
            }
        }
    }

    fn run(&mut self, buffer: String) {
        match self.jit.run_saving(buffer.clone(), &self.config.debug_config) {
            Ok(_) => {
                let state = self.jit.jit_state();

                println!("{state}");

                let mut symbols = self.current_symbols.lock().unwrap();
                *symbols = Symbols::new(self.jit.defined_symbols());
            },

            Err(err) => 
                err.report(buffer),
        }
    }
}