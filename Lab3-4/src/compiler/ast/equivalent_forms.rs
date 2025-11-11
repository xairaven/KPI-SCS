// xairaven/kpi-scs/KPI-SCS-main/Lab3-4/src/compiler/ast/equivalent_forms.rs

use crate::compiler::ast::tree::AbstractSyntaxTree;
use crate::compiler::reports::Reporter;
use crate::utils::StringBuffer;
use std::collections::{HashSet, VecDeque};

impl AbstractSyntaxTree {
    pub fn find_equivalent_forms(&self) -> Vec<AbstractSyntaxTree> {
        let mut all_forms: Vec<AbstractSyntaxTree> = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut expansion_queue: VecDeque<AbstractSyntaxTree> = VecDeque::new();

        // --- Stage 1: Expansion (Distributive Law, Nodes 0-7) ---
        // This stage finds all forms *only* by expanding parentheses.

        let initial_key = self.to_canonical_string();
        expansion_queue.push_back(self.clone());
        visited.insert(initial_key);
        all_forms.push(self.clone());

        let mut fully_expanded_node: Option<AbstractSyntaxTree> = None;

        while let Some(current_ast) = expansion_queue.pop_front() {
            let expansion_steps = current_ast.get_all_single_step_expansions();

            if expansion_steps.is_empty() && fully_expanded_node.is_none() {
                fully_expanded_node = Some(current_ast.clone());
            }

            for expanded_ast in expansion_steps {
                let key = expanded_ast.to_canonical_string();
                if !visited.contains(&key) {
                    visited.insert(key.clone());
                    all_forms.push(expanded_ast.clone());
                    expansion_queue.push_back(expanded_ast);
                }
            }
        }

        // --- Stage 2: Factoring (Associative Law, Nodes 7-13) ---
        // This stage starts *only* with the fully expanded form
        // and *only* applies factoring (collapse).

        let Some(start_node_for_factoring) = fully_expanded_node else {
            // This can happen if there was nothing to expand in the original expression.
            // In example `(a-c)*k...` this node will 100% be found.
            // If it is not there, we simply return the forms from Step 1.
            log::warn!(
                "No fully expanded form found (Node 7). Skipping Stage 2 (Factoring)."
            );
            return all_forms;
        };

        let mut factoring_queue: VecDeque<AbstractSyntaxTree> = VecDeque::new();
        // We don't need to add `start_node_for_factoring` to `all_forms` or `visited` again,
        // since it's already there from Step 1.
        factoring_queue.push_back(start_node_for_factoring);

        while let Some(current_ast) = factoring_queue.pop_front() {
            let factoring_steps = current_ast.get_all_single_step_factorings();

            for factored_ast in factoring_steps {
                let key = factored_ast.to_canonical_string();
                if !visited.contains(&key) {
                    // We are continuing to build on the same `visited` set
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
