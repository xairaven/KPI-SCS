use crate::compiler::tokenizer::{Token, TokenType};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum SyntaxError {
    ReservedOperand(Token),
    UnmatchedParenthesis(Token),
    UnexpectedOperator(Token),
    UnexpectedOperand(Token),
    UnknownOperand(Token),
}

#[derive(Default)]
pub struct Status {
    pub operand_expected: bool,
    pub operator_expected: bool,
    pub function_expected: bool,
    pub function_body: bool,
    pub string_expected: bool,
}

impl Status {
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub fn analyze(tokens: Vec<Token>) -> Vec<SyntaxError> {
    let mut errors: Vec<SyntaxError> = Vec::new();

    let mut status = Status::default();
    let mut parentheses_stack: VecDeque<&Token> = VecDeque::new();

    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            TokenType::Identifier(_) => {
                if status.operator_expected {
                    errors.push(SyntaxError::UnexpectedOperator(token.clone()));
                    continue;
                }

                status.function_expected = true;
                status.operator_expected = false;
            },
            TokenType::Number(_) => {
                if status.operator_expected {
                    errors.push(SyntaxError::UnexpectedOperator(token.clone()));
                    continue;
                }

                status.operator_expected = false;
            },
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Asterisk
            | TokenType::Slash
            | TokenType::Percent => {
                if status.operand_expected || status.function_expected {
                    errors.push(SyntaxError::UnexpectedOperator(token.clone()));
                    continue;
                }

                status.clear();
                status.operand_expected = true;
            },
            TokenType::LeftParenthesis => {
                parentheses_stack.push_front(token);
                if status.function_expected {
                    status.function_expected = false;
                    status.function_body = true;
                }
            },
            TokenType::RightParenthesis => {
                if let Some(_) = parentheses_stack.pop_front() {
                    if status.function_body {
                        status.function_body = false;
                        status.operator_expected = true;
                    }
                } else {
                    errors.push(SyntaxError::UnmatchedParenthesis(token.clone()));
                }
            },
            TokenType::ExclamationMark => {
                if status.operator_expected {
                    errors.push(SyntaxError::UnexpectedOperand(token.clone()));
                    continue;
                }
            },
            TokenType::Ampersand => {},
            TokenType::Pipe => {},
            TokenType::Dot => {},
            TokenType::Comma => {
                if !status.function_body {
                    errors.push(SyntaxError::UnexpectedOperand(token.clone()));
                    continue;
                }
            },
            TokenType::QuotationMark => {
                status.string_expected = !status.string_expected;
            },
            TokenType::Space | TokenType::Tab => {
                if !status.string_expected {
                    continue;
                }
            },
            TokenType::NewLine => {
                if status.function_body
                    || status.string_expected
                    || status.operand_expected
                {
                    errors.push(SyntaxError::UnexpectedOperand(token.clone()));
                    continue;
                }
            },
            TokenType::Unknown(_) => {
                errors.push(SyntaxError::UnknownOperand(token.clone()));
            },
        }
    }

    errors
}
