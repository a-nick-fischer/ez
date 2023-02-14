//mod hinter;

use reedline::{DefaultPrompt, Reedline, Signal};

use crate::{codegen::jit::Jit, config::Config};

pub struct Repl<'a> {
    line_editor: Reedline,
    jit: Jit<'a>,
    config: Config
}

impl<'a> Repl<'a> {
    pub fn new(config: Config) -> Self {
        Repl {
            line_editor: Reedline::create(),
            jit: Jit::new(),
            config
        }
    }

    pub fn start(&mut self){
        let prompt = DefaultPrompt::default();
    
        loop {
            let sig = self.line_editor.read_line(&prompt);
            
            match sig {
                Ok(Signal::Success(buffer)) => self.run(buffer),

                Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                    println!("\nAborted!");
                    break;
                },

                _ => continue
            }
        }
    }

    fn run(&mut self, buffer: String) {
        match self.jit.run_saving(buffer.clone(), &self.config) {
            Ok(_) => {
                let state = self.jit.jit_state();

                println!("{state}")
            },

            Err(err) => 
                err.report(buffer),
        }
    }
}