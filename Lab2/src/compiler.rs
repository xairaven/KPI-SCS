use crate::compiler::ast::tree::{AbstractSyntaxTree, AstParser};
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
    let ast_result = AstParser::new(lexemes).parse();
    let ast = match ast_result {
        Ok(ast) => {
            ast::tree::report_success(&ast);
            ast
        },
        Err(error) => {
            ast::tree::report_error(error);
            return;
        },
    };
    // AST Math Optimization, #1
    let ast = match compute_run(ast, 1) {
        Ok(ast) => ast,
        Err(_) => return,
    };
    // AST Parallelization
    let ast_result = ast.transform();
    let ast = match ast_result {
        Ok(ast) => {
            ast::transform::report_success(&ast);
            ast
        },
        Err(error) => {
            ast::transform::report_error(error);
            return;
        },
    };
    // AST Math Optimization, #2
    let ast = match compute_run(ast, 2) {
        Ok(ast) => ast,
        Err(_) => return,
    };
    // AST Balancing
    let ast_result = ast.balance();
    let _ast = match ast_result {
        Ok(ast) => {
            ast::balancer::report_success(&ast);
            ast
        },
        Err(error) => {
            ast::balancer::report_error(error);
            return;
        },
    };
}

pub fn compute_run(
    tree: AbstractSyntaxTree, number: u8,
) -> Result<AbstractSyntaxTree, ()> {
    // AST Math Optimization
    let ast_result = tree.compute();
    let ast = match ast_result {
        Ok(ast) => {
            ast::math::report_success(&ast, number);
            ast
        },
        Err(error) => {
            ast::math::report_error(error, number);
            return Err(());
        },
    };
    if ast::math::check_finalization(&ast) {
        return Err(());
    }
    Ok(ast)
}

pub mod ast {
    pub mod balancer;
    pub mod math;
    pub mod transform;
    pub mod tree;
}
pub mod lexer;
pub mod syntax;
pub mod tokenizer;
