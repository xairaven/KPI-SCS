use crate::compiler::tokenizer::Token;

#[derive(Debug)]
pub struct SyntaxAnalyzer {
    tokens: Vec<Token>,
    errors: Vec<SyntaxError>,
    current_index: usize,
}

#[derive(Debug)]
pub enum SyntaxError {
    UnexpectedOperand(Token),
    UnexpectedOperator(Token),
    UnmatchedParenthesis(Token),
}

#[derive(Debug, Default)]
pub struct Status {
    pub expect_operand: bool,
    pub expect_operator: bool,
    pub in_string: bool,
}

impl Status {
    pub fn clear(&mut self) {
        *self = Status::default();
    }
}

impl SyntaxAnalyzer {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            errors: Vec::new(),
            current_index: 0,
        }
    }

    pub fn analyze(self) -> Vec<SyntaxError> {
        self.errors
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.current_index + 1)
    }

    fn peek_previous(&self) -> Option<&Token> {
        self.tokens.get(self.current_index - 1)
    }
}
