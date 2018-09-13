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
                write!(f, "Invalid expression at ({}, {})", beg, end)
            },
            ParseError::InvalidName(beg, end) => {
                write!(f, "Invalid name at ({}, {})", beg, end)
            },
            ParseError::UnbalancedBrackets => write!(f, "Unbalanced brackets in expr"),
            ParseError::UnknownCommand(beg, end) => {
                write!(f, "Unknown command at ({}, {})", beg, end)
            },
            ParseError::InvalidRange(beg, end) => write!(f, "Invalid range specification at ({}, {})", beg, end),
            ParseError::ZeroLengthSubst(beg, end) => {
                write!(f, "Zero-length substitution expression ({}, {})", beg, end)
            },
            // yes, unreachable... FOR NOW
            _ => write!(f, "Unknown error!")
        }
    }
}


#[derive(Debug, Clone)]
pub struct InvalidRule<'a> {
    err: ParseError,
    expr: &'a str,
}


impl<'a> InvalidRule<'a> {
    pub fn from_parse_error(expr: &'a str, err: ParseError) -> Self {
        InvalidRule {
            err,
            expr
        }
    }
}

impl<'a> fmt::Display for InvalidRule<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.err {
            ParseError::InvalidRange(beg, _end) => {
                write!(f, "{}", self.expr)?;
                write!(f, "{}^--{}", " ".repeat(beg), "Invalid range")
            },
            ParseError::UnbalancedBrackets => {
                write!(f, "Unbalanced brackets in `{}`", self.expr)
            },
            ParseError::InvalidExpression(beg, _end) => {
                write!(f, "{}", self.expr)?;
                write!(f, "{}^--{}", " ".repeat(beg), "Invalid expression")
            },
            _ => {
                write!(f, "{}\n", self.expr)?;
                write!(f, " ^ -- {}", self.err)
            }
        }
    }
}


#[derive(Debug)]
pub enum AnnalsFailure {
    UnknownCognate {
        name: String,
    },
    EmptyCognate {
        name: String,
    },
    InvalidTemplate {
        template: String,
        error: String
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

    ParsingError {
        msg: String
    },

    UnknownError
}

impl de::Error for AnnalsFailure {
    fn custom<T: Display>(msg: T) -> Self {
        AnnalsFailure::ParsingError{msg: msg.to_string()}
    }
}

impl Display for AnnalsFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnnalsFailure::UnknownCognate{name} => 
                write!(formatter, "No groups in cognate: {}", name),
            AnnalsFailure::EmptyCognate{name} => 
                write!(formatter, "No groups in cognate: {}", name),
            AnnalsFailure::InvalidTemplate{error, ..} =>
                write!(formatter, "{}", error),
            AnnalsFailure::NoSuitableGroups{name, context} =>
                write!(formatter, "No suitable groups for {} in context: {}", name, context),
            AnnalsFailure::UnknownToken{content} => 
                write!(formatter, "Unknown token: {}", content),
            AnnalsFailure::UnboundVariable{name} => 
                write!(formatter, "Unbound variable: {}", name),
            AnnalsFailure::ParsingError{msg} =>
                write!(formatter, "Parsing error: {}", msg),
            AnnalsFailure::UnknownError => write!(formatter, "Unknown error")
        }
    }
}

impl Error for AnnalsFailure {
    /// This is soft-deprecated, so let Display do the heavy lifting.
    fn description(&self) -> &str {
        "Error"
    }
}

impl From<ParseError> for AnnalsFailure {
    fn from(parse_err: ParseError) -> Self {
        AnnalsFailure::ParsingError{
            msg: format!("{:?}", parse_err)
        }
    }
}
