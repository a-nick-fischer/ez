mod repl_helper;

use std::collections::HashMap;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use yansi::Paint;

use repl_helper::ReplHelper;

use crate::error::report_errors;
use crate::lexer::lex;
use crate::parser::parse;
use crate::parser::types::type_env::TypeEnv;

const HISTORY_FILE_PATH: &str = ".why_history";

pub fn start() {
    let type_env = &mut TypeEnv::new(&HashMap::new());    

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
            Ok(ref line) => {
                rl.add_history_entry(line.as_str());
                
                let res = lex(line.clone())
                    .and_then(|tokens| parse(tokens, type_env));

                match res {
                    Ok(nodes) => println!("{nodes:?}"),
                    Err(errs) => report_errors(line.clone(), errs),
                }
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