use crate::compiler::ast::tree::{AbstractSyntaxTree, AstNode, BinaryOperationKind};
use colored::Colorize;
use std::collections::VecDeque;

impl AbstractSyntaxTree {
    pub fn balance(self) -> Result<Self, BalancedAstError> {
        let peek = Self::balance_tree(self.peek)?;

        Ok(Self::from_node(peek))
    }

    pub fn balance_tree(node: AstNode) -> Result<AstNode, BalancedAstError> {
        match node {
            // Base cases, already balanced.
            AstNode::Number(_) | AstNode::Identifier(_) | AstNode::StringLiteral(_) => {
                Ok(node)
            },

            // Recursive cases for other node types.
            AstNode::UnaryOperation {
                operation,
                expression,
            } => Ok(AstNode::UnaryOperation {
                operation,
                expression: Box::new(Self::balance_tree(*expression)?),
            }),

            AstNode::FunctionCall { name, arguments } => {
                let mut balanced_arguments: Vec<AstNode> = vec![];
                for arg in arguments {
                    balanced_arguments.push(Self::balance_tree(arg)?);
                }

                Ok(AstNode::FunctionCall {
                    name,
                    arguments: balanced_arguments,
                })
            },

            AstNode::ArrayAccess {
                identifier,
                indices,
            } => {
                let mut balanced_indices: Vec<AstNode> = vec![];
                for index in indices {
                    balanced_indices.push(Self::balance_tree(index)?);
                }

                Ok(AstNode::ArrayAccess {
                    identifier,
                    indices: balanced_indices,
                })
            },

            // Main logic: Binary operations
            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } => {
                let balanced_left = Self::balance_tree(*left)?;
                let balanced_right = Self::balance_tree(*right)?;

                match operation {
                    BinaryOperationKind::Plus | BinaryOperationKind::Multiply => {
                        let mut operands = Vec::new();
                        Self::collect_operands(
                            AstNode::BinaryOperation {
                                operation: operation.clone(),
                                left: Box::new(balanced_left),
                                right: Box::new(balanced_right),
                            },
                            operation.clone(),
                            &mut operands,
                        );

                        Self::build_balanced_tree(operands, operation)
                    },

                    // Other operations (And, Or, etc.) are not associative
                    // in the arithmetic context. Just return them
                    // with already balanced children.
                    _ => Ok(AstNode::BinaryOperation {
                        operation,
                        left: Box::new(balanced_left),
                        right: Box::new(balanced_right),
                    }),
                }
            },
        }
    }

    /// Making flatten tree.
    /// Recursively "unfolds" a chain of associative operations
    /// into a flat list. For example, the tree `(a + (b + c)) + d`
    /// with `op_kind = Plus` will be "flattened" into the list `[a, b, c, d]`.
    fn collect_operands(
        node: AstNode, op_kind: BinaryOperationKind, operands: &mut Vec<AstNode>,
    ) {
        match node {
            // If operation node is the same as we are looking for...
            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } if operation == op_kind => {
                // ... we recursively collect operands from both sides.
                Self::collect_operands(*left, op_kind.clone(), operands);
                Self::collect_operands(*right, op_kind.clone(), operands);
            },
            // If operation node is different, or it's a leaf node...
            // or just operand (a, b, c...),
            // then it is a "leaf" for *this* chain.
            // We add it to the list.
            _ => {
                operands.push(node);
            },
        }
    }

    /// Building balanced tree
    /// Taking a flat list of operands and constructing
    /// a binary tree of minimal height using a queue-based algorithm.
    /// For example, `[a, b, c, d, e]` becomes `((a + b) + (c + d)) + e`
    /// (or a similar balanced structure).
    fn build_balanced_tree(
        operands: Vec<AstNode>, op_kind: BinaryOperationKind,
    ) -> Result<AstNode, BalancedAstError> {
        if operands.is_empty() {
            return Err(BalancedAstError::CannotBuildEmptyTree);
        }

        // Making a queue from the list of operands
        let mut queue: VecDeque<AstNode> = operands.into();

        // While more than one node remains in the queue...
        while queue.len() > 1 {
            let level_size = queue.len();

            // Process the current level of the tree:
            for _ in 0..(level_size / 2) {
                // Take two nodes from the front of the queue...
                let left = queue
                    .pop_front()
                    .ok_or(BalancedAstError::FailedPopFromQueue)?;
                let right = queue
                    .pop_front()
                    .ok_or(BalancedAstError::FailedPopFromQueue)?;

                // ...create a new binary operation node combining them...
                let new_node = AstNode::BinaryOperation {
                    operation: op_kind.clone(),
                    left: Box::new(left),
                    right: Box::new(right),
                };

                // (він буде операндом для наступного, вищого рівня)
                // .. and put the new node at the back of the queue
                // (it will be an operand for the next, higher level)
                queue.push_back(new_node);
            }

            // if level_size is odd...
            if !level_size.is_multiple_of(2) {
                // ...one node remains at the front of the queue.
                // We simply move it to the back,
                // so it can participate in the next iteration (next level).
                let odd_one_out = queue
                    .pop_front()
                    .ok_or(BalancedAstError::FailedPopFromQueue)?;
                queue.push_back(odd_one_out);
            }
        }

        // When only one node remains in the queue,
        // it is the root of the balanced tree.
        queue
            .pop_front()
            .ok_or(BalancedAstError::FailedPopFromQueue)
    }
}

#[derive(Debug, PartialEq)]
pub enum BalancedAstError {
    CannotBuildEmptyTree,
    FailedPopFromQueue,
}

impl std::fmt::Display for BalancedAstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::CannotBuildEmptyTree => {
                "Cannot build a balanced tree from zero operands"
            },
            Self::FailedPopFromQueue => {
                "Failed to pop node from the queue during tree construction"
            },
        };

        write!(f, "{}", text)
    }
}

pub fn report_success(tree: &AbstractSyntaxTree) {
    log::warn!(
        "{} {}.",
        "Balanced Abstract-Syntax Tree generation",
        "success".bold().green()
    );
    log::info!("{}", tree.pretty_print());
}

pub fn report_error(error: BalancedAstError) {
    log::error!("{} {}", "Balanced AST error:".bold().red(), error);
}
