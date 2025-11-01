use crate::compiler::lexer::Lexer;
use crate::compiler::syntax::SyntaxAnalyzer;

pub fn compile(source: &str, is_pretty: bool) {
    // Lexical Analysis
    let tokens = tokenizer::tokenize(source);
    // Syntax Analysis
    let syntax_errors = SyntaxAnalyzer::new(&tokens).analyze();
    let is_syntax_analysis_successful = syntax_errors.is_empty();
    syntax::report(source, syntax_errors, is_pretty);
    if !is_syntax_analysis_successful {
        return;
    }

    // Making lexemes
    let lexemes_result = Lexer::new(tokens).run();
    let lexemes = match lexemes_result {
        Ok(lexemes) => {
            lexer::report_success(&lexemes);
            lexemes
        },
        Err(error) => {
            lexer::report_error(error);
            return;
        },
    };

    // AST Generation
    let ast_result = ast::AstParser::new(lexemes).parse();
    let ast = match ast_result {
        Ok(ast) => {
            ast::report_success(&ast);
            ast
        },
        Err(error) => {
            ast::report_error(error);
            return;
        },
    };
}

pub mod ast;
pub mod lexer;
pub mod syntax;
pub mod tokenizer;
