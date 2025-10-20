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
    let lexemes_result = Lexer::new(tokens).run();
    let lexemes = match lexer::report(lexemes_result) {
        Ok((lexemes, lexer_report)) => {
            report = report.add(&lexer_report);
            lexemes
        },
        Err(lexer_report) => return report.add(&lexer_report),
    };

    // AST Generation
    let ast_result = ast::AstParser::new(&lexemes).parse();
    let ast = match ast::report(ast_result) {
        Ok((ast, ast_report)) => {
            report = report.add(&ast_report);
            ast
        },
        Err(ast_report) => return report.add(&ast_report),
    };

    report
}

pub mod ast;
pub mod lexer;
pub mod syntax;
pub mod tokenizer;
