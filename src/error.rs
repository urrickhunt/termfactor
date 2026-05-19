use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::process::ExitStatus;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    CommandFailed {
        program: String,
        status: ExitStatus,
        stderr: String,
    },
    Message(String),
    Quit,
}

impl AppError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    pub fn command_failed(
        program: impl Into<String>,
        status: ExitStatus,
        stderr: impl Into<String>,
    ) -> Self {
        Self::CommandFailed {
            program: program.into(),
            status,
            stderr: stderr.into(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::CommandFailed {
                program,
                status,
                stderr,
            } => {
                let stderr = stderr.trim();
                if stderr.is_empty() {
                    write!(formatter, "{program} exited with status {status}")
                } else {
                    write!(formatter, "{program} exited with status {status}\n{stderr}")
                }
            }
            Self::Message(message) => write!(formatter, "{message}"),
            Self::Quit => Ok(()),
        }
    }
}

impl Error for AppError {}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
