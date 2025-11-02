use crate::compiler::ast::tree::{
    AbstractSyntaxTree, AstNode, BinaryOperationKind, UnaryOperationKind,
};
use colored::Colorize;

impl AbstractSyntaxTree {
    pub fn compute(self) -> AbstractSyntaxTree {
        let computed = Self::compute_recursive(self.peek);

        Self::from_node(computed)
    }

    fn compute_recursive(node: AstNode) -> AstNode {
        match &node {
            AstNode::Number(_) | AstNode::Identifier(_) | AstNode::StringLiteral(_) => {
                node
            },
            AstNode::UnaryOperation {
                operation: op,
                expression,
            } => match &op {
                UnaryOperationKind::Minus => {
                    let child = Self::compute_recursive(*expression.clone());
                    if let AstNode::Number(number) = child {
                        AstNode::Number(-number)
                    } else {
                        node
                    }
                },
                UnaryOperationKind::Not => node,
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
                    let computed_left = Self::compute_recursive(*left.clone());
                    let computed_right = Self::compute_recursive(*right.clone());

                    if let (AstNode::Number(left_number), AstNode::Number(right_number)) =
                        (&computed_left, &computed_right)
                    {
                        let result = match operation {
                            BinaryOperationKind::Plus => left_number + right_number,
                            BinaryOperationKind::Minus => left_number - right_number,
                            BinaryOperationKind::Multiply => left_number * right_number,
                            BinaryOperationKind::Divide => left_number / right_number,
                            _ => unreachable!(),
                        };
                        AstNode::Number(result)
                    } else {
                        AstNode::BinaryOperation {
                            operation: operation.clone(),
                            left: Box::new(computed_left),
                            right: Box::new(computed_right),
                        }
                    }
                },
                _ => node,
            },
            AstNode::FunctionCall { name, arguments } => {
                let computed_arguments = arguments
                    .iter()
                    .map(|arg| Self::compute_recursive(arg.clone()))
                    .collect();
                AstNode::FunctionCall {
                    name: name.clone(),
                    arguments: computed_arguments,
                }
            },
            AstNode::ArrayAccess {
                identifier,
                indices,
            } => {
                let computed_indices = indices
                    .iter()
                    .map(|index| Self::compute_recursive(index.clone()))
                    .collect();
                AstNode::ArrayAccess {
                    identifier: identifier.clone(),
                    indices: computed_indices,
                }
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
