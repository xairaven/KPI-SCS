use crate::compiler::lexer::Lexeme;
use colored::Colorize;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    Number(f64),
    Identifier(String),
    UnaryOperation {
        operation: UnaryOperationKind,
        expression: Box<AstNode>,
    },
    BinaryOperation {
        operation: BinaryOperationKind,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<AstNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperationKind {
    Minus,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperationKind {
    Plus,
    Minus,
    Multiply,
    Divide,
    Or,
    And,
}

pub struct AstParser<'a> {
    lexemes: Peekable<Iter<'a, Lexeme>>,
}

impl<'a> AstParser<'a> {
    pub fn new(lexemes: &'a [Lexeme]) -> Self {
        Self {
            lexemes: lexemes.iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<AstNode, AstError> {
        let node = self.parse_logical_or()?;

        if self.peek().is_some()
            && let Some(peek) = self.consume()
        {
            Err(AstError::NotExpectedLexeme(peek.clone()))
        } else {
            Ok(node)
        }
    }

    fn peek(&mut self) -> Option<&Lexeme> {
        self.lexemes.peek().copied()
    }

    fn consume(&mut self) -> Option<&'a Lexeme> {
        self.lexemes.next()
    }

    fn parse_logical_or(&mut self) -> Result<AstNode, AstError> {
        let mut left_node = self.parse_logical_and()?;

        while let Some(Lexeme::Or) = self.peek()
            && let Some(_) = self.consume()
        {
            let right_node = self.parse_logical_and()?;
            left_node = AstNode::BinaryOperation {
                operation: BinaryOperationKind::Or,
                left: Box::new(left_node),
                right: Box::new(right_node),
            };
        }
        Ok(left_node)
    }

    fn parse_logical_and(&mut self) -> Result<AstNode, AstError> {
        let mut left_node = self.parse_expression()?;

        while let Some(Lexeme::And) = self.peek()
            && let Some(_) = self.consume()
        {
            let right_node = self.parse_expression()?;
            left_node = AstNode::BinaryOperation {
                operation: BinaryOperationKind::And,
                left: Box::new(left_node),
                right: Box::new(right_node),
            };
        }
        Ok(left_node)
    }

    fn parse_expression(&mut self) -> Result<AstNode, AstError> {
        let mut left_node = self.parse_term()?;

        while let Some(Lexeme::Plus) | Some(Lexeme::Minus) = self.peek()
            && let Some(lexeme) = self.consume()
        {
            let operation = match lexeme {
                Lexeme::Plus => BinaryOperationKind::Plus,
                Lexeme::Minus => BinaryOperationKind::Minus,
                _ => return Err(AstError::UnreachableLexeme(lexeme.clone())),
            };

            let right_node = self.parse_term()?;

            left_node = AstNode::BinaryOperation {
                operation,
                left: Box::new(left_node),
                right: Box::new(right_node),
            };
        }

        Ok(left_node)
    }

    fn parse_term(&mut self) -> Result<AstNode, AstError> {
        let mut left_node = self.parse_unary()?;

        while let Some(Lexeme::Multiply) | Some(Lexeme::Divide) = self.peek()
            && let Some(lexeme) = self.consume()
        {
            let operation = match lexeme {
                Lexeme::Multiply => BinaryOperationKind::Multiply,
                Lexeme::Divide => BinaryOperationKind::Divide,
                _ => return Err(AstError::UnreachableLexeme(lexeme.clone())),
            };

            let right_node = self.parse_unary()?;

            left_node = AstNode::BinaryOperation {
                operation,
                left: Box::new(left_node),
                right: Box::new(right_node),
            };
        }

        Ok(left_node)
    }

    fn parse_unary(&mut self) -> Result<AstNode, AstError> {
        if let Some(Lexeme::Not) | Some(Lexeme::Minus) = self.peek()
            && let Some(lexeme) = self.consume()
        {
            let operation_kind = match lexeme {
                Lexeme::Not => UnaryOperationKind::Not,
                Lexeme::Minus => UnaryOperationKind::Minus,
                _ => return Err(AstError::UnreachableLexeme(lexeme.clone())),
            };

            let child_node = self.parse_unary()?;

            Ok(AstNode::UnaryOperation {
                operation: operation_kind,
                expression: Box::new(child_node),
            })
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<AstNode, AstError> {
        if let Some(lexeme) = self.consume() {
            match lexeme {
                Lexeme::Number(value) => Ok(AstNode::Number(*value)),

                Lexeme::LeftParenthesis => {
                    let inner_node = self.parse_logical_or()?;

                    if self.peek() == Some(&Lexeme::RightParenthesis) {
                        self.consume();
                        Ok(inner_node)
                    } else {
                        Err(AstError::ExpectedRightParenthesis)
                    }
                },

                Lexeme::Identifier(name) => {
                    if self.peek() == Some(&Lexeme::LeftParenthesis)
                        && let Some(_) = self.consume()
                    {
                        let function_name = name.clone();
                        let mut args = Vec::new();

                        if self.peek() != Some(&Lexeme::RightParenthesis) {
                            loop {
                                args.push(self.parse_logical_or()?);

                                let peek = self.peek();

                                if peek == Some(&Lexeme::Comma) {
                                    let _ = self.consume();
                                } else if peek == Some(&Lexeme::RightParenthesis) {
                                    break;
                                } else {
                                    return Err(match peek {
                                        None => AstError::NotExpectedEndOfExpression,
                                        Some(lexeme) => {
                                            AstError::ExpectedCommaOrRightParenthesis(
                                                lexeme.clone(),
                                            )
                                        },
                                    });
                                }
                            }
                        }

                        let _ = self.consume();

                        Ok(AstNode::FunctionCall {
                            name: function_name,
                            arguments: args,
                        })
                    } else {
                        Ok(AstNode::Identifier(name.clone()))
                    }
                },

                _ => Err(AstError::NotExpectedLexeme(lexeme.clone())),
            }
        } else {
            Err(AstError::NotExpectedEndOfExpression)
        }
    }
}

pub fn report(result: Result<AstNode, AstError>) -> Result<(AstNode, String), String> {
    match result {
        Ok(ast) => {
            let report = format!(
                "\n{}\n{}",
                "Abstract-Syntax Tree generation success.".bold().green(),
                ast.pretty_print()
            );

            Ok((ast, report))
        },
        Err(error) => Err(format!("\n{} {}", "AST error:".bold().red(), error)),
    }
}

#[derive(Debug)]
pub enum AstError {
    ExpectedRightParenthesis,
    ExpectedCommaOrRightParenthesis(Lexeme),
    NotExpectedEndOfExpression,
    NotExpectedLexeme(Lexeme),
    UnreachableLexeme(Lexeme),
}

impl std::fmt::Display for AstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::ExpectedCommaOrRightParenthesis(lexeme) => &format!(
                "Expected ',' or ')', but found \"{}\".",
                lexeme.display_type()
            ),
            Self::ExpectedRightParenthesis => "Expected right parenthesis.",
            Self::NotExpectedEndOfExpression => "Not expected end of expression.",
            Self::NotExpectedLexeme(lexeme) => {
                &format!("Not expected lexeme \"{}\".", lexeme.display_type())
            },
            Self::UnreachableLexeme(lexeme) => {
                &format!("Unreachable lexeme \"{}\".", lexeme.display_type())
            },
        };

        write!(f, "{}", text)
    }
}

impl std::fmt::Display for UnaryOperationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minus => write!(f, "-"),
            Self::Not => write!(f, "!"),
        }
    }
}

impl std::fmt::Display for BinaryOperationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
            Self::Or => write!(f, "|"),
            Self::And => write!(f, "&"),
        }
    }
}

impl AstNode {
    pub fn pretty_print(&self) -> String {
        let mut tree = String::new();
        self.print_recursive(&mut tree, "".to_string(), true);
        tree
    }

    fn print_recursive(&self, tree: &mut String, prefix: String, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };

        tree.push_str(&format!("{}{}", prefix.dimmed(), connector.dimmed()));

        let node_text = match self {
            AstNode::Number(n) => n.to_string().bright_blue(),
            AstNode::Identifier(s) => s.to_string().green(),
            AstNode::UnaryOperation { operation, .. } => {
                operation.to_string().yellow().bold()
            },
            AstNode::BinaryOperation { operation, .. } => {
                operation.to_string().yellow().bold()
            },
            AstNode::FunctionCall { name, .. } => format!("{}(...)", name).cyan().bold(),
        };
        tree.push_str(&format!("{}\n", node_text));

        let new_prefix = prefix + if is_last { "    " } else { "│   " };

        match self {
            AstNode::Number(_) | AstNode::Identifier(_) => {},

            AstNode::UnaryOperation { expression, .. } => {
                expression.print_recursive(tree, new_prefix, true);
            },

            AstNode::BinaryOperation { left, right, .. } => {
                left.print_recursive(tree, new_prefix.clone(), false);
                right.print_recursive(tree, new_prefix, true);
            },

            AstNode::FunctionCall { arguments, .. } => {
                let arg_count = arguments.len();
                for (i, arg) in arguments.iter().enumerate() {
                    let is_last_arg = i == arg_count - 1;
                    arg.print_recursive(tree, new_prefix.clone(), is_last_arg);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{ast, lexer, tokenizer};

    fn process(code: &str) -> AstNode {
        let tokens = tokenizer::tokenize(code);
        let lexemes = lexer::Lexer::new(tokens).run();
        assert!(lexemes.is_ok());
        let lexemes = lexemes.unwrap();
        let result = AstParser::new(&lexemes).parse();
        let report = report(result);
        assert!(report.is_ok());
        match report {
            Ok((ast, report)) => {
                println!("{}", report);
                ast
            },
            Err(report) => {
                println!("{}", report);
                panic!();
            },
        }
    }

    #[test]
    fn test_1() {
        let code = "a + b * c";
        let actual_ast = process(code);
        let expected_ast = AstNode::BinaryOperation {
            operation: BinaryOperationKind::Plus,
            left: Box::new(AstNode::Identifier("a".to_string())),
            right: Box::new(AstNode::BinaryOperation {
                operation: BinaryOperationKind::Multiply,
                left: Box::new(AstNode::Identifier("b".to_string())),
                right: Box::new(AstNode::Identifier("c".to_string())),
            }),
        };
        assert_eq!(expected_ast, actual_ast);
    }

    #[test]
    fn test_2() {
        let code = "a + b * func(a, (b - c) * !d)";
        let actual_ast = process(code);
        let expected_ast = AstNode::BinaryOperation {
            operation: BinaryOperationKind::Plus,
            left: Box::new(AstNode::Identifier("a".to_string())),
            right: Box::new(AstNode::BinaryOperation {
                operation: BinaryOperationKind::Multiply,
                left: Box::new(AstNode::Identifier("b".to_string())),
                right: Box::new(AstNode::FunctionCall {
                    name: "func".to_string(),
                    arguments: vec![
                        AstNode::Identifier("a".to_string()),
                        AstNode::BinaryOperation {
                            operation: BinaryOperationKind::Multiply,
                            left: Box::new(AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Minus,
                                left: Box::new(AstNode::Identifier("b".to_string())),
                                right: Box::new(AstNode::Identifier("c".to_string())),
                            }),
                            right: Box::new(AstNode::UnaryOperation {
                                operation: UnaryOperationKind::Not,
                                expression: Box::new(AstNode::Identifier(
                                    "d".to_string(),
                                )),
                            }),
                        },
                    ],
                }),
            }),
        };
        assert_eq!(expected_ast, actual_ast);
    }
}
