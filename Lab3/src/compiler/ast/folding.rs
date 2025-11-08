use crate::compiler::ast::tree::{
    AbstractSyntaxTree, AstError, AstNode, BinaryOperationKind, UnaryOperationKind,
};
use colored::Colorize;

impl AbstractSyntaxTree {
    pub fn fold(self) -> Result<AbstractSyntaxTree, AstError> {
        let folded = Self::fold_recursive(self.peek)?;

        Ok(Self::from_node(folded))
    }

    fn fold_recursive(node: AstNode) -> Result<AstNode, AstError> {
        match &node {
            AstNode::Number(_) | AstNode::Identifier(_) | AstNode::StringLiteral(_) => {
                Ok(node)
            },
            AstNode::UnaryOperation {
                operation,
                expression,
            } => {
                let folded_child = Self::fold_recursive(*expression.clone())?;
                Ok(AstNode::UnaryOperation {
                    operation: operation.clone(),
                    expression: Box::new(folded_child),
                })
            },
            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } => {
                let folded_left = Self::fold_recursive(*left.clone())?;
                let folded_right = Self::fold_recursive(*right.clone())?;

                match operation {
                    BinaryOperationKind::Plus => {
                        if let AstNode::UnaryOperation {
                            operation,
                            expression,
                        } = &folded_right
                            && operation.eq(&UnaryOperationKind::Minus)
                        {
                            return Ok(AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Minus,
                                left: Box::new(folded_left),
                                right: expression.clone(),
                            });
                        }
                    },
                    BinaryOperationKind::Multiply => {
                        if let AstNode::BinaryOperation {
                            operation,
                            left,
                            right,
                        } = &folded_right
                            && operation.eq(&BinaryOperationKind::Divide)
                            && let AstNode::Number(number) = **left
                            && [1.0, -1.0].contains(&number)
                        {
                            return Ok(AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Divide,
                                left: Box::new(folded_left),
                                right: right.clone(),
                            });
                        }
                    },
                    _ => {},
                }

                Ok(AstNode::BinaryOperation {
                    operation: operation.clone(),
                    left: Box::new(folded_left),
                    right: Box::new(folded_right),
                })
            },
            AstNode::FunctionCall { name, arguments } => {
                let folded_arguments: Result<Vec<AstNode>, AstError> = arguments
                    .iter()
                    .map(|arg| Self::fold_recursive(arg.clone()))
                    .collect();

                Ok(AstNode::FunctionCall {
                    name: name.clone(),
                    arguments: folded_arguments?,
                })
            },
            AstNode::ArrayAccess {
                identifier,
                indices,
            } => {
                let folded_indices: Result<Vec<AstNode>, AstError> = indices
                    .iter()
                    .map(|index| Self::fold_recursive(index.clone()))
                    .collect();

                Ok(AstNode::ArrayAccess {
                    identifier: identifier.clone(),
                    indices: folded_indices?,
                })
            },
        }
    }
}

pub fn report_success(tree: &AbstractSyntaxTree) {
    log::warn!(
        "{} {}.",
        "Folding Abstract-Syntax Tree",
        "success".bold().green()
    );
    log::info!("{}", tree.pretty_print());
}

pub fn report_error(error: AstError) {
    log::error!("{} {}", "Folding AST error:".bold().red(), error);
}
