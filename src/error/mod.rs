mod error;

pub type Error = error::Error;

pub fn error<M: ToString>(message: M) -> Error {
    Error::GeneralError { message: message.to_string() }
}
