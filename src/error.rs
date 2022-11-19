use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::error::SimpleReason::*;
use chumsky::prelude::*;
use yansi::Paint;

use std::ops::Range;
use std::fmt::Debug;

pub type TErr = Vec<Simple<char>>;

#[derive(Debug, Clone)]
pub struct Spaned<T: Debug + Clone + PartialEq>(T, Range<usize>);

pub fn adhoc_error<M: ToString, T>(msg: M) -> Result<T, TErr> {
    Err(vec![ Simple::custom(
        0..1, 
        msg
    )])
}

impl<T: Debug + Clone + PartialEq> Spaned<T> {
    pub fn new(content: T, range: Range<usize>) -> Self {
        Spaned(content, range)
    }

    pub fn content(&self) -> &T {
        &self.0
    }

    pub fn range(&self) -> &Range<usize> {
        &self.1
    }

    pub fn err_with<M: ToString>(&self, msg: M) -> Result<Spaned<T>, TErr> {
        Err(vec![ Simple::custom(
            self.range().clone(),
            msg
        )])
    }
}

impl<T: Debug + Clone + PartialEq> PartialEq for Spaned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}


/*impl<T: Debug + Clone + PartialEq> Display for Spaned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}*/

pub fn report_errors(src: &str, errs: Vec<Simple<char>>) {
    errs.into_iter()
        .map(|e| e.map(|c| c.to_string()))
        .for_each(|e| {
            let report = Report::build(ReportKind::Error, (), e.span().start);

            let report = match e.reason() {
                Unclosed { span, delimiter } => report
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

                Unexpected => report
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
                                        let exp = escape(expected).to_string();
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

                Custom(msg) => report.with_message(msg).with_label(
                    Label::new(e.span())
                        .with_message(format!("{}", msg.fg(Color::Red)))
                        .with_color(Color::Red),
                ),
            };

            report.finish().print(Source::from(&src)).unwrap();
        });
}


fn escape(inp: &str) -> String {
    match inp {
        "\n" => "newline".to_string(),
        "\t" => "tab".to_string(),
        " " => "space".to_string(),
        _ => format!("'{inp}'")
    }
}