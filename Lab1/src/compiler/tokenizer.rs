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
                Token {
                    token_type: TokenType::Identifier(value),
                    position: start..end,
                }
            },
            '0'..='9' => {
                let start = index;
                let mut end = index + 1;

                while end < chars.len() && chars[end].is_numeric() {
                    end += 1;
                }

                let number: String = chars[start..end].iter().collect();
                Token {
                    token_type: TokenType::Number(number),
                    position: start..end,
                }
            },
            '+' => Token {
                token_type: TokenType::Plus,
                position: index..index + 1,
            },
            '-' => Token {
                token_type: TokenType::Minus,
                position: index..index + 1,
            },
            '*' => Token {
                token_type: TokenType::Asterisk,
                position: index..index + 1,
            },
            '/' => Token {
                token_type: TokenType::Slash,
                position: index..index + 1,
            },
            '(' => Token {
                token_type: TokenType::LeftParenthesis,
                position: index..index + 1,
            },
            ')' => Token {
                token_type: TokenType::RightParenthesis,
                position: index..index + 1,
            },
            '.' => Token {
                token_type: TokenType::Dot,
                position: index..index + 1,
            },
            ',' => Token {
                token_type: TokenType::Comma,
                position: index..index + 1,
            },
            ';' => Token {
                token_type: TokenType::Semicolon,
                position: index..index + 1,
            },
            '"' => Token {
                token_type: TokenType::QuotationMark,
                position: index..index + 1,
            },
            '\n' => Token {
                token_type: TokenType::NewLine,
                position: index..index + 1,
            },
            c if c.eq(&'\t') => Token {
                token_type: TokenType::Tab,
                position: index..index + 1,
            },
            c if c.is_whitespace() => {
                let start = index;
                let mut end = index + 1;

                while end < chars.len() && chars[end].is_whitespace() {
                    end += 1;
                }

                Token {
                    token_type: TokenType::Space,
                    position: start..end,
                }
            },
            c => Token {
                token_type: TokenType::Unknown(c.to_owned()),
                position: index..index + 1,
            },
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
            Token {
                token_type: TokenType::Minus,
                position: 0..1,
            },
            Token {
                token_type: TokenType::Identifier("a".to_string()),
                position: 1..2,
            },
            Token {
                token_type: TokenType::Space,
                position: 2..3,
            },
            Token {
                token_type: TokenType::Plus,
                position: 3..4,
            },
            Token {
                token_type: TokenType::Plus,
                position: 4..5,
            },
            Token {
                token_type: TokenType::Space,
                position: 5..6,
            },
            Token {
                token_type: TokenType::Identifier("b".to_string()),
                position: 6..7,
            },
            Token {
                token_type: TokenType::Space,
                position: 7..8,
            },
            Token {
                token_type: TokenType::Minus,
                position: 8..9,
            },
            Token {
                token_type: TokenType::Space,
                position: 9..10,
            },
            Token {
                token_type: TokenType::Number("2".to_string()),
                position: 10..11,
            },
            Token {
                token_type: TokenType::Identifier("v".to_string()),
                position: 11..12,
            },
            Token {
                token_type: TokenType::Asterisk,
                position: 12..13,
            },
            Token {
                token_type: TokenType::Identifier("func".to_string()),
                position: 13..17,
            },
            Token {
                token_type: TokenType::LeftParenthesis,
                position: 17..18,
            },
            Token {
                token_type: TokenType::LeftParenthesis,
                position: 18..19,
            },
            Token {
                token_type: TokenType::Identifier("t".to_string()),
                position: 19..20,
            },
            Token {
                token_type: TokenType::Plus,
                position: 20..21,
            },
            Token {
                token_type: TokenType::Number("2".to_string()),
                position: 21..22,
            },
            Token {
                token_type: TokenType::Space,
                position: 22..23,
            },
            Token {
                token_type: TokenType::Minus,
                position: 23..24,
            },
            Token {
                token_type: TokenType::Comma,
                position: 24..25,
            },
            Token {
                token_type: TokenType::Space,
                position: 25..26,
            },
            Token {
                token_type: TokenType::Identifier("sin".to_string()),
                position: 26..29,
            },
            Token {
                token_type: TokenType::LeftParenthesis,
                position: 29..30,
            },
            Token {
                token_type: TokenType::Identifier("x".to_string()),
                position: 30..31,
            },
            Token {
                token_type: TokenType::Slash,
                position: 31..32,
            },
            Token {
                token_type: TokenType::Asterisk,
                position: 32..33,
            },
            Token {
                token_type: TokenType::Number("2".to_string()),
                position: 33..34,
            },
            Token {
                token_type: TokenType::Dot,
                position: 34..35,
            },
            Token {
                token_type: TokenType::Number("01".to_string()),
                position: 35..37,
            },
            Token {
                token_type: TokenType::Dot,
                position: 37..38,
            },
            Token {
                token_type: TokenType::Number("2".to_string()),
                position: 38..39,
            },
            Token {
                token_type: TokenType::RightParenthesis,
                position: 39..40,
            },
            Token {
                token_type: TokenType::Comma,
                position: 40..41,
            },
            Token {
                token_type: TokenType::Space,
                position: 41..42,
            },
            Token {
                token_type: TokenType::RightParenthesis,
                position: 42..43,
            },
            Token {
                token_type: TokenType::Slash,
                position: 43..44,
            },
            Token {
                token_type: TokenType::Number("8".to_string()),
                position: 44..45,
            },
            Token {
                token_type: TokenType::LeftParenthesis,
                position: 45..46,
            },
            Token {
                token_type: TokenType::Minus,
                position: 46..47,
            },
            Token {
                token_type: TokenType::RightParenthesis,
                position: 47..48,
            },
            Token {
                token_type: TokenType::Asterisk,
                position: 48..49,
            },
            Token {
                token_type: TokenType::Asterisk,
                position: 49..50,
            },
        ];

        assert_eq!(tokens_actual, tokens_expected);
    }
}
