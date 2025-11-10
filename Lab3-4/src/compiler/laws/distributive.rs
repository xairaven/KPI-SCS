// File: Lab3-4/src/compiler/ast/distributive.rs

use crate::compiler::ast::tree::{
    AbstractSyntaxTree, AstError, AstNode, BinaryOperationKind,
};

impl AbstractSyntaxTree {
    /// Applies the distributive law (expanding brackets).
    /// e.g., a*(b+c) -> a*b + a*c
    pub fn expand(self) -> Result<AbstractSyntaxTree, AstError> {
        let peek = Self::expand_recursive(self.peek)?;
        Ok(Self::from_node(peek))
    }

    /// A recursive helper that traverses the tree bottom-up (post-order).
    fn expand_recursive(node: AstNode) -> Result<AstNode, AstError> {
        match node {
            // 1. Recursively process children first (post-order traversal)
            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } => {
                let expanded_left = Self::expand_recursive(*left)?;
                let expanded_right = Self::expand_recursive(*right)?;

                // 2. Analyze the current node AFTER its children are processed
                match operation {
                    // 3. We are ONLY interested in `Multiply` nodes
                    BinaryOperationKind::Multiply => {
                        // Case 1: A * (B + C)
                        if let AstNode::BinaryOperation {
                            operation: BinaryOperationKind::Plus,
                            left: b,  // B
                            right: c, // C
                        } = &expanded_right
                        {
                            // Create new nodes: (A*B) and (A*C)
                            let new_left = AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Multiply,
                                left: Box::new(expanded_left.clone()), // A
                                right: b.clone(),                      // B
                            };
                            let new_right = AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Multiply,
                                left: Box::new(expanded_left), // A
                                right: c.clone(),              // C
                            };

                            // Return the new node: (A*B) + (A*C)
                            // We MUST recursively call expand_recursive on the result,
                            // in case B or C were also expressions that need expanding.
                            return Self::expand_recursive(AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Plus,
                                left: Box::new(new_left),
                                right: Box::new(new_right),
                            });
                        }

                        // Case 2: (A + B) * C
                        if let AstNode::BinaryOperation {
                            operation: BinaryOperationKind::Plus,
                            left: a,  // A
                            right: b, // B
                        } = &expanded_left
                        {
                            // Create new nodes: (A*C) and (B*C)
                            let new_left = AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Multiply,
                                left: a.clone(), // A
                                right: Box::new(expanded_right.clone()), // C
                            };
                            let new_right = AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Multiply,
                                left: b.clone(),                 // B
                                right: Box::new(expanded_right), // C
                            };

                            // Return the new node: (A*C) + (B*C)
                            return Self::expand_recursive(AstNode::BinaryOperation {
                                operation: BinaryOperationKind::Plus,
                                left: Box::new(new_left),
                                right: Box::new(new_right),
                            });
                        }

                        // If no 'Plus' child was found, return the Multiply node as is
                        Ok(AstNode::BinaryOperation {
                            operation: BinaryOperationKind::Multiply,
                            left: Box::new(expanded_left),
                            right: Box::new(expanded_right),
                        })
                    },
                    // Return other nodes (Plus, etc.) as is
                    _ => Ok(AstNode::BinaryOperation {
                        operation,
                        left: Box::new(expanded_left),
                        right: Box::new(expanded_right),
                    }),
                }
            },
            // Also recurse into UnaryOperations
            AstNode::UnaryOperation {
                operation,
                expression,
            } => Ok(AstNode::UnaryOperation {
                operation,
                expression: Box::new(Self::expand_recursive(*expression)?),
            }),
            // Base cases (Number, Identifier)
            _ => Ok(node),
        }
    }
}
