// https://github.com/kkawakam/rustyline/blob/master/examples/example.rs
use std::borrow::Cow::{self, Borrowed, Owned};


use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline_derive::{Completer, Helper, Hinter, Validator};
use yansi::Paint;
use rand::seq::SliceRandom;

const WELCOME_MESSAGES: [&str; 5] = [
    "\t\tHey, you, you're finally awake",
    "\t    Crashing... Blazingly (🦀) fast (🚀)!",
    "\t🚀🦀🦀🦀🚀🦀🦀🦀🚀🦀🦀🚀🦀🚀🚀🚀🚀🦀🦀🦀🦀\n\t\t  🚀🚀🦀🚀🚀🦀🦀🦀🦀🦀🦀",
    "\tYou're probably wondering why you just started me.\n\tI do too.",
    "\tThis welcome messages are random... I ran out of ideas."
];

#[derive(Helper, Completer, Hinter, Validator)]
pub struct ReplHelper {
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
    colored_prompt: String,
}

impl ReplHelper {
    pub fn new() -> ReplHelper {
        ReplHelper {
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: "<No Prompt>".to_owned(),
            validator: MatchingBracketValidator::new(),
        }
    }

    pub fn colored_prompt(&mut self, prompt: &str) {
        self.colored_prompt = Paint::green(prompt).bold().to_string();
    }

    pub fn banner() -> String {
        let msg = WELCOME_MESSAGES.choose(&mut rand::thread_rng()).unwrap();

        let banner = r#"
                 _    _ _   ___   _____  _ 
                | |  | | | | \ \ / /__ \| |
                | |  | | |_| |\ V /   ) | |
                | |/\| |  _  | \ /   / /| |
                \  /\  / | | | | |  |_| |_|
                 \/  \/\_| |_/ \_/  (_) (_)
        "#;

        format!(
            "{}\n{}\n\n",
            Paint::green(banner).bold().to_string(),
            Paint::white(msg).dimmed()
        )
    }
}

impl Highlighter for ReplHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        let msg = Paint::white(hint).dimmed().to_string();
        Owned(msg)
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}