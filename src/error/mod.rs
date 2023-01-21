mod error;

pub type Error = error::Error;

use ariadne::Source;

pub fn report_errors(src: String, errs: Vec<Error>){
    for err in errs {
        let source = Source::from(src.clone());
        err.report().eprint(source);
    }
}

pub fn error<M: ToString>(message: M) -> Error {
    Error::GeneralError { message: message.to_string() }
}
