#[derive(Debug)]
pub struct CustomError {
    details: String,
}

impl CustomError {
    pub fn new(value: &str) -> Self {
        CustomError {
            details: value.to_owned(),
        }
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for CustomError {}
