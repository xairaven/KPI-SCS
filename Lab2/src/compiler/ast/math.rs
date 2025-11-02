use crate::compiler::ast::tree::{
    AbstractSyntaxTree, AstError, AstNode, BinaryOperationKind, UnaryOperationKind,
};
use colored::Colorize;

impl AbstractSyntaxTree {
    pub fn compute(self) -> Result<AbstractSyntaxTree, AstError> {
        let computed = Self::compute_recursive(self.peek)?;

        Ok(Self::from_node(computed))
    }

    fn compute_recursive(node: AstNode) -> Result<AstNode, AstError> {
        match &node {
            AstNode::Number(_) | AstNode::Identifier(_) | AstNode::StringLiteral(_) => {
                Ok(node)
            },
            AstNode::UnaryOperation {
                operation: op,
                expression,
            } => match &op {
                UnaryOperationKind::Minus => {
                    let child = Self::compute_recursive(*expression.clone())?;
                    if let AstNode::Number(number) = child {
                        Ok(AstNode::Number(-number))
                    } else {
                        Ok(node)
                    }
                },
                UnaryOperationKind::Not => Ok(node),
            },
            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } => match operation {
                BinaryOperationKind::Plus
                | BinaryOperationKind::Minus
                | BinaryOperationKind::Multiply
                | BinaryOperationKind::Divide => {
                    let computed_left = Self::compute_recursive(*left.clone())?;
                    let computed_right = Self::compute_recursive(*right.clone())?;

                    if let (AstNode::Number(left_number), AstNode::Number(right_number)) =
                        (&computed_left, &computed_right)
                    {
                        let result = match operation {
                            BinaryOperationKind::Plus => left_number + right_number,
                            BinaryOperationKind::Minus => left_number - right_number,
                            BinaryOperationKind::Multiply => left_number * right_number,
                            BinaryOperationKind::Divide => {
                                if *right_number == 0.0 {
                                    return Err(AstError::DivisionByZero);
                                } else {
                                    left_number / right_number
                                }
                            },
                            _ => unreachable!(),
                        };
                        Ok(AstNode::Number(result))
                    } else {
                        Ok(AstNode::BinaryOperation {
                            operation: operation.clone(),
                            left: Box::new(computed_left),
                            right: Box::new(computed_right),
                        })
                    }
                },
                _ => Ok(node),
            },
            AstNode::FunctionCall { name, arguments } => {
                let mut computed_arguments = Vec::new();
                for arg in arguments {
                    let arg = Self::compute_recursive(arg.clone())?;
                    computed_arguments.push(arg);
                }

                Ok(AstNode::FunctionCall {
                    name: name.clone(),
                    arguments: computed_arguments,
                })
            },
            AstNode::ArrayAccess {
                identifier,
                indices,
            } => {
                let mut computed_indices = Vec::new();
                for index in indices {
                    let index = Self::compute_recursive(index.clone())?;
                    computed_indices.push(index);
                }
                Ok(AstNode::ArrayAccess {
                    identifier: identifier.clone(),
                    indices: computed_indices,
                })
            },
        }
    }
}

pub fn report_success(tree: &AbstractSyntaxTree) {
    log::warn!(
        "{} {}.",
        "Computing constants of Abstract-Syntax Tree",
        "success".bold().green()
    );
    log::info!("{}", tree.pretty_print());
}

pub fn report_error(error: AstError) {
    log::error!(
        "{} {}",
        "Computing constants of Abstract-Syntax Tree:".bold().red(),
        error
    );
}

pub fn check_finalization(tree: &AbstractSyntaxTree) -> bool {
    if let AstNode::Number(number) = &tree.peek {
        log::warn!(
            "{} = {}. {}.",
            "Computing solved code, result".bold().green(),
            number,
            "Further optimization is not needed".bold().red(),
        );
        return true;
    }
    false
}
