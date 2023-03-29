mod hinter;
mod symbols;
mod completer;
mod highlighter;

use std::sync::{Mutex, Arc};

use reedline::{DefaultPrompt, Reedline, Signal, Prompt, DefaultPromptSegment, DefaultValidator, Keybindings, KeyModifiers, ReedlineEvent, KeyCode, default_emacs_keybindings, Emacs, ColumnarMenu, ReedlineMenu};
use yansi::{Color, Style};

use crate::{codegen::jit::Jit, config::Config};

use self::{hinter::EzHinter, symbols::Symbols, completer::EzCompleter, highlighter::EzHighlighter};

lazy_static! {
    static ref BOLD: Style = Style::new(Color::Fixed(7)).bold();
    static ref DIMMED: Style = Style::new(Color::Fixed(7)).dimmed();

    static ref BANNER: String = {
        let banner_style = Style::new(Color::Green).bold();
        let line_style = Style::new(Color::Green).dimmed(); 
        let text_style = Style::new(Color::Fixed(7)).dimmed();

        let banner = r#"
        _____ ____ 
       /  __//_   \
       |  \   /   /
       |  /_ /   /_
       \____\\____/
                   
       "#;

       let line = "- - - - - - - - - - - - -";

       let version_text = format!("[EZ JIT {}]", env!("CARGO_PKG_VERSION"));

       let help_text = "Type .help for cookies";

        format!(
            "{}\n      {}\n{}\n  {}", 
            banner_style.paint(banner),
            text_style.paint(version_text),
            line_style.paint(line),
            text_style.paint(help_text)
        )
    };

    static ref HELP_MESSAGE: String = {
        let style = Style::new(Color::Fixed(7)).bold();

        format!(r#"
        Ha! I fooled you! You are not getting my 🍪, here's a help message instead:

          - Type {} or {} to display the value of a variable without pushing it on the stack
          - Type {} or {} to toggle the printing the stack
          - Type {} or {} to toggle the {} options, for debugging mainly
          - Type {} or {} to display this message (amazing, right?)

        To see a list of autocompletion-options, press {}

        "#,
        style.paint(".i"), style.paint(".inspect"),
        style.paint(".s"), style.paint(".silent"),
        style.paint(".e"), style.paint(".emit"), style.paint("--emit-*"),
        style.paint(".h"), style.paint(".help"),
        style.paint("<TAB>")
        )
    };
}

pub struct Repl {
    line_editor: Reedline,
    jit: Jit,
    current_symbols: Arc<Mutex<Symbols>>,
    config: Config,
    prompt: Box<dyn Prompt>,
    silent: bool
}

impl Repl {
    pub fn new(config: Config) -> Self {
        let jit = Jit::new();

        let symbols = Symbols::new(jit.defined_symbols());
        let current_symbols = Arc::new(Mutex::new(symbols));

        let hinter = EzHinter::new(current_symbols.clone());
        let completer = EzCompleter::new(current_symbols.clone());
        let highlighter = EzHighlighter {};
        let validator = DefaultValidator {};
        let edit_mode = Emacs::new(Self::default_keybindings());

        let completion_menu = ColumnarMenu::default().with_name("completion_menu");

        let line_editor = Reedline::create()
            .with_hinter(Box::new(hinter))
            .with_completer(Box::new(completer))
            .with_highlighter(Box::new(highlighter))
            .with_validator(Box::new(validator))
            .with_edit_mode(Box::new(edit_mode))
            .with_menu(ReedlineMenu::EngineCompleter(Box::new(completion_menu)));

        let prompt = Self::default_prompt();

        Repl {
            line_editor,
            jit,
            current_symbols,
            config,
            prompt,
            silent: false
        }
    }

    pub fn start(&mut self){
        println!("{}", *BANNER);
    
        loop {
            let sig = self.line_editor.read_line(self.prompt.as_ref());
            
            match sig {
                Ok(Signal::Success(buffer)) if buffer.trim().is_empty() => 
                    continue,

                Ok(Signal::Success(buffer)) if buffer.starts_with('.') => 
                    self.handle_command(buffer),

                Ok(Signal::Success(buffer)) => 
                    self.run(buffer),

                Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                    println!("\n{}", DIMMED.paint("kthxbye"));
                    break;
                },

                _ => continue
            }
        }
    }

    fn run(&mut self, buffer: String) {
        match self.jit.run_saving(buffer.clone(), &self.config.debug_config) {
            Ok(_) => {
                if !self.silent {
                    let state = self.jit.jit_state();
                    println!("|{}|", DIMMED.paint(state));
                }

                let mut symbols = self.current_symbols.lock().unwrap();
                *symbols = Symbols::new(self.jit.defined_symbols());
            },

            Err(err) => 
                err.report(buffer),
        }
    }

    fn handle_command(&mut self, buffer: String){
        let line: Vec<&str> = buffer.split_whitespace().collect();

        let (command, args) = line.split_first().unwrap();

        match *command {
            ".help" | ".h" =>
                println!("{}", *HELP_MESSAGE),

            ".emit" | ".e" => {
                if args.is_empty(){
                    println!(
                        "Please use one of the following options: {}", 
                        BOLD.paint("tokens ast clif asm")
                    );
                    return
                }

                let mut config = &mut self.config.debug_config;

                if args.contains(&"tokens"){
                    config.emit_tokens = !config.emit_tokens;
                    println!("Emiting Tokens: {}", BOLD.paint(config.emit_tokens))
                }

                if args.contains(&"ast"){
                    config.emit_ast = !config.emit_ast;
                    println!("Emiting AST: {}", BOLD.paint(config.emit_ast))
                }

                if args.contains(&"clif"){
                    config.emit_clif = !config.emit_clif;
                    println!("Emiting CLIF: {}", BOLD.paint(config.emit_clif))
                }

                if args.contains(&"asm"){
                    config.emit_asm = !config.emit_asm;
                    println!("Emiting ASM: {}", BOLD.paint(config.emit_asm))
                }
            },

            ".silent" | ".s" => {
                self.silent = !self.silent;
                println!("Stack printing: {}", BOLD.paint(!self.silent))
            },

            ".inspect" | ".i" => {
                todo!()
            },

            _ => todo!()
        }
    }

    fn default_prompt() -> Box<dyn Prompt> {
        Box::new(DefaultPrompt::new(
            DefaultPromptSegment::Basic("ez".to_string()),
            DefaultPromptSegment::CurrentDateTime
        ))
    }

    fn default_keybindings() -> Keybindings {
        let mut keybindings = default_emacs_keybindings();

        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu("completion_menu".to_string()),
                ReedlineEvent::MenuNext,
            ]),
        );

        keybindings
    }
}