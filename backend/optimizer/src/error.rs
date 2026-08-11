use crate::result::ExecutionResult;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum OptimizerError {
    StaticEvaluationError(String, ExecutionResult),
    LLVMError(String),
}

impl Display for OptimizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizerError::StaticEvaluationError(msg, _) => {
                write!(f, "{msg}")
            }
            OptimizerError::LLVMError(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for OptimizerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
