use nu_ansi_term::{Style, Color};
use reedline::{Highlighter, StyledText};

use crate::lexer::{lex, token::Token};

// This os the only place where we have to use nu_ansi_term 'cause
// reedline depends on it
lazy_static! {
    static ref DEFAULT_STYLE: Style = Style::new().fg(Color::White);
    static ref ASSIGMENT_STYLE: Style = Style::new().fg(Color::Cyan).bold();
    static ref GET_STYLE: Style = Style::new().fg(Color::LightBlue).bold();
    static ref STRING_STYLE: Style = Style::new().fg(Color::Green);
    static ref NUMBER_STYLE: Style = Style::new().fg(Color::Magenta);
}

pub struct EzHighlighter;

impl Highlighter for EzHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> reedline::StyledText {
        let mut text = StyledText::new();

        let mut tokens = if let Ok(tokens) = lex(line.to_owned()){
            tokens
        }
        else {
            text.push((*DEFAULT_STYLE, line.to_owned()));

            return text
        };

        tokens.sort_by(|a, b| 
            a.range().start.cmp(&b.range().start));

        // This will 100% fail for some unicode input..
        // but I don't care at the moment
        let mut i = 0;

        loop {
            // Basically a for-loop, but rust for-loops do not allow us
            // to change the index
            if i == line.len() { break; }

            // Retrieve the next token, but if there are none, add the
            // rest of the line with the default style to the buffer
            let token = if let Some(token) = tokens.first() {
                token
            }
            else {
                text.push((*DEFAULT_STYLE, line[i..].to_owned()));
                break;
            };

            let range = token.range();
            if i == range.start {
                // Handle if we actually found a token
                let style = token_to_style(token);
                text.push((style, line[i..range.end].to_owned()));

                i = range.end;
                tokens.remove(0);
            }
            else {
                // Handle if we didn't find a token (e.g. whitespace or smth)
                // Just add the text with the default style till the next token
                text.push((*DEFAULT_STYLE, line[i..range.start].to_owned()));

                i = range.start;
            }
        }
        
        text
    }
}

fn token_to_style(tok: &Token) -> Style {
    match tok {
        Token::Number { .. } => *NUMBER_STYLE,
        
        Token::Quote { .. } => *STRING_STYLE,
        
        Token::GetIdent { .. } => *GET_STYLE,
        
        Token::Assigment { .. } => *ASSIGMENT_STYLE,

        _ => *DEFAULT_STYLE,
    }
}