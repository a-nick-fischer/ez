//mod hinter;

use reedline::{DefaultPrompt, Reedline, Signal};

use crate::codegen::jit::Jit;

pub struct Repl {
    line_editor: Reedline,
    jit: Jit
}

impl Repl {
    pub fn new() -> Self {
        Repl {
            line_editor: Reedline::create(),
            jit: Jit::new()
        }
    }

    pub fn start(&mut self){
        let prompt = DefaultPrompt::default();
    
        loop {
            let sig = self.line_editor.read_line(&prompt);
            
            match sig {
                Ok(Signal::Success(buffer)) => {
                    println!("We processed: {}", buffer);
                },

                Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                    println!("\nAborted!");
                    break;
                },

                _ => continue
            }
        }
    }
}