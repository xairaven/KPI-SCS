use crate::compiler::lexer::Lexer;
use crate::compiler::syntax::SyntaxAnalyzer;
use std::ops::Add;

pub fn compile(source: &str, is_pretty: bool) -> String {
    let mut report = String::new();

    // Lexical Analysis
    let tokens = tokenizer::tokenize(source);
    // Syntax Analysis
    let syntax_errors = SyntaxAnalyzer::new(&tokens).analyze();
    let is_syntax_analysis_successful = syntax_errors.is_empty();
    let syntax_report = syntax::report(source, syntax_errors, is_pretty);
    if !is_syntax_analysis_successful {
        return syntax_report;
    } else {
        report = report.add(&syntax_report);
    };

    // Making lexemes
    let lexemes = match Lexer::new(tokens).run() {
        Ok(data) => data,
        Err(error) => {
            let lexer_report = format!("Lexer error: {}", error);
            return report.add(&lexer_report);
        },
    };

    report
}

pub mod lexer;
pub mod syntax;
pub mod tokenizer;
