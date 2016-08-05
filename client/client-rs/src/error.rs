use ::std::io;
use ::std::string;

#[derive(Debug)]
pub struct Error {
    pub source: &'static str,
    pub detail: Option<String>,
}

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error {
            source: "TODO: Io error",
            detail: None,
        }
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(_: string::FromUtf8Error) -> Error {
        Error {
            source: "TODO: FromUtf8 error",
            detail: None,
        }
    }
}
