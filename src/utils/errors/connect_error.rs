use std::fmt;

#[derive(Debug)]
pub struct ConnectError {
    inner: tungstenite::Error,
}

impl std::error::Error for ConnectError {}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write a descriptive message for your error
        write!(f, "Connection error occurred")
    }
}

impl From<ConnectError> for Box<dyn std::error::Error + Send> {
    fn from(err: ConnectError) -> Box<dyn std::error::Error + Send> {
        Box::new(err)
    }
}

impl ConnectError {
    pub fn new(err: tungstenite::Error) -> ConnectError {
        ConnectError { inner: err }
    }
}
