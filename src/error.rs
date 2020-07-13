use serde::de;
use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    EmptyRule,
    InternalError,
    InvalidExpression(usize, usize),
    InvalidName(usize, usize),
    InvalidRange(usize, usize),
    UnbalancedBrackets,
    UnknownCommand(usize, usize),
    ZeroLengthSubst(usize, usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::EmptyRule => write!(f, "Cannot create rule from empty string!"),
            ParseError::InvalidExpression(beg, end) => {
                write!(f, "Invalid expression ({}, {})", beg, end)
            }
            ParseError::InvalidName(beg, end) => write!(f, "Invalid name ({}, {})", beg, end),
            ParseError::UnbalancedBrackets => write!(f, "Unbalanced brackets in expr"),
            ParseError::UnknownCommand(beg, end) => write!(f, "Unknown command ({}, {})", beg, end),
            ParseError::InvalidRange(beg, end) => {
                write!(f, "Invalid range specification ({}, {})", beg, end)
            }
            ParseError::ZeroLengthSubst(beg, end) => {
                write!(f, "Zero-length substitution expression ({}, {})", beg, end)
            }
            // yes, unreachable... FOR NOW
            _ => write!(f, "Unknown error!"),
        }
    }
}

#[derive(Debug)]
pub enum AnnalsError {
    UnknownCognate { name: String },
    EmptyCognate { name: String },
    NoSuitableGroups { name: String, context: String },
    UnknownToken { content: String },
    UnboundVariable { name: String },
    SerdeError { msg: String },
    InvalidRule { err: ParseError, expr: String },
    IOError(std::io::Error),
    YAMLError(serde_yaml::Error),
    UnknownError,
}

impl AnnalsError {
    pub fn from_invalid_rule(expr: String, err: ParseError) -> Self {
        AnnalsError::InvalidRule { err, expr }
    }
}

impl std::convert::From<std::io::Error> for AnnalsError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}
impl std::convert::From<serde_yaml::Error> for AnnalsError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::YAMLError(err)
    }
}

/// Handler for transforming de::Error.
impl de::Error for AnnalsError {
    fn custom<T: Display>(msg: T) -> Self {
        AnnalsError::SerdeError {
            msg: msg.to_string(),
        }
    }
}

impl Display for AnnalsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AnnalsError::*;
        match self {
            UnknownCognate { name } => write!(f, "Unknown cognate: {}", name),
            EmptyCognate { name } => write!(f, "No groups in cognate: {}", name),
            NoSuitableGroups { name, context } => {
                write!(f, "No suitable groups for {} in context: {}", name, context)
            }
            UnknownToken { content } => write!(f, "Unknown token: {}", content),
            UnboundVariable { name } => write!(f, "Unbound variable: {}", name),
            SerdeError { msg } => write!(f, "{}", msg),
            InvalidRule { err, expr } => format_invalid_rule(f, err, expr),
            UnknownError => write!(f, "Unknown error"),
            IOError(err) => write!(f, "{}", err),
            YAMLError(err) => write!(f, "{}", err),
        }
    }
}

/// Format ParseError emitted by an invalid rule.
/// Because these will be run through serde, we prepend a newline so that errors
/// will end up looking something like:
/// ```bash
/// .[0].groups[0]:
/// <)>
///  ^--Invalid name at (1, 2) at line 5 column 10
/// ```
fn format_invalid_rule(f: &mut fmt::Formatter, err: &ParseError, expr: &str) -> fmt::Result {
    writeln!(f)?;
    match err {
        ParseError::EmptyRule => write!(f, "{}", err),
        ParseError::InternalError => {
            write!(f, "Unknown internal error while parsing:\n    {}", expr)
        }
        ParseError::InvalidExpression(beg, _end)
        | ParseError::InvalidName(beg, _end)
        | ParseError::InvalidRange(beg, _end)
        | ParseError::UnknownCommand(beg, _end)
        | ParseError::ZeroLengthSubst(beg, _end) => {
            writeln!(f, "{}", expr)?;
            write!(f, "{}^-- {}", " ".repeat(*beg), err)
        }
        ParseError::UnbalancedBrackets => write!(f, "Unbalanced brackets in rule:\n    {}", expr),
    }
}

impl Error for AnnalsError {}
