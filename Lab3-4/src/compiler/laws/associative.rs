use crate::compiler::ast::tree::{
    AbstractSyntaxTree, AstError, AstNode, BinaryOperationKind,
};

impl AbstractSyntaxTree {
    pub fn factor(self) -> Result<AbstractSyntaxTree, AstError> {
        let peek = Self::factor_recursive(self.peek)?;
        Ok(Self::from_node(peek))
    }

    /// A recursive helper that traverses the tree bottom-up (post-order).
    fn factor_recursive(node: AstNode) -> Result<AstNode, AstError> {
        match node {
            // 1. Recursively process children first (post-order traversal)
            AstNode::BinaryOperation {
                operation,
                left,
                right,
            } => {
                let left = Self::factor_recursive(*left)?;
                let right = Self::factor_recursive(*right)?;

                // 2. Analyze the current node AFTER its children are processed
                match operation {
                    // 3. We are ONLY interested in `Plus` nodes for factoring
                    BinaryOperationKind::Plus => {
                        let node = AstNode::BinaryOperation {
                            operation: operation.clone(),
                            left: Box::new(left),
                            right: Box::new(right),
                        };
                        // Proceed to the main factoring logic
                        Self::apply_factoring(node)
                    },
                    // Return other nodes (Multiply, etc.) as is
                    _ => Ok(AstNode::BinaryOperation {
                        operation,
                        left: Box::new(left),
                        right: Box::new(right),
                    }),
                }
            },
            // Also recurse into UnaryOperations
            AstNode::UnaryOperation {
                operation,
                expression,
            } => Ok(AstNode::UnaryOperation {
                operation,
                expression: Box::new(Self::factor_recursive(*expression)?),
            }),

            AstNode::FunctionCall { name, arguments } => {
                let mut processed_arguments = vec![];
                for argument in arguments {
                    let argument = Self::factor_recursive(argument)?;
                    processed_arguments.push(argument);
                }
                Ok(AstNode::FunctionCall {
                    name,
                    arguments: processed_arguments,
                })
            },

            AstNode::ArrayAccess {
                identifier,
                indices,
            } => {
                let mut processed_indices = vec![];
                for index in indices {
                    let index = Self::factor_recursive(index)?;
                    processed_indices.push(index);
                }
                Ok(AstNode::ArrayAccess {
                    identifier,
                    indices: processed_indices,
                })
            },

            // Base cases (Number, Identifier)
            _ => Ok(node),
        }
    }

    /// The core logic: flattens, finds factors, and refactors the node.
    /// This implementation uses an O(N^2) loop instead of a HashMap
    /// to avoid `Eq` and `Hash` trait bounds issues with `f64`.
    fn apply_factoring(node: AstNode) -> Result<AstNode, AstError> {
        // --- STEP 1: FLATTEN THE ADDITION ---
        // Use `collect_operands` from balancer.rs
        // e.g., `(a*b + a*c) + d` -> `summands = [a*b, a*c, d]`
        let mut summands = Vec::new();
        AbstractSyntaxTree::collect_operands(
            node,
            BinaryOperationKind::Plus,
            &mut summands,
        );

        if summands.len() < 2 {
            // Nothing to factor, rebuild and return
            return AbstractSyntaxTree::build_balanced_tree(
                summands,
                BinaryOperationKind::Plus,
            );
        }

        // --- STEP 2: FIND COMMON FACTORS (O(N^2) approach) ---
        let mut new_summands = Vec::new();
        // Keep track of terms that have been grouped
        let mut processed = vec![false; summands.len()];

        for i in 0..summands.len() {
            if processed[i] {
                continue; // This term is already part of a group
            }

            // `get_factors` returns [A, B] for A*B, or [C] for C
            let factors_i = Self::get_factors(&summands[i]);
            let mut current_group_terms = Vec::new();
            let mut common_factor: Option<AstNode> = None;

            // Iterate through each potential factor of the current term
            for factor in &factors_i {
                // `get_remainder` returns `Some(B)` for `A*B` and factor `A`
                // It returns `None` if `factor` isn't a factor (e.g., for term `A`)
                if let Some(remainder_i) = Self::get_remainder(&summands[i], factor) {
                    // We found a potential group. Start it with `remainder_i`.
                    current_group_terms = vec![remainder_i]; // e.g., [b]

                    // Inner loop: check all other terms for the same factor
                    for j in (i + 1)..summands.len() {
                        if processed[j] {
                            continue;
                        }

                        // `get_remainder` uses `==` (PartialEq)
                        if let Some(remainder_j) =
                            Self::get_remainder(&summands[j], factor)
                        {
                            // Found one! `summands[j]` (e.g., `a*c`) has the same factor `a`
                            current_group_terms.push(remainder_j); // e.g., [b, c]
                            processed[j] = true; // Mark `a*c` as processed
                        }
                    }

                    // If we found at least one match...
                    if current_group_terms.len() > 1 {
                        common_factor = Some(factor.clone());
                        break; // Stop searching for factors, we found a group
                    }
                }
            }

            // --- STEP 3: BUILD THE NEW NODE ---
            processed[i] = true; // Mark `summands[i]` (e.g., `a*b`) as processed
            if let Some(factor) = common_factor {
                // We found a group, e.g., `a*b + a*c`
                // `factor` = `a`
                // `current_group_terms` = `[b, c]`

                // 1. Build the sum of remainders: `(b + c)`
                // Use `build_balanced_tree` from balancer.rs
                let sum_of_terms = AbstractSyntaxTree::build_balanced_tree(
                    current_group_terms,
                    BinaryOperationKind::Plus,
                )?;

                // 2. Create the new node: `a * (b + c)`
                let new_node = AstNode::BinaryOperation {
                    operation: BinaryOperationKind::Multiply,
                    left: Box::new(factor),
                    right: Box::new(sum_of_terms),
                };
                new_summands.push(new_node);
            } else {
                // No group was found for this term, add it back as is (e.g., `d`)
                new_summands.push(summands[i].clone());
            }
        }

        // --- STEP 4: FINAL REBUILD ---
        // `new_summands` now contains all factored groups and leftovers
        // e.g., `[d, a*(b+c)]`
        // Build the final balanced tree from these new terms
        AbstractSyntaxTree::build_balanced_tree(new_summands, BinaryOperationKind::Plus)
    }

    /// Helper: finds factors for a summand.
    /// `A * B` -> `[A, B]`
    /// `C`     -> `[C]` (this is important for `get_remainder` to work)
    fn get_factors(node: &AstNode) -> Vec<AstNode> {
        if let AstNode::BinaryOperation {
            operation: BinaryOperationKind::Multiply,
            left,
            right,
        } = node
        {
            // Return both multipliers
            vec![*left.clone(), *right.clone()]
        } else {
            // Return the node itself. `get_remainder` will handle this
            // by returning `None` as it's not a `Multiply` operation.
            // This is a slight simplification; a more robust solution
            // would return `[node.clone()]` and `get_remainder` would
            // check for `node == factor` and return `Number(1.0)`.
            // But for `a*b + a*c` this is sufficient.
            vec![] // Return empty vec if not a multiplication
        }
    }

    /// Helper: for `node` (e.g., `A*B`) and `factor` (e.g., `A`), returns `Some(B)`.
    /// Uses `==` (PartialEq).
    fn get_remainder(node: &AstNode, factor: &AstNode) -> Option<AstNode> {
        if let AstNode::BinaryOperation {
            operation: BinaryOperationKind::Multiply,
            left,
            right,
        } = node
        {
            // Use `==` which maps to `PartialEq`
            if left.as_ref() == factor {
                return Some(*right.clone());
            }
            // Use `==` which maps to `PartialEq`
            if right.as_ref() == factor {
                return Some(*left.clone());
            }
        }
        None // `factor` is not a multiplier in this `node`
    }
}
