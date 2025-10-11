use crate::compiler::tokenizer::{Token, TokenType};
use colored::Colorize;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct SyntaxAnalyzer {
    tokens: Vec<Token>,

    errors: Vec<SyntaxError>,
    current_index: usize,
    parentheses_stack: VecDeque<Token>,
    status: Status,
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum SyntaxErrorKind {
    UnexpectedFunctionName,
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
                    TokenType::Identifier | TokenType::Number => {
                        match &self.token.value {
                            None => format!("{}", "UNDEFINED".bright_purple().italic()),
                            Some(value) => format!("'{}'", value),
                        }
                    },
                    _ => self.token.kind.to_string(),
                };

                let unexpected = match self.kind {
                    SyntaxErrorKind::UnexpectedOperand => "operand",
                    SyntaxErrorKind::UnexpectedOperator => "operator",
                    _ => unreachable!(),
                };
                format!(
                    "{:30} {}",
                    format!("Unexpected {} {}.", unexpected, token).bold().red(),
                    self.token.display_position().bold()
                )
            },
            SyntaxErrorKind::UnexpectedFunctionName => {
                format!(
                    "{:30} {}",
                    format!(
                        "Unexpected function name \"{}\".",
                        match &self.token.value {
                            None => "<UNDEFINED>",
                            Some(value) => value,
                        }
                    ),
                    self.token.display_position().bold()
                )
            },
            SyntaxErrorKind::UnexpectedComma => {
                format!(
                    "{:30} {}",
                    "Unexpected comma.".bold().red(),
                    self.token.display_position().bold()
                )
            },
            SyntaxErrorKind::UnexpectedDot => {
                format!(
                    "{:30} {}",
                    "Unexpected dot.".bold().red(),
                    self.token.display_position().bold()
                )
            },
            SyntaxErrorKind::UnexpectedParenthesis => {
                format!(
                    "{:30} {}",
                    "Unexpected parenthesis.".bold().red(),
                    self.token.display_position().bold()
                )
            },
            SyntaxErrorKind::UnmatchedParenthesis => {
                format!(
                    "{:30} {}",
                    "Unmatched parenthesis.".bold().red(),
                    self.token.display_position().bold()
                )
            },
            SyntaxErrorKind::UnknownToken => {
                format!(
                    "{:30} {}",
                    "Unknown token.".bold().red(),
                    self.token.display_position().bold()
                )
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

                TokenType::Identifier => {
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

                TokenType::Number => {
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
                            if matches!(&second.kind, TokenType::Number) {
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
                    self.errors.push(syntax_error!(UnexpectedDot, token));
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
                        || matches!(self.peek_previous(), Some(t) if matches!(t.kind, TokenType::Identifier))
                        || matches!(self.peek_previous(), Some(t) if matches!(t.kind, TokenType::Number));
                    if !allow {
                        self.errors.push(syntax_error!(UnmatchedParenthesis, token));
                        self.current_index += 1;
                        continue;
                    }

                    if let Some(previous) = self.peek_previous()
                        && matches!(previous.kind, TokenType::Number)
                    {
                        // Function name cannot start with a number
                        self.errors
                            .push(syntax_error!(UnexpectedFunctionName, previous));
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

                TokenType::Unknown => {
                    // Unknown — always an error
                    self.errors.push(syntax_error!(UnknownToken, token));
                    self.current_index += 1;
                    continue;
                },
                TokenType::Space | TokenType::Tab | TokenType::NewLine => continue,
            }
        }

        // Error for every unmatched left parenthesis
        for unmatched in self.parentheses_stack.into_iter() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::tokenizer;

    macro_rules! test_error {
        ($error_kind:ident, $token_kind:expr, $position:literal) => {
            SyntaxError {
                token: Token {
                    kind: $token_kind,
                    position: $position..$position + 1,
                    value: None,
                },
                kind: SyntaxErrorKind::$error_kind,
            }
        };
        ($error_kind:ident, $token_kind:expr, $position:expr) => {
            SyntaxError {
                token: Token {
                    kind: $token_kind,
                    position: $position,
                    value: None,
                },
                kind: SyntaxErrorKind::$error_kind,
            }
        };
        ($error_kind:ident, $token_kind:expr, $position:literal, $value:expr) => {
            SyntaxError {
                token: Token {
                    kind: $token_kind,
                    position: $position..$position + 1,
                    value: Some($value),
                },
                kind: SyntaxErrorKind::$error_kind,
            }
        };
        ($error_kind:ident, $token_kind:expr, $position:expr, $value:expr) => {
            SyntaxError {
                token: Token {
                    kind: $token_kind,
                    position: $position,
                    value: Some($value),
                },
                kind: SyntaxErrorKind::$error_kind,
            }
        };
    }

    #[test]
    fn test_syntax_01() {
        let code = "-a ++ b - 2v*func((t+2 -, sin(x/*2.01.2), )/8(-)**";

        let mut errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        errors_actual.sort_by(|a, b| a.token.position.start.cmp(&b.token.position.start));
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(UnexpectedOperator, TokenType::Minus, 0),
            test_error!(UnexpectedOperator, TokenType::Plus, 4),
            test_error!(
                UnexpectedOperand,
                TokenType::Identifier,
                11,
                "v".to_string()
            ),
            test_error!(UnmatchedParenthesis, TokenType::LeftParenthesis, 17),
            test_error!(UnexpectedComma, TokenType::Comma, 24),
            test_error!(UnexpectedOperator, TokenType::Asterisk, 32),
            test_error!(UnexpectedDot, TokenType::Dot, 37),
            test_error!(UnexpectedOperand, TokenType::Number, 38, "2".to_string()),
            test_error!(
                UnexpectedFunctionName,
                TokenType::Number,
                44,
                "8".to_string()
            ),
            test_error!(UnexpectedOperator, TokenType::Minus, 46),
            test_error!(UnexpectedOperator, TokenType::Asterisk, 49),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

    //     #[test]
    //     fn test_syntax_02() {
    //         let code = "*a + nb -";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_03() {
    //         let code = "a ++ nb /* k -+/ g";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_04() {
    //         let code = "a^b$c - d#h + q%t + !b&(z|t)";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_05() {
    //         let code = "x + var1 + var_2 + _var_3 + var#4 + var!5
    // + 6var_ + $7 + ?8";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_06() {
    //         let code = "125 + 2nb - 0xAB * 0x0R + 0b010 * 0b20+ ABh * 0Rh + 010b*20b";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_07() {
    //         let code = "0.71/0.72.3 + .3 + 127.0.0.1*8. + 6.07ab - 9f.89hgt";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_08() {
    //         let code = ")a+b( -(g+h)(g-k))*()) + (-b(t-2*x*(5) + A[7][2-x]";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_09() {
    //         let code = "2(t) - f2(t) + g()/h(2, )*func(-t/q, f(4-t), - (x+2)*(y-2))";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_10() {
    //         let code = "/a*b**c + m)*a*b + a*c - a*smn(j*k/m + m";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_11() {
    //         let code =
    //             "-cos(-&t))/(*(*f)(127.0.0.1, \"/dev/null\", (t==0)?4more_errors:b^2) - .5";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_12() {
    //         let code = "//(*0)- an*0p(a+b)-1.000.5//6(*f(-b, 1.8-0*(2-6) %1 + (++a)/(6x^2+4x-1) + d/dt*(smn(at+q)/(4cos(at)-ht^2)";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_13() {
    //         let code = "-(-5x((int*)exp())/t - 3.14.15k/(2x^2-5x-1)*y - A[N*(i++)+j]";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_14() {
    //         let code = "-(-exp(3et/4.0.2, 2i-1)/L + )((void*)*f()) + ((i++) + (++i/(i--))/k//) + 6.000.500.5";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_15() {
    //         let code = "**f(*k, -p+1, ))2.1.1 + 1.8q((-5x ++ i)";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_16() {
    //         let code = "/.1(2x^2-5x+7)-(-i)+ (j++)/0 - )(*f)(2, 7-x, )/q + send(-(2x+7)/A[j, i], 127.0.0.1 ) + )/";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
    //
    //     #[test]
    //     fn test_syntax_17() {
    //         let code =
    //             "*101*1#(t-q)(t+q)//dt - (int*)f(8t, -(k/h)A[i+6.]), exp(), ))(t-k*8.00.1/.0";
    //
    //         let errors_actual: Vec<SyntaxError> =
    //             SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
    //         let errors_expected: Vec<SyntaxError> = vec![];
    //         assert_eq!(errors_actual, errors_expected);
    //     }
}
