pub fn error<M: ToString>(message: M) -> Error {
    Error::General { message: message.to_string() }
}

use std::{ops::Range};

use ariadne::{Report, ReportKind, Color, Label, Fmt, ReportBuilder, Source};
use chumsky::{prelude::Simple, error::SimpleReason};
use yansi::Paint;

use crate::{lexer::token::Token, parser::types::{typelist::TypeList, typ::Type}};

#[derive(Debug)]
pub enum Error {
    Lexer { 
        inner: Vec<Simple<char>> 
    },

    General {
        message: String,
    },

    VariableNotFound {
        token: Token
    },

    AssigmentEmptyStack {
        token: Token
    },

    Reassigment {
        token: Token
    },

    WrongTypeInList {
        token: Token,
        expected: Type,
        got: Type
    },

    UnificationError {
        token: Token,
        msg: String
    },

    WrongArguments {
        fname: String,
        token: Token,
        expected: TypeList,
        got: TypeList
    },

    IncompatibleFunctionReturn {
        token: Token,
        expected: TypeList,
        got: TypeList
    },
}


impl Error {
    pub fn report(&self, src: String) {
        let print = |r: ReportBuilder<_>| r
            .finish()
            .eprint(Source::from(src.clone()))
            .expect("Beeing able to print errors");

        match self {
            Error::Lexer { inner } => 
                inner
                    .iter()
                    .for_each(|lex_err| 
                        print(lexer_error_report(lex_err))),

            Error::General { message } => print(message_error_report(message.clone())),

            Error::VariableNotFound { token: Token::Ident { value, range } } | 
            Error::VariableNotFound { token: Token::GetIdent { value, range } } 
            => print(simple_error_report(
                range.clone(), 
                format!(
                    "The variable {} is not defined at this point",
                    value.fg(Color::Cyan)
                ),
                "this one".to_string()
            )),

            Error::AssigmentEmptyStack { token: Token::Assigment { value, range } } => print(simple_error_report(
                range.clone(), 
                format!(
                    "Cannot assign to {}, as the stack is empty at this point",
                    value.fg(Color::Cyan)
                ),
                "this one".to_string()
            )),

            Error::Reassigment { token: Token::Assigment { value, range } } => print(simple_error_report(
                range.clone(), 
                format!(
                    "Cannot assign to {}, as it already assigned a value at this point",
                    value.fg(Color::Cyan)
                ),
                "this one".to_string()
            )),

            Error::WrongTypeInList { token, expected, got } => print(simple_error_report(
                token.range().clone(), 
                format!(
                    "This list of type {} cannot contain value of type {}",
                    expected.fg(Color::Cyan),
                    got.fg(Color::Red)
                ),
                "somewhere in this list".to_string()
            )),

            Error::UnificationError { token, msg } => print(simple_error_report(
                token.range().clone(), 
                msg.clone(), 
                "here".to_string()
            )),

            Error::WrongArguments { fname, token, expected, got } => {
                let builder = simple_error_report(
                    token.range().clone(), 
                    format!(
                        "Function {} called with unexpected arguments",
                        fname.fg(Color::Cyan)
                    ),
                    "here".to_string()
                );

                print(add_stack_comparison(builder, expected, got));
            },

            Error::IncompatibleFunctionReturn { token, expected, got } => {
                let builder = simple_error_report(
                    token.range().clone(), 
                    "Return stack of function does not match its signature".to_string(),
                    "here".to_string()
                );

                print(add_stack_comparison(builder, expected, got));
            },

            _ => unimplemented!()
        }
    }
}

fn message_error_report<'a>(msg: String) -> ReportBuilder<'a, Range<usize>> {
    Report::build(ReportKind::Error, (), 0)
        .with_message(msg)
}

fn simple_error_report<'a>(range: Range<usize>, msg: String, label: String) -> ReportBuilder<'a, Range<usize>> {
    Report::build(ReportKind::Error, (), range.start)
        .with_message(msg)
        .with_label(
            Label::new(range)
                .with_message(label)
                .with_color(Color::Red)
        )
}

fn add_stack_comparison<'a>(builder: ReportBuilder<'a, Range<usize>>, expected: &TypeList, got: &TypeList) -> ReportBuilder<'a, Range<usize>> {
    builder
        .with_note(
            format!(
                "\n\tExpected:\n\t{}\n\n\tGot:\n\t{}",
                expected.fg(Color::Cyan),
                got.fg(Color::Red)
            )
        )
}

fn lexer_error_report(err: &Simple<char>) -> ReportBuilder<Range<usize>> {
    let e = err.clone().map(|c| c.to_string());
    let report = Report::build(ReportKind::Error, (), e.span().start);

    match e.reason() {
        SimpleReason::Unclosed { span, delimiter } => report
            .with_message(format!(
                "Unclosed delimiter {}",
                delimiter.fg(Color::Yellow)
            ))
            .with_label(
                Label::new(span.clone())
                    .with_message(format!(
                        "Unclosed delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_color(Color::Yellow),
            )
            .with_label(
                Label::new(e.span())
                    .with_message(format!(
                        "Must be closed before this {}",
                        e.found()
                            .unwrap_or(&"end of file".to_string())
                            .fg(Color::Red)
                    ))
                    .with_color(Color::Red),
            ),

        SimpleReason::Unexpected => report
            .with_message(format!(
                "{}, expected {}",
                if e.found().is_some() {
                    "Unexpected token in input"
                } else {
                    "Unexpected end of input"
                },
                if e.expected().len() == 0 {
                    "something else".to_string()
                } else {
                    e.expected()
                        .map(|expected| match expected {
                            Some(expected) => {
                                let exp = escape(expected);
                                format!("{}", Paint::green(exp))
                            },
                            None => Paint::green("end of input").to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ))
            .with_label(
                Label::new(e.span())
                    .with_message(format!(
                        "Unexpected token {}",
                        e.found()
                            .unwrap_or(&"end of file".to_string())
                            .fg(Color::Red)
                    ))
                    .with_color(Color::Red),
            ),

        SimpleReason::Custom(msg) => report.with_message(msg).with_label(
            Label::new(e.span())
                .with_message(format!("{}", msg.fg(Color::Red)))
                .with_color(Color::Red),
        ),
    }
}

fn escape(inp: &str) -> String {
    match inp {
        "\n" => "newline".to_string(),
        "\t" => "tab".to_string(),
        " " => "space".to_string(),
        _ => format!("'{inp}'")
    }
}