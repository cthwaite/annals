use std::fmt::{self, Display};
use std::error::Error;
use serde::de;


#[derive(Debug, Clone)]
pub enum ParseError {
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
            ParseError::InvalidExpression(beg, end) => {
                write!(f, "Invalid expression ({}, {})", beg, end)
            },
            ParseError::InvalidName(beg, end) => {
                write!(f, "Invalid name ({}, {})", beg, end)
            },
            ParseError::UnbalancedBrackets => write!(f, "Unbalanced brackets in expr"),
            ParseError::UnknownCommand(beg, end) => {
                write!(f, "Unknown command ({}, {})", beg, end)
            },
            ParseError::InvalidRange(beg, end) =>
                write!(f, "Invalid range specification ({}, {})", beg, end),
            ParseError::ZeroLengthSubst(beg, end) => {
                write!(f, "Zero-length substitution expression ({}, {})", beg, end)
            },
            // yes, unreachable... FOR NOW
            _ => write!(f, "Unknown error!")
        }
    }
}


#[derive(Clone, Debug)]
pub enum AnnalsFailure {
    UnknownCognate {
        name: String,
    },
    EmptyCognate {
        name: String,
    },
    NoSuitableGroups {
        name: String,
        context: String
    },
    UnknownToken {
        content: String,
    },
    UnboundVariable {
        name: String
    },
    SerdeError {
        msg: String
    },
    InvalidRule {
        err: ParseError,
        expr: String
    },
    UnknownError
}

impl AnnalsFailure {
    pub fn from_invalid_rule(expr: String, err: ParseError) -> Self {
        AnnalsFailure::InvalidRule {
            err,
            expr
        }
    }
}

/// Handler for transforming de::Error.
impl de::Error for AnnalsFailure {
    fn custom<T: Display>(msg: T) -> Self {
        AnnalsFailure::SerdeError{msg: msg.to_string()}
    }
}

impl Display for AnnalsFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnnalsFailure::UnknownCognate{name} => write!(formatter, "Unknown cognate: {}", name),
            AnnalsFailure::EmptyCognate{name} => write!(formatter, "No groups in cognate: {}", name),
            AnnalsFailure::NoSuitableGroups{name, context} => write!(formatter, "No suitable groups for {} in context: {}", name, context),
            AnnalsFailure::UnknownToken{content} => write!(formatter, "Unknown token: {}", content),
            AnnalsFailure::UnboundVariable{name} => write!(formatter, "Unbound variable: {}", name),
            AnnalsFailure::SerdeError{msg} => write!(formatter, "{}", msg),
            AnnalsFailure::InvalidRule{ err, expr } => format_invalid_rule(formatter, err, expr),
            AnnalsFailure::UnknownError => write!(formatter, "Unknown error")
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
fn format_invalid_rule(f: &mut fmt::Formatter, err: &ParseError, expr: &String) -> fmt::Result {
    write!(f, "\n")?;
    match err {
        ParseError::InternalError => {
            write!(f, "Unknown internal error while parsing:\n    {}", expr)
        },
        ParseError::InvalidExpression(beg, _end)
        | ParseError::InvalidName(beg, _end)
        | ParseError::InvalidRange(beg, _end)
        | ParseError::UnknownCommand(beg, _end)
        | ParseError::ZeroLengthSubst(beg, _end) => {
            write!(f, "{}\n", expr)?;
            write!(f, "{}^-- {}", " ".repeat(*beg), err)
        },
        ParseError::UnbalancedBrackets => {
            write!(f, "Unbalanced brackets in rule:\n    {}", expr)
        },
    }
}


impl Error for AnnalsFailure {
    /// This is soft-deprecated, so let Display do the heavy lifting.
    fn description(&self) -> &str {
        "Error"
    }
}
