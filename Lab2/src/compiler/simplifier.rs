use crate::compiler::tokenizer::{Token, TokenType};
use thiserror::Error;

#[derive(Debug)]
pub struct Simplifier {
    tokens: Vec<Token>,
    current_index: usize,
}

impl Simplifier {
    pub fn new(tokens: Vec<Token>) -> Self {
        Simplifier {
            tokens,
            current_index: 0,
        }
    }

    pub fn simplify(&mut self) -> Result<Vec<Token>, SimplifierError> {
        let mut tokens: Vec<Token> = Vec::new();

        while self.current_index < self.tokens.len() {
            let token = &self.tokens[self.current_index];

            match &token.kind {
                TokenType::Number => {
                    let mut number = token
                        .value
                        .clone()
                        .ok_or(SimplifierError::NumberTokenWithoutValue)?;
                    let mut position = token.position.clone();
                    let mut index_after = 1;

                    if let Some(possible_dot) = self.peek_next()
                        && possible_dot.kind == TokenType::Dot
                        && let Some(fractional_part_token) = self.peek_next_by(2)
                        && fractional_part_token.kind == TokenType::Number
                        && let Some(fractional_part) = &fractional_part_token.value
                    {
                        let float = format!("{}.{}", number, fractional_part);
                        number = float;
                        index_after += 2;
                        position.end = fractional_part_token.position.end;
                    }

                    self.current_index += index_after;
                    tokens.push(Token {
                        kind: TokenType::Number,
                        position,
                        value: Some(number),
                    });
                },
                _ => {
                    self.current_index += 1;
                    tokens.push(token.clone());
                },
            }
        }

        Ok(tokens)
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.current_index + 1)
    }

    fn peek_next_by(&self, by: usize) -> Option<&Token> {
        self.tokens.get(self.current_index + by)
    }
}

#[derive(Debug, Error)]
pub enum SimplifierError {
    #[error("Number token is missing a value")]
    NumberTokenWithoutValue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::tokenizer;
    use crate::token;

    #[test]
    fn test_1() {
        let code = "a + b + c - 4.5";
        let tokens = tokenizer::tokenize(code);
        let simplifier = Simplifier::new(tokens).simplify();
        assert!(simplifier.is_ok());
        let simplified_tokens = simplifier.unwrap();
        let expected_tokens = vec![
            token!(TokenType::Identifier, "a".to_string(), 0),
            token!(TokenType::Plus, 2),
            token!(TokenType::Identifier, "b".to_string(), 4),
            token!(TokenType::Plus, 6),
            token!(TokenType::Identifier, "c".to_string(), 8),
            token!(TokenType::Minus, 10),
            token!(TokenType::Number, "4.5".to_string(), 12..15),
        ];
        assert_eq!(expected_tokens, simplified_tokens);
    }
}
