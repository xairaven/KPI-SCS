use crate::compiler::ast::tree::AbstractSyntaxTree;
use crate::compiler::reports::Reporter;
use crate::utils::StringBuffer;
use std::collections::{HashSet, VecDeque};

impl AbstractSyntaxTree {
    pub fn find_equivalent_forms(&self) -> Vec<AbstractSyntaxTree> {
        // BFS
        let mut queue: VecDeque<AbstractSyntaxTree> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut all_forms: Vec<AbstractSyntaxTree> = Vec::new();

        let initial_key = self.to_canonical_string();

        queue.push_back(self.clone());
        visited.insert(initial_key);
        all_forms.push(self.clone());

        // Running the search for equivalent forms
        while let Some(current_ast) = queue.pop_front() {
            // Distributive Law (Expanding parentheses)
            if let Ok(expanded_ast) = current_ast.clone().expand() {
                let key = expanded_ast.to_canonical_string();

                // Is this form unique?
                if !visited.contains(&key) {
                    visited.insert(key);
                    all_forms.push(expanded_ast.clone());
                    queue.push_back(expanded_ast);
                }
            }

            if let Ok(factored_ast) = current_ast.clone().factor() {
                let key = factored_ast.to_canonical_string();

                // Is this form unique?
                if !visited.contains(&key) {
                    visited.insert(key);
                    all_forms.push(factored_ast.clone());
                    queue.push_back(factored_ast);
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
