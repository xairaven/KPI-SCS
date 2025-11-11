use crate::compiler::ast::tree::{AbstractSyntaxTree, AstNode, BinaryOperationKind};

// New type alias for node path
// 0 = left, 1 = right, 2 = expression (for Unary)
type NodePath = Vec<u8>;

impl AbstractSyntaxTree {
    /// Returns a vector of AbstractSyntaxTree, each representing a single-step expansion.
    pub fn get_all_single_step_expansions(&self) -> Vec<AbstractSyntaxTree> {
        let mut expandable_nodes_paths: Vec<NodePath> = Vec::new();
        // 1. Find all expandable nodes' paths
        Self::find_expandable_nodes_recursive(
            &self.peek,
            &mut vec![],
            &mut expandable_nodes_paths,
        );

        let mut all_forms = Vec::new();

        // 2. For each found path...
        for path in expandable_nodes_paths {
            let mut new_tree = self.clone();
            if let Some(target_node) =
                Self::get_node_mut_by_path(&mut new_tree.peek, &path)
            {
                // `perform_expansion` replaces the node with its unfolded form
                *target_node = Self::perform_expansion(target_node.clone());
            }
            all_forms.push(new_tree);
        }

        all_forms
    }

    /// Recursively finds nodes matching the pattern `A * (B +/- C)` or `(A +/- B) * C`
    fn find_expandable_nodes_recursive(
        node: &AstNode, path: &mut NodePath, paths: &mut Vec<NodePath>,
    ) {
        if let AstNode::BinaryOperation {
            operation: BinaryOperationKind::Multiply,
            left,
            right,
        } = node
        {
            // Pattern 1: (A +/- B) * C
            if let AstNode::BinaryOperation { operation: op, .. } = left.as_ref()
                && (*op == BinaryOperationKind::Plus || *op == BinaryOperationKind::Minus)
            {
                paths.push(path.clone());
            }
            // Pattern 2: A * (B +/- C)
            if let AstNode::BinaryOperation { operation: op, .. } = right.as_ref()
                && (*op == BinaryOperationKind::Plus || *op == BinaryOperationKind::Minus)
            {
                paths.push(path.clone());
            }
        }

        // Recursive traversal
        match node {
            AstNode::BinaryOperation { left, right, .. } => {
                path.push(0); // 0 = left
                Self::find_expandable_nodes_recursive(left, path, paths);
                path.pop();

                path.push(1); // 1 = right
                Self::find_expandable_nodes_recursive(right, path, paths);
                path.pop();
            },
            AstNode::UnaryOperation { expression, .. } => {
                path.push(2); // 2 = expression
                Self::find_expandable_nodes_recursive(expression, path, paths);
                path.pop();
            },
            // Basic cases: nowhere to go
            _ => {},
        }
    }

    /// Helper function to get mutable reference to a node by path
    pub fn get_node_mut_by_path<'a>(
        node: &'a mut AstNode, path: &[u8],
    ) -> Option<&'a mut AstNode> {
        let mut current = node;
        for &index in path {
            match current {
                AstNode::BinaryOperation { left, right, .. } => {
                    current = if index == 0 { left } else { right };
                },
                AstNode::UnaryOperation { expression, .. } => {
                    current = expression;
                },
                _ => return None, // Шлях недійсний
            }
        }
        Some(current)
    }

    /// Performs ONE unfolding on a node that is *guaranteed* to be `Multiply`
    fn perform_expansion(node: AstNode) -> AstNode {
        if let AstNode::BinaryOperation {
            operation: BinaryOperationKind::Multiply,
            left,
            right,
        } = node
        {
            // Case 1: A * (B +/- C)
            if let AstNode::BinaryOperation {
                operation: op @ (BinaryOperationKind::Plus | BinaryOperationKind::Minus),
                left: b,
                right: c,
            } = *right
            {
                let new_left = AstNode::BinaryOperation {
                    operation: BinaryOperationKind::Multiply,
                    left: left.clone(), // A
                    right: b,           // B
                };
                let new_right = AstNode::BinaryOperation {
                    operation: BinaryOperationKind::Multiply,
                    left,     // A
                    right: c, // C
                };
                // Повертаємо (A*B) +/- (A*C)
                return AstNode::BinaryOperation {
                    operation: op,
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                };
            }

            // Case 2: (A +/- B) * C
            if let AstNode::BinaryOperation {
                operation: op @ (BinaryOperationKind::Plus | BinaryOperationKind::Minus),
                left: a,
                right: b,
            } = *left
            {
                let new_left = AstNode::BinaryOperation {
                    operation: BinaryOperationKind::Multiply,
                    left: a,              // A
                    right: right.clone(), // C
                };
                let new_right = AstNode::BinaryOperation {
                    operation: BinaryOperationKind::Multiply,
                    left: b, // B
                    right,   // C
                };
                // Повертаємо (A*C) +/- (B*C)
                return AstNode::BinaryOperation {
                    operation: op,
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                };
            }

            // If something went wrong, return as is
            return AstNode::BinaryOperation {
                operation: BinaryOperationKind::Multiply,
                left,
                right,
            };
        }

        // Return the node as is if it's not `Multiply`
        node
    }
}
