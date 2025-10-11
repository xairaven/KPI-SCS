use crate::compiler::tokenizer::{Token, TokenType};
use std::collections::VecDeque;

#[derive(Debug)]
pub struct SyntaxAnalyzer {
    tokens: Vec<Token>,

    errors: Vec<SyntaxError>,
    current_index: usize,
    parentheses_stack: VecDeque<Token>,
    status: Status,
}

#[derive(Debug)]
pub struct SyntaxError {
    pub token: Token,
    pub kind: SyntaxErrorKind,
}

macro_rules! syntax_error {
    ($kind:ident, $token:expr) => {
        SyntaxError {
            token: $token.clone(),
            kind: SyntaxErrorKind::$kind,
        }
    };
}

#[derive(Debug)]
pub enum SyntaxErrorKind {
    UnexpectedOperand,
    UnexpectedOperator,
    UnexpectedComma,
    UnexpectedDot,
    UnexpectedParenthesis,
    UnmatchedParenthesis,
    UnknownToken,
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self.kind {
            SyntaxErrorKind::UnexpectedOperand | SyntaxErrorKind::UnexpectedOperator => {
                let token = match &self.token.kind {
                    TokenType::Identifier(value) | TokenType::Number(value) => {
                        format!("'{}'", value)
                    },
                    _ => self.token.kind.display_type(),
                };

                let unexpected = match self.kind {
                    SyntaxErrorKind::UnexpectedOperand => "operand",
                    SyntaxErrorKind::UnexpectedOperator => "operator",
                    _ => unreachable!(),
                };
                format!(
                    "Unexpected {} {}. {}",
                    unexpected,
                    token,
                    self.token.display_position()
                )
            },
            SyntaxErrorKind::UnexpectedComma => {
                format!("Unexpected comma. {}", self.token.display_position())
            },
            SyntaxErrorKind::UnexpectedDot => {
                format!("Unexpected dot. {}", self.token.display_position())
            },
            SyntaxErrorKind::UnexpectedParenthesis => {
                format!("Unexpected parenthesis. {}", self.token.display_position())
            },
            SyntaxErrorKind::UnmatchedParenthesis => {
                format!("Unmatched parenthesis. {}", self.token.display_position())
            },
            SyntaxErrorKind::UnknownToken => {
                format!("Unknown token. {}", self.token.display_position())
            },
        };

        write!(f, "{}", text)
    }
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
            parentheses_stack: VecDeque::new(),
            status: Status::default(),
        }
    }

    pub fn analyze(mut self) -> Vec<SyntaxError> {
        self.status = Status {
            expect_operand: true,
            expect_operator: false,
            in_string: false,
        };

        while self.current_index < self.tokens.len() {
            let token = &self.tokens[self.current_index];

            // If we're not in the string, we can skip spaces, tabs, and newlines
            match &token.kind {
                TokenType::Space | TokenType::Tab | TokenType::NewLine
                    if !self.status.in_string =>
                {
                    self.current_index += 1;
                    continue;
                },
                _ => {},
            }

            match &token.kind {
                TokenType::QuotationMark => {
                    // Toggle string state.
                    if !self.status.in_string {
                        // Start mark. We're expecting an operand here.
                        if !self.status.expect_operand {
                            // If we didn't expect an operand, it's an error.
                            self.errors.push(syntax_error!(UnexpectedOperator, token));
                        }
                        self.status.in_string = true;
                        // While inside string we're considering that operand is not finished
                        self.status.expect_operand = false;
                        self.status.expect_operator = false;

                        self.current_index += 1;
                        continue;
                    } else {
                        // Closing mark
                        self.status.in_string = false;
                        // String literal is operand
                        self.status.expect_operand = false;
                        self.status.expect_operator = true;

                        self.current_index += 1;
                        continue;
                    }
                },

                _ if self.status.in_string => {
                    self.current_index += 1;
                    continue;
                },

                TokenType::Identifier(_) => {
                    // Identifier - operand
                    if !self.status.expect_operand {
                        self.errors.push(syntax_error!(UnexpectedOperand, token));
                        // Continuing, but considering that operand was read
                    }
                    self.status.expect_operand = false;
                    self.status.expect_operator = true;
                    self.current_index += 1;
                    continue;
                },

                TokenType::Number(_) => {
                    // Number - operand
                    if !self.status.expect_operand {
                        self.errors.push(syntax_error!(UnexpectedOperand, token));
                        // Continuing, but considering that operand was read
                    }

                    // Float validating
                    if let Some(next) = self.peek_next()
                        && next.kind == TokenType::Dot
                    {
                        if let Some(second) = self.peek_next_by(2) {
                            if matches!(&second.kind, TokenType::Number(_)) {
                                // Correct float! Number-Dot-Number
                                // Next token - the third
                                self.current_index += 3;
                                self.status.expect_operand = false;
                                self.status.expect_operator = true;
                                continue;
                            } else {
                                // Something else after dot - error
                                self.errors.push(syntax_error!(UnexpectedDot, second));
                                // Skipping number with the dot
                                self.current_index += 2;
                                self.status.expect_operand = false;
                                self.status.expect_operator = true;
                                continue;
                            }
                        } else {
                            // Dot in the end - error
                            self.errors.push(syntax_error!(UnexpectedOperator, next));
                            self.current_index += 2;
                            self.status.expect_operand = false;
                            self.status.expect_operator = true;
                            continue;
                        }
                    }

                    // Integer literal
                    self.current_index += 1;
                    self.status.expect_operand = false;
                    self.status.expect_operator = true;
                    continue;
                },

                TokenType::Dot => {
                    self.errors.push(syntax_error!(UnexpectedOperator, token));
                    self.current_index += 1;
                    continue;
                },

                // Binary mathematical operations
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Asterisk
                | TokenType::Slash
                | TokenType::Percent => {
                    if self.status.expect_operand {
                        // Not supporting unary operations
                        self.errors.push(syntax_error!(UnexpectedOperator, token));
                        // Waiting for operand still
                        self.current_index += 1;
                        continue;
                    } else {
                        // Correct binary operator
                        self.status.expect_operand = true;
                        self.status.expect_operator = false;
                        self.current_index += 1;
                        continue;
                    }
                },

                // Unary logical operations. Located where operand is expected
                TokenType::ExclamationMark | TokenType::Ampersand | TokenType::Pipe => {
                    if !self.status.expect_operand {
                        self.errors.push(syntax_error!(UnexpectedOperator, token));
                        self.current_index += 1;
                        continue;
                    } else {
                        // Correct! But still expecting operand
                        self.current_index += 1;
                        continue;
                    }
                },

                TokenType::LeftParenthesis => {
                    // LeftParenthesis can be there if we're waiting for operand (grouping)
                    // or previous token is Identifier (function call)
                    let allow = self.status.expect_operand
                        || matches!(self.peek_previous(), Some(t) if matches!(t.kind, TokenType::Identifier(_)));
                    if !allow {
                        self.errors.push(syntax_error!(UnmatchedParenthesis, token));
                        self.current_index += 1;
                        continue;
                    }

                    self.parentheses_stack.push_back(token.clone());
                    self.status.expect_operand = true;
                    self.status.expect_operator = false;
                    self.current_index += 1;
                    continue;
                },

                TokenType::RightParenthesis => {
                    if self.parentheses_stack.pop_back().is_some() {
                        // Correct
                        self.status.expect_operand = false;
                        self.status.expect_operator = true;
                    } else {
                        self.errors.push(syntax_error!(UnmatchedParenthesis, token));
                    }
                    self.current_index += 1;
                    continue;
                },

                TokenType::Comma => {
                    // Allowed only inside parentheses (function)
                    if self.parentheses_stack.is_empty() {
                        // Surely an error
                        self.errors.push(syntax_error!(UnexpectedComma, token));
                        self.current_index += 1;
                        continue;
                    } else {
                        // Inside parentheses comma need to be after operand and before new operand
                        if self.status.expect_operand {
                            // Empty argument
                            self.errors.push(syntax_error!(UnexpectedComma, token));
                        }
                        // Expecting new operand
                        self.status.expect_operand = true;
                        self.status.expect_operator = false;
                        self.current_index += 1;
                        continue;
                    }
                },

                TokenType::Unknown(_) => {
                    // Unknown — always an error
                    self.errors.push(syntax_error!(UnknownToken, token));
                    self.current_index += 1;
                    continue;
                },
                TokenType::Space | TokenType::Tab | TokenType::NewLine => continue,
            }
        }

        // Error for every unmatched left parenthesis
        for unmatched in self.parentheses_stack.into_iter().rev() {
            self.errors
                .push(syntax_error!(UnmatchedParenthesis, unmatched));
        }

        // If operand is expected in the end, it's the error.
        if let Some(last) = self.tokens.last()
            && self.status.expect_operand
        {
            self.errors.push(syntax_error!(UnexpectedOperand, last));
        }

        self.errors
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.current_index + 1)
    }

    fn peek_next_by(&self, by: usize) -> Option<&Token> {
        self.tokens.get(self.current_index + by)
    }

    fn peek_previous(&self) -> Option<&Token> {
        self.tokens.get(self.current_index - 1)
    }
}
