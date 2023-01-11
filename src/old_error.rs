use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::error::SimpleReason::*;
use chumsky::prelude::*;
use yansi::Paint;

use crate::lexer::token::Token;

pub type TErr = Vec<Simple<char>>;

pub fn err_to_str(err: TErr) -> String {
    err
        .first()
        .unwrap()
        .to_string()
}

impl Token {
    pub fn to_error<M: ToString>(&self, msg: M) -> TErr {
        vec![ Simple::custom(
            self.range().clone(),
            msg
        )]
    }
}

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