use crate::compiler::ast::tree::{
    AbstractSyntaxTree, AstError, AstNode, BinaryOperationKind, UnaryOperationKind,
};
use colored::Colorize;

impl AbstractSyntaxTree {
    pub fn transform(self) -> Result<AbstractSyntaxTree, AstError> {
        let peek = Self::transform_recursive(self.peek)?;

        Ok(Self::from_node(peek))
    }

    pub fn transform_recursive(node: AstNode) -> Result<AstNode, AstError> {
        match node {
            AstNode::Number(_) | AstNode::Identifier(_) | AstNode::StringLiteral(_) => {
                Ok(node)
            },

            AstNode::UnaryOperation {
                operation,
                expression,
            } => {
                let transformed_expression = Self::transform_recursive(*expression)?;
                Ok(AstNode::UnaryOperation {
                    operation,
                    expression: Box::new(transformed_expression),
                })
            },

            AstNode::FunctionCall { name, arguments } => {
                let mut transformed_arguments = vec![];
                for argument in arguments {
                    transformed_arguments.push(Self::transform_recursive(argument)?);
                }

                Ok(AstNode::FunctionCall {
                    name,
                    arguments: transformed_arguments,
                })
            },

            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } => {
                let transformed_left = Self::transform_recursive(*left)?;
                let transformed_right = Self::transform_recursive(*right)?;

                match operation {
                    // Rule 1: A - B  =>  A + (-B)
                    BinaryOperationKind::Minus => {
                        if let AstNode::Number(number) = transformed_right
                            && number.is_sign_negative()
                        {
                            // Rule 1: A - (-B)  =>  A + B
                            Ok(AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Plus,
                                left: Box::new(transformed_left),
                                right: Box::new(AstNode::Number(f64::abs(number))),
                            })
                        } else if let AstNode::Number(number) = transformed_right {
                            // Rule 1: A - B  =>  A + (-B)
                            Ok(AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Plus,
                                left: Box::new(transformed_left),
                                right: Box::new(AstNode::Number(-number)),
                            })
                        } else {
                            Ok(AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Plus,
                                left: Box::new(transformed_left),
                                right: Box::new(AstNode::UnaryOperation {
                                    operation: UnaryOperationKind::Minus,
                                    expression: Box::new(transformed_right),
                                }),
                            })
                        }
                    },

                    // Rule 2: A / B  =>  A * (1 / B)
                    BinaryOperationKind::Divide => {
                        if let AstNode::Number(number) = transformed_right {
                            return if number == 0.0 {
                                Err(AstError::DivisionByZero)
                            } else {
                                Ok(AstNode::BinaryOperation {
                                    operation: BinaryOperationKind::Multiply,
                                    left: Box::new(transformed_left),
                                    right: Box::new(AstNode::Number(1.0 / number)),
                                })
                            };
                        }
                        Ok(AstNode::BinaryOperation {
                            operation: BinaryOperationKind::Multiply,
                            left: Box::new(transformed_left),
                            right: Box::new(AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Divide,
                                left: Box::new(AstNode::Number(1.0)), // "1"
                                right: Box::new(transformed_right),   // "B"
                            }),
                        })
                    },

                    // Other operations (Plus, Multiply, And, Or)
                    // left without editing, but with transformed kids.
                    _ => Ok(AstNode::BinaryOperation {
                        operation,
                        left: Box::new(transformed_left),
                        right: Box::new(transformed_right),
                    }),
                }
            },

            AstNode::ArrayAccess {
                identifier,
                indices,
            } => {
                let mut transformed_indices = vec![];
                for index in indices {
                    transformed_indices.push(Self::transform_recursive(index)?);
                }
                Ok(AstNode::ArrayAccess {
                    identifier,
                    indices: transformed_indices,
                })
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

pub fn report_error(error: AstError) {
    log::error!(
        "{} {}",
        "Transformed Abstract-Syntax Tree generation:".bold().red(),
        error
    );
}
