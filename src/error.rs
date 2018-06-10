#[derive(Debug, Fail)]
pub enum AnnalsFailure {
    #[fail(display = "Unknown or unrecognised cognate: {}", name)]
    UnknownCognate {
        name: String,
    },
    #[fail(display = "No groups in cognate: {}", name)]
    EmptyCognate {
        name: String,
    },
    #[fail(display = "{}", error)]
    InvalidTemplate {
        template: String,
        error: String
    },
    #[fail(display = "No suitable groups for {} in context: {}", name, context)]
    NoSuitableGroups {
        name: String,
        context: String
    },
    #[fail(display = "Unknown token: {}", content)]
    UnknownToken {
        content: String,
    },

    #[fail(display = "Unbound variable: {}", name)]
    UnboundVariable {
        name: String
    }
}
