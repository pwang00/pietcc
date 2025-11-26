use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum CompilerError {
    NotImplementedError,
    LLVMError(String),
}

impl Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilerError::NotImplementedError => write!(f, "Feature not yet implemented"),
            CompilerError::LLVMError(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for CompilerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
