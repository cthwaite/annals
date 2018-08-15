use std::fmt::{self, Display};
use std::error::Error;
use serde::de;


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