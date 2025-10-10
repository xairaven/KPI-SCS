use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub position: Range<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Identifier(String),
    Number(String),

    Plus,
    Minus,
    Asterisk,
    Slash,

    LeftParenthesis,
    RightParenthesis,

    Dot,
    Comma,
    Semicolon,
    QuotationMark,

    Space,
    Tab,
    NewLine,

    Unknown(char),
}

macro_rules! token {
    ($token_type:expr, $position:literal) => {
        Token {
            token_type: $token_type,
            position: $position..$position + 1,
        }
    };
    ($token_type:expr, $position:expr) => {
        Token {
            token_type: $token_type,
            position: $position,
        }
    };
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = input.chars().collect();

    for (index, symbol) in chars.iter().enumerate() {
        if let Some(last_token) = tokens.last()
            && last_token.position.end > index
        {
            continue;
        }

        let token = match symbol {
            symbol if symbol.is_alphabetic() || symbol.eq(&'_') => {
                let start = index;
                let mut end = index + 1;

                while end < chars.len()
                    && (chars[end].is_alphanumeric() || chars[end] == '_')
                {
                    end += 1;
                }

                let value: String = chars[start..end].iter().collect();
                token!(TokenType::Identifier(value), start..end)
            },
            '0'..='9' => {
                let start = index;
                let mut end = index + 1;

                while end < chars.len() && chars[end].is_numeric() {
                    end += 1;
                }

                let value: String = chars[start..end].iter().collect();
                token!(TokenType::Number(value), start..end)
            },
            '+' => token!(TokenType::Plus, index..index + 1),
            '-' => token!(TokenType::Minus, index..index + 1),
            '*' => token!(TokenType::Asterisk, index..index + 1),
            '/' => token!(TokenType::Slash, index..index + 1),
            '(' => token!(TokenType::LeftParenthesis, index..index + 1),
            ')' => token!(TokenType::RightParenthesis, index..index + 1),
            '.' => token!(TokenType::Dot, index..index + 1),
            ',' => token!(TokenType::Comma, index..index + 1),
            ';' => token!(TokenType::Semicolon, index..index + 1),
            '"' => token!(TokenType::QuotationMark, index..index + 1),
            '\n' => token!(TokenType::NewLine, index..index + 1),
            c if c.eq(&'\t') => token!(TokenType::Tab, index..index + 1),
            c if c.is_whitespace() => {
                let start = index;
                let mut end = index + 1;

                while end < chars.len() && chars[end].is_whitespace() {
                    end += 1;
                }

                token!(TokenType::Space, start..end)
            },
            c => token!(TokenType::Unknown(c.to_owned()), index..index + 1),
        };

        tokens.push(token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_1() {
        let code = "-a ++ b - 2v*func((t+2 -, sin(x/*2.01.2), )/8(-)**";

        let tokens_actual = tokenize(code);
        let tokens_expected = vec![
            token!(TokenType::Minus, 0),
            token!(TokenType::Identifier("a".to_string()), 1),
            token!(TokenType::Space, 2),
            token!(TokenType::Plus, 3),
            token!(TokenType::Plus, 4),
            token!(TokenType::Space, 5),
            token!(TokenType::Identifier("b".to_string()), 6),
            token!(TokenType::Space, 7),
            token!(TokenType::Minus, 8),
            token!(TokenType::Space, 9),
            token!(TokenType::Number("2".to_string()), 10),
            token!(TokenType::Identifier("v".to_string()), 11),
            token!(TokenType::Asterisk, 12),
            token!(TokenType::Identifier("func".to_string()), 13..17),
            token!(TokenType::LeftParenthesis, 17),
            token!(TokenType::LeftParenthesis, 18),
            token!(TokenType::Identifier("t".to_string()), 19),
            token!(TokenType::Plus, 20),
            token!(TokenType::Number("2".to_string()), 21),
            token!(TokenType::Space, 22),
            token!(TokenType::Minus, 23),
            token!(TokenType::Comma, 24),
            token!(TokenType::Space, 25),
            token!(TokenType::Identifier("sin".to_string()), 26..29),
            token!(TokenType::LeftParenthesis, 29),
            token!(TokenType::Identifier("x".to_string()), 30),
            token!(TokenType::Slash, 31),
            token!(TokenType::Asterisk, 32),
            token!(TokenType::Number("2".to_string()), 33),
            token!(TokenType::Dot, 34),
            token!(TokenType::Number("01".to_string()), 35..37),
            token!(TokenType::Dot, 37),
            token!(TokenType::Number("2".to_string()), 38),
            token!(TokenType::RightParenthesis, 39),
            token!(TokenType::Comma, 40),
            token!(TokenType::Space, 41),
            token!(TokenType::RightParenthesis, 42),
            token!(TokenType::Slash, 43),
            token!(TokenType::Number("8".to_string()), 44),
            token!(TokenType::LeftParenthesis, 45),
            token!(TokenType::Minus, 46),
            token!(TokenType::RightParenthesis, 47),
            token!(TokenType::Asterisk, 48),
            token!(TokenType::Asterisk, 49),
        ];

        assert_eq!(tokens_actual, tokens_expected);
    }
}
