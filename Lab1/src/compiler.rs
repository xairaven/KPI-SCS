use crate::compiler::syntax::{SyntaxAnalyzer, SyntaxError};
use colored::Colorize;

pub fn compile(source: &str) -> String {
    let tokens = tokenizer::tokenize(source);
    let syntax_errors = SyntaxAnalyzer::new(tokens).analyze();

    report(source, &syntax_errors)
}

fn report(source: &str, syntax_errors: &[SyntaxError]) -> String {
    let mut result = String::new();

    let first_line = match syntax_errors.len() {
        0 => format!("{}: {}\n", "Analysis result".bold(), "OK!".bold().green()),
        n => {
            format!(
                "{}: Found {} {}.\n",
                "Analysis result".bold(),
                n.to_string().red(),
                "errors".red()
            )
        },
    };
    result.push_str(&first_line);

    result.push_str(&format!("\n{}:\n", "Code".bold().yellow()));
    result.push_str(source);

    for error in syntax_errors {
        // TODO: formatting
    }

    result
}

pub mod syntax;
pub mod tokenizer;
