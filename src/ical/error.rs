#[derive(thiserror::Error, Debug)]
pub enum ICalError {
    #[error("Error during the parse of the token given by LEX")]
    TokenParse,
    #[error("Error during the parse of a date")]
    DateParse,
    #[error("Unable to evaluate expression")]
    UnableEvaluateExpression,
}

pub type Result<T> = core::result::Result<T, ICalError>;
