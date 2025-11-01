use crate::compiler::ast::tree::{
    AbstractSyntaxTree, AstNode, BinaryOperationKind, UnaryOperationKind,
};
use colored::Colorize;

impl AbstractSyntaxTree {
    pub fn transform(self) -> AbstractSyntaxTree {
        let peek = Self::transform_recursive(self.peek);

        Self::from_node(peek)
    }

    pub fn transform_recursive(node: AstNode) -> AstNode {
        match node {
            AstNode::Number(_) | AstNode::Identifier(_) | AstNode::StringLiteral(_) => {
                node
            },

            AstNode::UnaryOperation {
                operation,
                expression,
            } => {
                let transformed_expression = Self::transform_recursive(*expression);
                AstNode::UnaryOperation {
                    operation,
                    expression: Box::new(transformed_expression),
                }
            },

            AstNode::FunctionCall { name, arguments } => {
                let transformed_arguments = arguments
                    .into_iter()
                    .map(Self::transform_recursive) // Рекурсія для кожного аргумента
                    .collect();
                AstNode::FunctionCall {
                    name,
                    arguments: transformed_arguments,
                }
            },

            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } => {
                let transformed_left = Self::transform_recursive(*left);
                let transformed_right = Self::transform_recursive(*right);

                match operation {
                    // Rule 1: A - B  =>  A + (-B)
                    BinaryOperationKind::Minus => {
                        if let AstNode::Number(number) = transformed_right
                            && number.is_sign_negative()
                        {
                            // Rule 1: A - (-B)  =>  A + B
                            AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Plus,
                                left: Box::new(transformed_left),
                                right: Box::new(AstNode::Number(f64::abs(number))),
                            }
                        } else if let AstNode::Number(number) = transformed_right {
                            // Rule 1: A - B  =>  A + (-B)
                            AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Plus,
                                left: Box::new(transformed_left),
                                right: Box::new(AstNode::Number(-number)),
                            }
                        } else {
                            AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Plus,
                                left: Box::new(transformed_left),
                                right: Box::new(AstNode::UnaryOperation {
                                    operation: UnaryOperationKind::Minus,
                                    expression: Box::new(transformed_right),
                                }),
                            }
                        }
                    },

                    // Rule 2: A / B  =>  A * (1 / B)
                    BinaryOperationKind::Divide => AstNode::BinaryOperation {
                        operation: BinaryOperationKind::Multiply,
                        left: Box::new(transformed_left),
                        right: Box::new(AstNode::BinaryOperation {
                            operation: BinaryOperationKind::Divide,
                            left: Box::new(AstNode::Number(1.0)), // "1"
                            right: Box::new(transformed_right),   // "B"
                        }),
                    },

                    // Other operations (Plus, Multiply, And, Or)
                    // left without editing, but with transformed kids.
                    _ => AstNode::BinaryOperation {
                        operation,
                        left: Box::new(transformed_left),
                        right: Box::new(transformed_right),
                    },
                }
            },

            AstNode::ArrayAccess {
                identifier,
                indices,
            } => {
                let transformed_indices = indices
                    .into_iter()
                    .map(Self::transform_recursive) // Recursion for every argument
                    .collect();
                AstNode::ArrayAccess {
                    identifier,
                    indices: transformed_indices,
                }
            },
        }
    }
}

pub fn report_success(tree: &AbstractSyntaxTree) {
    log::warn!(
        "{} {}.",
        "Transformed Abstract-Syntax Tree generation",
        "success".bold().green()
    );
    log::info!("{}", tree.pretty_print());
}
