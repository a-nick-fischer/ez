mod repl_helper;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use yansi::Paint;

use repl_helper::ReplHelper;
use crate::interpreter::Interpreter;
use crate::stdlib::STDLIB;

const HISTORY_FILE_PATH: &str = ".why_history";

pub fn repl() {
    let mut interpreter = Interpreter::new(&STDLIB);

    let helper = ReplHelper::new();
    let mut rl = Editor::new().expect("Could not create editor!");
    rl.set_helper(Some(helper));

    let p = "|>> ";
    rl.helper_mut().expect("No helper").colored_prompt(p);

    println!("{}", ReplHelper::banner());

    if rl.load_history(HISTORY_FILE_PATH).is_err() && rl.save_history(HISTORY_FILE_PATH).is_err(){
        println!("{}", Paint::yellow("\tNo permissions to create a history file, you gonna live without one").dimmed());
    }

    loop {
        let readline = rl.readline(&p);

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                interpreter.run(line);
            },

            Err(ReadlineError::Interrupted) => {
                println!("{}", Paint::green("\nkthxbye\n").dimmed());
                break
            },

            Err(ReadlineError::Eof) => break,
            
            _ => println!()
        }
    }

    if rl.save_history(HISTORY_FILE_PATH).is_err() {
        println!("{}", Paint::red("Welp, could not save history file."))
    }
}