use thiserror::Error;
use std::fmt;

#[derive(Error, Debug, Clone)]
pub enum EolError {
    #[error("Lexer error at line {line}, column {column}: {message}")]
    Lexer { line: usize, column: usize, message: String },
    
    #[error("Parser error at line {line}, column {column}: {message}")]
    Parser { line: usize, column: usize, message: String },
    
    #[error("Semantic error at line {line}, column {column}: {message}")]
    Semantic { line: usize, column: usize, message: String },
    
    #[error("Code generation error: {0}")]
    CodeGen(String),
    
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("LLVM error: {0}")]
    Llvm(String),
}

pub type EolResult<T> = Result<T, EolError>;

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

pub fn lexer_error(line: usize, column: usize, message: impl Into<String>) -> EolError {
    EolError::Lexer {
        line,
        column,
        message: message.into(),
    }
}

pub fn parser_error(line: usize, column: usize, message: impl Into<String>) -> EolError {
    EolError::Parser {
        line,
        column,
        message: message.into(),
    }
}

pub fn semantic_error(line: usize, column: usize, message: impl Into<String>) -> EolError {
    EolError::Semantic {
        line,
        column,
        message: message.into(),
    }
}
