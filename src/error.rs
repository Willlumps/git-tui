pub enum Error {
    // crossterm?
    Git(git2::Error),
    Io(std::io::Error),
    Unknown(String),
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
