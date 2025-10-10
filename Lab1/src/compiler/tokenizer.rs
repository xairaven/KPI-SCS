use std::ops::Range;

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub position: Range<usize>,
}

#[derive(Debug)]
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

    EOF,

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
                token_type: TokenType::EOF,
                position: index..index + 1,
            },
            c if c.is_whitespace() => continue,
            c => Token {
                token_type: TokenType::Unknown(c.to_owned()),
                position: index..index + 1,
            },
        };

        tokens.push(token);
    }

    tokens
}
