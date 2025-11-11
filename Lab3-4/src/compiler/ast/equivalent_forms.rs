use crate::compiler::ast::tree::AbstractSyntaxTree;
use crate::compiler::reports::Reporter;
use crate::utils::StringBuffer;
use std::collections::{HashSet, VecDeque};

impl AbstractSyntaxTree {
    pub fn find_equivalent_forms(&self) -> Vec<AbstractSyntaxTree> {
        let mut all_forms: Vec<AbstractSyntaxTree> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut expansion_queue: VecDeque<AbstractSyntaxTree> = VecDeque::new();

        // --- Stage 1: Expansion (Distributive Law, Nodes 0-7 from memo) ---
        // This stage finds all forms *only* by expanding parentheses.
        // It performs a Breadth-First Search (BFS) starting from the original expression.

        let initial_key = self.to_canonical_string();
        expansion_queue.push_back(self.clone());
        visited.insert(initial_key);
        all_forms.push(self.clone());

        let mut fully_expanded_node: Option<AbstractSyntaxTree> = None;

        while let Some(current_ast) = expansion_queue.pop_front() {
            // Get all possible next forms by applying *one step* of expansion
            let expansion_steps = current_ast.get_all_single_step_expansions();

            // If a node has no possible expansions, it's a "leaf" in this stage.
            // We assume the first one we find is the fully expanded form (Node 7).
            if expansion_steps.is_empty() && fully_expanded_node.is_none() {
                fully_expanded_node = Some(current_ast.clone());
            }

            for expanded_ast in expansion_steps {
                let key = expanded_ast.to_canonical_string();
                // Check if we've already seen this form to avoid cycles and duplicates
                if !visited.contains(&key) {
                    visited.insert(key.clone());
                    all_forms.push(expanded_ast.clone());
                    expansion_queue.push_back(expanded_ast);
                }
            }
        }

        // --- Stage 2: Factoring (Associative Law, Nodes 7-13 from memo) ---
        // This stage starts *only* with the fully expanded form (Node 7)
        // and *only* applies factoring (collapsing terms).

        let Some(start_node_for_factoring) = fully_expanded_node else {
            // This can happen if the original expression had no parentheses to expand.
            // For the example `(a-c)*k...`, this node will always be found.
            log::warn!(
                "No fully expanded form (Node 7) found. Skipping Stage 2 (Factoring)."
            );
            return all_forms;
        };

        let mut factoring_queue: VecDeque<AbstractSyntaxTree> = VecDeque::new();
        // We don't need to add `start_node_for_factoring` to `all_forms` or `visited` again,
        // as it was already added in Stage 1.
        factoring_queue.push_back(start_node_for_factoring);

        // This BFS explores all forms reachable by *factoring*
        while let Some(current_ast) = factoring_queue.pop_front() {
            // Get all possible next forms by applying *one step* of factoring
            let factoring_steps = current_ast.get_all_single_step_factorings();

            for factored_ast in factoring_steps {
                let key = factored_ast.to_canonical_string();
                if !visited.contains(&key) {
                    // We continue using the *same* `visited` set to prevent
                    // re-discovering forms from Stage 1.
                    visited.insert(key.clone());
                    all_forms.push(factored_ast.clone());
                    factoring_queue.push_back(factored_ast);
                }
            }
        }

        all_forms
    }
}

impl Reporter {
    pub fn finding_equivalent_form(&self, forms: &[String]) -> String {
        let mut buffer = StringBuffer::default();

        buffer.add_line(format!("Found {} equivalent forms!\n", forms.len()));

        for (index, form) in forms.iter().enumerate() {
            buffer.add_line(format!("{}) {}", index + 1, form));
        }

        buffer.get()
    }
}
