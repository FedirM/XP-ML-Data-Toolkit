#[derive(Debug, Clone, PartialEq)]
pub enum UserInput {
    VALUE(String),
    EXPR(String),
}

impl UserInput {
    pub fn is_expr(&self) -> bool {
        match self {
            UserInput::EXPR(_) => true,
            UserInput::VALUE(_) => false,
        }
    }

    pub fn is_value(&self) -> bool {
        !self.is_expr()
    }
}
