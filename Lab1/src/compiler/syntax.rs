use crate::compiler::tokenizer::{Token, TokenType};
use colored::Colorize;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct SyntaxAnalyzer {
    tokens: Vec<Token>,

    errors: Vec<SyntaxError>,
    current_index: usize,
    parentheses_stack: VecDeque<Token>,
    brackets_stack: VecDeque<Token>,
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
    EmptyParentheses,
    EmptyBrackets,
    IncorrectVariableName,
    IncorrectFloat,
    IncorrectHexLiteral,
    IncorrectBinaryLiteral,
    MissingArgument,
    UnexpectedFunctionName,
    UnexpectedEndOfExpression,
    UnexpectedOperand,
    UnexpectedOperator,
    UnexpectedComma,
    UnexpectedDot,
    UnexpectedNewLine,
    UnexpectedParenthesis,
    UnexpectedBrackets,
    UnmatchedParenthesis,
    UnmatchedBrackets,
    UnknownToken,
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self.kind {
            SyntaxErrorKind::EmptyParentheses => "Empty function or grouping.",
            SyntaxErrorKind::EmptyBrackets => "Empty array access.",
            SyntaxErrorKind::IncorrectVariableName => "Incorrect variable name.",
            SyntaxErrorKind::IncorrectFloat => "Incorrect float.",
            SyntaxErrorKind::IncorrectHexLiteral => match &self.token.value {
                None => "Incorrect hexadecimal literal.",
                Some(value) => &format!("Incorrect hexadecimal literal '0{}'.", value),
            },
            SyntaxErrorKind::IncorrectBinaryLiteral => match &self.token.value {
                None => "Incorrect binary literal.",
                Some(value) => &format!("Incorrect binary literal '0{}'.", value),
            },
            SyntaxErrorKind::MissingArgument => "Missing function argument.",
            SyntaxErrorKind::UnexpectedOperand => match &self.token.value {
                None => "Unexpected operand.",
                Some(value) => &format!("Unexpected operand '{}'.", value),
            },
            SyntaxErrorKind::UnexpectedOperator => "Unexpected operator.",
            SyntaxErrorKind::UnexpectedFunctionName => match &self.token.value {
                None => "Unexpected function name.",
                Some(value) => &format!("Unexpected function name '{}'.", value),
            },
            SyntaxErrorKind::UnexpectedComma => "Unexpected comma.",
            SyntaxErrorKind::UnexpectedDot => "Unexpected dot.",
            SyntaxErrorKind::UnexpectedParenthesis => "Unexpected parenthesis.",
            SyntaxErrorKind::UnexpectedBrackets => "Unexpected brackets.",
            SyntaxErrorKind::UnexpectedNewLine => "Unexpected newline.",
            SyntaxErrorKind::UnmatchedParenthesis => "Unmatched parenthesis.",
            SyntaxErrorKind::UnmatchedBrackets => "Unmatched brackets.",
            SyntaxErrorKind::UnknownToken => "Unknown token.",
            SyntaxErrorKind::UnexpectedEndOfExpression => "Unexpected end of expression.",
        };

        write!(f, "{}", text)
    }
}

impl SyntaxError {
    pub fn display(&self, column_length: usize) -> String {
        format!(
            "{:fill$} {}",
            self.to_string().bold().red(),
            self.token.display_position().bold(),
            fill = column_length,
        )
    }
}

#[derive(Debug, Default)]
pub struct Status {
    pub expect_operand: bool,
    pub expect_operator: bool,
    pub in_string: bool,
}

impl SyntaxAnalyzer {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,

            errors: Vec::new(),
            current_index: 0,
            parentheses_stack: VecDeque::new(),
            brackets_stack: VecDeque::new(),
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
                        self.current_index += 1;
                        continue;
                    }

                    // Hex Validating
                    if let Some(number) = &token.value
                        && number.eq("0")
                        && let Some(next) = self.peek_next()
                        && next.kind == TokenType::Identifier
                        && let Some(value) = &next.value
                        && value.to_ascii_lowercase().starts_with('x')
                        && value.len() > 1
                    {
                        let hex_part = &value[1..];
                        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
                            // Incorrect hex literal
                            self.errors.push(syntax_error!(IncorrectHexLiteral, next));
                        }

                        // Anyway, considering that hex identifier was read
                        self.current_index += 2;
                        self.status.expect_operand = false;
                        self.status.expect_operator = true;
                        continue;
                    }

                    // Binary Validating
                    if let Some(number) = &token.value
                        && number.eq("0")
                        && let Some(next) = self.peek_next()
                        && next.kind == TokenType::Identifier
                        && let Some(value) = &next.value
                        && value.to_ascii_lowercase().starts_with('b')
                        && value.len() > 1
                    {
                        let binary_part = &value[1..];
                        if !binary_part.chars().all(|c| c == '0' || c == '1') {
                            // Incorrect binary literal
                            self.errors
                                .push(syntax_error!(IncorrectBinaryLiteral, next));
                        }

                        // Anyway, considering that hex identifier was read
                        self.current_index += 2;
                        self.status.expect_operand = false;
                        self.status.expect_operator = true;
                        continue;
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
                                self.errors.push(syntax_error!(IncorrectFloat, next));
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

                    // Bad variable name?
                    if let Some(next) = self.peek_next()
                        && next.kind == TokenType::Identifier
                    {
                        // If next token is identifier, then it's bad variable name
                        self.errors
                            .push(syntax_error!(IncorrectVariableName, token));
                        // Skipping bad variable name
                        self.current_index += 2;
                        self.status.expect_operand = false;
                        self.status.expect_operator = true;
                        continue;
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
                    let minus_with_identifier = if token.kind == TokenType::Minus
                        && let Some(next) = self.peek_next()
                        && [
                            TokenType::Identifier,
                            TokenType::Number,
                            TokenType::LeftParenthesis,
                        ]
                        .contains(&next.kind)
                    {
                        true
                    } else {
                        false
                    };

                    if self.status.expect_operator || minus_with_identifier {
                        self.status.expect_operand = true;
                        self.status.expect_operator = false;
                    } else {
                        // Not supporting unary operations unless it's minus before identifier
                        self.errors.push(syntax_error!(UnexpectedOperator, token));
                        // Waiting for operand still
                    }
                    self.current_index += 1;
                    continue;
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

                TokenType::LeftBracket => {
                    // LeftBracket can be there if previous token is Identifier (array access)
                    // or that's array with more than one dimension (e.g. arr[2][3])
                    let allow = matches!(self.peek_previous(), Some(t) if matches!(t.kind, TokenType::Identifier))
                        || matches!(self.peek_previous(), Some(t) if matches!(t.kind, TokenType::RightBracket));
                    if !allow {
                        self.errors.push(syntax_error!(UnexpectedBrackets, token));
                        self.current_index += 1;
                        continue;
                    }

                    self.brackets_stack.push_back(token.clone());
                    self.status.expect_operand = true;
                    self.status.expect_operator = false;
                    self.current_index += 1;
                    continue;
                },

                TokenType::RightBracket => {
                    match self.brackets_stack.pop_back().is_some() {
                        true => {
                            // Correct
                            self.status.expect_operand = false;
                            self.status.expect_operator = true;
                        },
                        false => {
                            self.errors.push(syntax_error!(UnmatchedBrackets, token))
                        },
                    }

                    // Empty array access check
                    if let Some(previous) = self.peek_previous()
                        && matches!(previous.kind, TokenType::LeftBracket)
                    {
                        self.errors.push(syntax_error!(EmptyBrackets, token));
                    }

                    self.current_index += 1;
                    continue;
                },

                TokenType::LeftParenthesis => {
                    // LeftParenthesis can be there if we're waiting for operand (grouping)
                    // or previous token is Identifier (function call)
                    // Number - error (processing later)
                    // RightParenthesis - error (processing later)
                    let allow = self.status.expect_operand
                        || matches!(self.peek_previous(), Some(t) if matches!(t.kind, TokenType::Identifier))
                        || matches!(self.peek_previous(), Some(t) if matches!(t.kind, TokenType::RightParenthesis))
                        || matches!(self.peek_previous(), Some(t) if matches!(t.kind, TokenType::Number));
                    if !allow {
                        self.errors
                            .push(syntax_error!(UnexpectedParenthesis, token));
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

                    if let Some(previous) = self.peek_previous()
                        && matches!(previous.kind, TokenType::RightParenthesis)
                    {
                        // Needed operation. but anyway, pushing to the stack
                        self.errors
                            .push(syntax_error!(UnexpectedParenthesis, token));
                    }

                    self.parentheses_stack.push_back(token.clone());
                    self.status.expect_operand = true;
                    self.status.expect_operator = false;
                    self.current_index += 1;
                    continue;
                },

                TokenType::RightParenthesis => {
                    match self.parentheses_stack.pop_back().is_some() {
                        true => {
                            // Correct
                            self.status.expect_operand = false;
                            self.status.expect_operator = true;
                        },
                        false => {
                            self.errors.push(syntax_error!(UnmatchedParenthesis, token))
                        },
                    }

                    // Empty grouping/function arguments check
                    if let Some(previous) = self.peek_previous()
                        && matches!(previous.kind, TokenType::LeftParenthesis)
                    {
                        self.errors.push(syntax_error!(EmptyParentheses, token));
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
                    }

                    // Inside parentheses comma need to be after operand and before new operand
                    if self.status.expect_operand {
                        // Empty argument
                        self.errors.push(syntax_error!(UnexpectedComma, token));
                        self.current_index += 1;
                        continue;
                    }

                    // Argument is not present
                    if let Some(next) = self.peek_next()
                        && matches!(next.kind, TokenType::RightParenthesis)
                    {
                        // Empty argument
                        self.errors.push(syntax_error!(MissingArgument, token));
                        self.current_index += 1;
                        continue;
                    }

                    // Expecting new operand
                    self.status.expect_operand = true;
                    self.status.expect_operator = false;
                    self.current_index += 1;
                    continue;
                },

                TokenType::Unknown => {
                    // Unknown — always an error
                    self.errors.push(syntax_error!(UnknownToken, token));
                    self.current_index += 1;
                    continue;
                },
                TokenType::NewLine => {
                    // Unexpected newline is error, if we're not in string
                    if !self.status.in_string {
                        self.errors.push(syntax_error!(UnexpectedNewLine, token));
                    }
                    self.current_index += 1;
                    continue;
                },
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
            self.errors
                .push(syntax_error!(UnexpectedEndOfExpression, last));
        }

        self.errors
            .sort_by(|a, b| a.token.position.start.cmp(&b.token.position.start));

        self.errors
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.current_index + 1)
    }

    fn peek_next_by(&self, by: usize) -> Option<&Token> {
        self.tokens.get(self.current_index + by)
    }

    fn peek_previous(&self) -> Option<&Token> {
        self.tokens.get(self.current_index.checked_sub(1)?)
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

        let errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(UnexpectedOperator, TokenType::Plus, 4),
            test_error!(
                IncorrectVariableName,
                TokenType::Number,
                10,
                "2".to_string()
            ),
            test_error!(UnmatchedParenthesis, TokenType::LeftParenthesis, 17),
            test_error!(UnexpectedComma, TokenType::Comma, 24),
            test_error!(UnexpectedOperator, TokenType::Asterisk, 32),
            test_error!(UnexpectedDot, TokenType::Dot, 37),
            test_error!(UnexpectedOperand, TokenType::Number, 38, "2".to_string()),
            test_error!(MissingArgument, TokenType::Comma, 40),
            test_error!(
                UnexpectedFunctionName,
                TokenType::Number,
                44,
                "8".to_string()
            ),
            test_error!(UnexpectedOperator, TokenType::Minus, 46),
            test_error!(UnexpectedOperator, TokenType::Asterisk, 49),
            test_error!(UnexpectedEndOfExpression, TokenType::Asterisk, 49),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

    #[test]
    fn test_syntax_02() {
        let code = "*a + nb -";

        let errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(UnexpectedOperator, TokenType::Asterisk, 0),
            test_error!(UnexpectedEndOfExpression, TokenType::Minus, 8),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

    #[test]
    fn test_syntax_03() {
        let code = "a ++ nb /* k -+/ g";

        let errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(UnexpectedOperator, TokenType::Plus, 3),
            test_error!(UnexpectedOperator, TokenType::Asterisk, 9),
            test_error!(UnexpectedOperator, TokenType::Plus, 14),
            test_error!(UnexpectedOperator, TokenType::Slash, 15),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

    #[test]
    fn test_syntax_04() {
        let code = "a^b$c - d#h + q%t + !b&(z|t)";

        let errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(UnknownToken, TokenType::Unknown, 1, "^".to_string()),
            test_error!(UnexpectedOperand, TokenType::Identifier, 2, "b".to_string()),
            test_error!(UnknownToken, TokenType::Unknown, 3, "$".to_string()),
            test_error!(UnexpectedOperand, TokenType::Identifier, 4, "c".to_string()),
            test_error!(UnknownToken, TokenType::Unknown, 9, "#".to_string()),
            test_error!(
                UnexpectedOperand,
                TokenType::Identifier,
                10,
                "h".to_string()
            ),
            test_error!(UnexpectedOperator, TokenType::Ampersand, 22),
            test_error!(UnexpectedParenthesis, TokenType::LeftParenthesis, 23),
            test_error!(
                UnexpectedOperand,
                TokenType::Identifier,
                24,
                "z".to_string()
            ),
            test_error!(UnexpectedOperator, TokenType::Pipe, 25),
            test_error!(
                UnexpectedOperand,
                TokenType::Identifier,
                26,
                "t".to_string()
            ),
            test_error!(UnmatchedParenthesis, TokenType::RightParenthesis, 27),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

    #[test]
    fn test_syntax_05() {
        let code = "x + var1 + var_2 + _var_3 + var#4 + var!5
    + 6var_ + $7 + ?8";

        let errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(UnknownToken, TokenType::Unknown, 31, "#".to_string()),
            test_error!(UnexpectedOperand, TokenType::Number, 32, "4".to_string()),
            test_error!(UnexpectedOperator, TokenType::ExclamationMark, 39),
            test_error!(UnexpectedOperand, TokenType::Number, 40, "5".to_string()),
            test_error!(UnexpectedNewLine, TokenType::NewLine, 41),
            test_error!(
                IncorrectVariableName,
                TokenType::Number,
                48,
                "6".to_string()
            ),
            test_error!(UnknownToken, TokenType::Unknown, 56, "$".to_string()),
            test_error!(UnknownToken, TokenType::Unknown, 61, "?".to_string()),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

    #[test]
    fn test_syntax_06() {
        let code = "125 + 2nb - 0xAB * 0x0R + 0b010 * 0b20+ ABh * 0Rh + 010b*20b";

        let errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(IncorrectVariableName, TokenType::Number, 6, "2".to_string()),
            test_error!(
                IncorrectHexLiteral,
                TokenType::Identifier,
                20..23,
                "x0R".to_string()
            ),
            test_error!(
                IncorrectBinaryLiteral,
                TokenType::Identifier,
                35..38,
                "b20".to_string()
            ),
            test_error!(
                IncorrectVariableName,
                TokenType::Number,
                46,
                "0".to_string()
            ),
            test_error!(
                IncorrectVariableName,
                TokenType::Number,
                52..55,
                "010".to_string()
            ),
            test_error!(
                IncorrectVariableName,
                TokenType::Number,
                57..59,
                "20".to_string()
            ),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

    #[test]
    fn test_syntax_07() {
        let code = "0.71/0.72.3 + .3 + 127.0.0.1*8. + 6.07ab - 9f.89hgt";

        let errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(UnexpectedDot, TokenType::Dot, 9),
            test_error!(UnexpectedOperand, TokenType::Number, 10, "3".to_string()),
            test_error!(UnexpectedDot, TokenType::Dot, 14),
            test_error!(UnexpectedDot, TokenType::Dot, 24),
            test_error!(UnexpectedOperand, TokenType::Number, 25, "0".to_string()),
            test_error!(UnexpectedDot, TokenType::Dot, 26),
            test_error!(UnexpectedOperand, TokenType::Number, 27, "1".to_string()),
            test_error!(IncorrectFloat, TokenType::Dot, 30),
            test_error!(
                UnexpectedOperand,
                TokenType::Identifier,
                38..40,
                "ab".to_string()
            ),
            test_error!(
                IncorrectVariableName,
                TokenType::Number,
                43,
                "9".to_string()
            ),
            test_error!(UnexpectedDot, TokenType::Dot, 45),
            test_error!(
                UnexpectedOperand,
                TokenType::Number,
                46..48,
                "89".to_string()
            ),
            test_error!(
                UnexpectedOperand,
                TokenType::Identifier,
                48..51,
                "hgt".to_string()
            ),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

    #[test]
    fn test_syntax_08() {
        let code = ")a+b( -(g+h)(g-k))*()) + (-b(t-2*x*(5) + A[7][2-x]";

        let errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(UnmatchedParenthesis, TokenType::RightParenthesis, 0),
            test_error!(UnexpectedParenthesis, TokenType::LeftParenthesis, 12),
            test_error!(EmptyParentheses, TokenType::RightParenthesis, 20),
            test_error!(UnmatchedParenthesis, TokenType::RightParenthesis, 21),
            test_error!(UnmatchedParenthesis, TokenType::LeftParenthesis, 25),
            test_error!(UnmatchedParenthesis, TokenType::LeftParenthesis, 28),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

    #[test]
    fn test_syntax_09() {
        let code = "2(t) - f2(t) + g()/h(2, )*func(-t/q, f(4-t), - (x+2)*(y-2))";

        let errors_actual: Vec<SyntaxError> =
            SyntaxAnalyzer::new(tokenizer::tokenize(code)).analyze();
        let errors_expected: Vec<SyntaxError> = vec![
            test_error!(
                UnexpectedFunctionName,
                TokenType::Number,
                0,
                "2".to_string()
            ),
            test_error!(EmptyParentheses, TokenType::RightParenthesis, 17),
            test_error!(MissingArgument, TokenType::Comma, 22),
        ];
        assert_eq!(errors_actual, errors_expected);
    }

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
