#[derive(Debug)]
pub enum Error {
    // crossterm?
    Git(git2::Error),
    Io(std::io::Error),
    Unknown(String),
}

impl Error {
    pub fn message(&self) -> String {
        match self {
            Self::Git(err) => err.message().to_string(),
            Self::Io(err) => err.kind().to_string(),
            Self::Unknown(message) => message.clone(),
        }
    }
}

impl From<git2::Error> for Error {
    fn from(err: git2::Error) -> Self {
        Self::Git(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::Unknown(message)
    }
}
