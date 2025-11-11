use crate::compiler::ast::tree::{
    AbstractSyntaxTree, AstError, AstNode, BinaryOperationKind,
};
use std::collections::VecDeque;

type NodePath = Vec<u8>;

impl AbstractSyntaxTree {
    /// Returns a VECTOR of trees, where each tree is ONE step of the parentheses.
    pub fn get_all_single_step_factorings(&self) -> Vec<AbstractSyntaxTree> {
        let mut factorable_nodes_paths: Vec<NodePath> = Vec::new();
        // 1. Find paths to *all* `Plus` or `Minus` nodes
        Self::find_factorable_nodes_recursive(
            &self.peek,
            &mut vec![],
            &mut factorable_nodes_paths,
        );

        let mut all_forms = Vec::new();

        // 2. For each node found...
        for path in factorable_nodes_paths {
            let mut new_tree = self.clone();
            // ...and try to apply the extrude *only* to this node
            if let Some(target_node) =
                Self::get_node_mut_by_path(&mut new_tree.peek, &path)
            {
                // `perform_factoring` tries to group the terms
                // and *replaces* `target_node` with the new, grouped version
                let original_node = target_node.clone();
                *target_node = Self::perform_factoring(target_node.clone());

                // If `perform_factoring` changed something, add it to the list
                if original_node != *target_node {
                    all_forms.push(new_tree);
                }
            }
        }
        all_forms
    }

    /// (Local version) Recursively "unfolds" a chain of associative operations.
    fn local_collect_operands(
        node: AstNode, op_kind: BinaryOperationKind, operands: &mut Vec<AstNode>,
    ) {
        match node {
            // If the node is the same operation (Plus or Minus)...
            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } if operation == op_kind => {
                // ...recursively collect operands from both sides.
                Self::local_collect_operands(*left, op_kind.clone(), operands);
                Self::local_collect_operands(*right, op_kind.clone(), operands);
            },
            // If it's a `Plus` inside a `Minus` (or vice versa), we also add it.
            // This is needed for `a - (b + c)` -> `[a, -(b+c)]`
            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } if (op_kind == BinaryOperationKind::Minus
                && operation == BinaryOperationKind::Plus)
                || (op_kind == BinaryOperationKind::Plus
                    && operation == BinaryOperationKind::Minus) =>
            {
                Self::local_collect_operands(*left, op_kind.clone(), operands);

                // If we collect `Plus` and the node `Minus`, then the right operand
                // must become negative.
                if op_kind == BinaryOperationKind::Plus
                    && operation == BinaryOperationKind::Minus
                {
                    Self::local_collect_operands(
                        AstNode::UnaryOperation {
                            operation:
                                crate::compiler::ast::tree::UnaryOperationKind::Minus,
                            expression: right,
                        },
                        op_kind,
                        operands,
                    );
                } else {
                    Self::local_collect_operands(*right, op_kind, operands);
                }
            },
            _ => {
                operands.push(node);
            },
        }
    }

    /// Building balanced tree
    fn local_build_balanced_tree(
        operands: Vec<AstNode>, op_kind: BinaryOperationKind,
    ) -> Result<AstNode, AstError> {
        if operands.is_empty() {
            return Err(AstError::CannotBuildEmptyTree);
        }
        let mut queue: VecDeque<AstNode> = operands.into();
        while queue.len() > 1 {
            let level_size = queue.len();
            for _ in 0..(level_size / 2) {
                let left = queue.pop_front().ok_or(AstError::FailedPopFromQueue)?;
                let right = queue.pop_front().ok_or(AstError::FailedPopFromQueue)?;
                let new_node = AstNode::BinaryOperation {
                    operation: op_kind.clone(),
                    left: Box::new(left),
                    right: Box::new(right),
                };
                queue.push_back(new_node);
            }
            if !level_size.is_multiple_of(2) {
                let odd_one_out =
                    queue.pop_front().ok_or(AstError::FailedPopFromQueue)?;
                queue.push_back(odd_one_out);
            }
        }
        queue.pop_front().ok_or(AstError::FailedPopFromQueue)
    }

    /// Recursively searches for nodes that are `Plus` or `Minus`
    fn find_factorable_nodes_recursive(
        node: &AstNode, path: &mut NodePath, paths: &mut Vec<NodePath>,
    ) {
        if let AstNode::BinaryOperation { operation: op, .. } = node
            && (*op == BinaryOperationKind::Plus || *op == BinaryOperationKind::Minus)
        {
            paths.push(path.clone());
        }

        // Recursive traversal
        match node {
            AstNode::BinaryOperation { left, right, .. } => {
                path.push(0); // 0 = left
                Self::find_factorable_nodes_recursive(left, path, paths);
                path.pop();

                path.push(1); // 1 = right
                Self::find_factorable_nodes_recursive(right, path, paths);
                path.pop();
            },
            AstNode::UnaryOperation { expression, .. } => {
                path.push(2); // 2 = expression
                Self::find_factorable_nodes_recursive(expression, path, paths);
                path.pop();
            },
            _ => {},
        }
    }

    /// Performs ONE factorization on a node that is *guaranteed* to be `Plus` or `Minus`
    fn perform_factoring(node: AstNode) -> AstNode {
        let op_kind = match &node {
            AstNode::BinaryOperation { operation, .. } => operation.clone(),
            _ => return node, // Не `Plus` або `Minus`, нічого робити
        };

        // --- STEP 1: Expand ---
        let mut summands = Vec::new();
        Self::local_collect_operands(node, op_kind.clone(), &mut summands);

        if summands.len() < 2 {
            return Self::local_build_balanced_tree(summands, op_kind)
                .unwrap_or(AstNode::Number(0.0));
        }

        // --- STEP 2: FIND COMMON MULTIPLIERS (O(N^2)) ---
        // (HashMap doesn't work with f64, so we use O(N^2))
        let mut new_summands = Vec::new();
        let mut processed = vec![false; summands.len()];

        for i in 0..summands.len() {
            if processed[i] {
                continue;
            }

            let factors_i = Self::get_factors(&summands[i]);
            if factors_i.is_empty() {
                new_summands.push(summands[i].clone());
                processed[i] = true;
                continue;
            }

            let mut best_group: (Option<AstNode>, Vec<AstNode>) = (None, vec![]);

            // We iterate over each factor of the current term
            for factor in &factors_i {
                if let Some(remainder_i) = Self::get_remainder(&summands[i], factor) {
                    let mut current_group_terms = vec![remainder_i]; // [b]
                    let mut current_group_indices = vec![i];

                    // We look for this factor in other terms
                    for j in (i + 1)..summands.len() {
                        if processed[j] {
                            continue;
                        }
                        if let Some(remainder_j) =
                            Self::get_remainder(&summands[j], factor)
                        {
                            current_group_terms.push(remainder_j); // [b, c]
                            current_group_indices.push(j);
                        }
                    }

                    // If we find a group, we save it
                    if current_group_terms.len() > 1 {
                        // Found the *first* possible group, exit.
                        // This guarantees "one step"
                        best_group = (Some(factor.clone()), current_group_terms);
                        for &idx in &current_group_indices {
                            processed[idx] = true;
                        }
                        break;
                    }
                }
                if best_group.0.is_some() {
                    break;
                }
            }

            // --- STEP 3: BUILD A NEW NODE ---
            if let (Some(factor), terms) = best_group {
                // Found a group, for example a*b + a*c
                let sum_of_terms =
                    Self::local_build_balanced_tree(terms, BinaryOperationKind::Plus)
                        .unwrap_or(AstNode::Number(0.0));

                let new_node = AstNode::BinaryOperation {
                    operation: BinaryOperationKind::Multiply,
                    left: Box::new(factor),
                    right: Box::new(sum_of_terms),
                };
                new_summands.push(new_node);
            } else if !processed[i] {
                // Group not found, return term as is
                new_summands.push(summands[i].clone());
                processed[i] = true;
            }
        }

        // Add all remaining terms
        for (i, p) in processed.iter().enumerate() {
            if !p {
                new_summands.push(summands[i].clone());
            }
        }

        // --- STEP 4: FINAL ASSEMBLY ---
        Self::local_build_balanced_tree(new_summands, op_kind)
            .unwrap_or(AstNode::Number(0.0))
    }

    /// Auxiliary function: finds the factors for the term.
    fn get_factors(node: &AstNode) -> Vec<AstNode> {
        if let AstNode::BinaryOperation {
            operation: BinaryOperationKind::Multiply,
            left,
            right,
        } = node
        {
            vec![*left.clone(), *right.clone()]
        } else {
            vec![] // Not multiplication, no factors
        }
    }

    /// Helper function: for `node` (A*B) and `factor` (A), returns `Some(B)`.
    fn get_remainder(node: &AstNode, factor: &AstNode) -> Option<AstNode> {
        if let AstNode::BinaryOperation {
            operation: BinaryOperationKind::Multiply,
            left,
            right,
        } = node
        {
            if left.as_ref() == factor {
                return Some(*right.clone());
            }
            if right.as_ref() == factor {
                return Some(*left.clone());
            }
        }
        None
    }
}
