use crate::compiler::ast::balancer::AstBalancerReporter;
use crate::compiler::ast::folding::AstFolderReporter;
use crate::compiler::ast::math::AstComputerReporter;
use crate::compiler::ast::transform::AstTransformerReporter;
use crate::compiler::ast::tree::{AbstractSyntaxTree, AstError, AstParser, AstReporter};
use crate::compiler::lexer::{Lexeme, Lexer, LexerError, LexerReporter};
use crate::compiler::syntax::{SyntaxAnalyzer, SyntaxError, SyntaxReporter};
use crate::compiler::tokenizer::{Token, Tokenizer};
use crate::config::Config;

pub struct CompilerContext {
    pub code: String,
    pub pretty_output: bool,
}

impl CompilerContext {
    pub fn new(config: &Config) -> Self {
        Self {
            code: String::new(),
            pretty_output: config.pretty_output,
        }
    }

    fn tokenize(&self) -> Vec<Token> {
        Tokenizer::process(&self.code)
    }

    pub fn tokenize_report(&self) -> String {
        Tokenizer::report(&self.tokenize())
    }

    fn check_syntax(&self) -> Vec<SyntaxError> {
        let tokens = self.tokenize();
        SyntaxAnalyzer::new(&tokens).analyze()
    }

    pub fn syntax_report(&self) -> String {
        SyntaxReporter::new(&self.code, &self.check_syntax(), self.pretty_output).report()
    }

    fn create_lexemes(&self) -> Result<Result<Vec<Lexeme>, LexerError>, String> {
        let tokens = self.tokenize();
        let syntax_errors = self.check_syntax();
        if !syntax_errors.is_empty() {
            return Err(SyntaxReporter::new(
                &self.code,
                &syntax_errors,
                self.pretty_output,
            )
            .report());
        }
        let lexemes = Lexer::new(tokens).run();
        Ok(lexemes)
    }

    pub fn lexer_report(&self) -> String {
        match self.create_lexemes() {
            Ok(lexer_result) => LexerReporter::report(&lexer_result),
            Err(syntax_error) => syntax_error,
        }
    }

    fn create_ast(&self) -> Result<Result<AbstractSyntaxTree, AstError>, String> {
        let lexer_result = self.create_lexemes()?;
        let lexemes = match lexer_result {
            Ok(value) => value,
            Err(_) => return Err(LexerReporter::report(&lexer_result)),
        };

        Ok(AstParser::new(lexemes).parse())
    }

    pub fn ast_report(&self) -> String {
        match self.create_ast() {
            Ok(ast_result) => AstReporter::report(&ast_result),
            Err(error) => error,
        }
    }

    fn compute_ast_1(&self) -> Result<Result<AbstractSyntaxTree, AstError>, String> {
        let ast_creation_result = self.create_ast()?;
        let ast = match ast_creation_result {
            Ok(value) => value,
            Err(_) => return Err(AstReporter::report(&ast_creation_result)),
        };

        Ok(ast.compute())
    }

    pub fn compute_1_report(&self) -> String {
        match self.compute_ast_1() {
            Ok(compute_result) => AstComputerReporter::report(&compute_result, 1),
            Err(error) => error,
        }
    }

    fn transform_ast(&self) -> Result<Result<AbstractSyntaxTree, AstError>, String> {
        let ast_compute_result = self.compute_ast_1()?;
        let ast = match ast_compute_result {
            Ok(value) => value,
            Err(_) => return Err(AstComputerReporter::report(&ast_compute_result, 1)),
        };

        if ast.is_finalized() {
            return Err(AstComputerReporter::report_finalization());
        }

        Ok(ast.transform())
    }

    pub fn transform_report(&self) -> String {
        match self.transform_ast() {
            Ok(transform_result) => AstTransformerReporter::report(&transform_result),
            Err(error) => error,
        }
    }

    fn compute_ast_2(&self) -> Result<Result<AbstractSyntaxTree, AstError>, String> {
        let ast_transformation_result = self.transform_ast()?;
        let ast = match ast_transformation_result {
            Ok(value) => value,
            Err(_) => {
                return Err(AstTransformerReporter::report(&ast_transformation_result));
            },
        };

        Ok(ast.compute())
    }

    pub fn compute_2_report(&self) -> String {
        match self.compute_ast_2() {
            Ok(compute_result) => AstComputerReporter::report(&compute_result, 2),
            Err(error) => error,
        }
    }

    fn balance_ast(&self) -> Result<Result<AbstractSyntaxTree, AstError>, String> {
        let ast_compute_result = self.compute_ast_2()?;
        let ast = match ast_compute_result {
            Ok(value) => value,
            Err(_) => return Err(AstComputerReporter::report(&ast_compute_result, 2)),
        };

        if ast.is_finalized() {
            return Err(AstComputerReporter::report_finalization());
        }

        Ok(ast.balance())
    }

    pub fn balance_report(&self) -> String {
        match self.balance_ast() {
            Ok(balance_result) => AstBalancerReporter::report(&balance_result),
            Err(error) => error,
        }
    }

    fn compute_ast_3(&self) -> Result<Result<AbstractSyntaxTree, AstError>, String> {
        let ast_balance_result = self.balance_ast()?;
        let ast = match ast_balance_result {
            Ok(value) => value,
            Err(_) => return Err(AstBalancerReporter::report(&ast_balance_result)),
        };

        Ok(ast.compute())
    }

    pub fn compute_3_report(&self) -> String {
        match self.compute_ast_3() {
            Ok(compute_result) => AstComputerReporter::report(&compute_result, 3),
            Err(error) => error,
        }
    }

    fn folding_ast(&self) -> Result<Result<AbstractSyntaxTree, AstError>, String> {
        let ast_compute_result = self.compute_ast_3()?;
        let ast = match ast_compute_result {
            Ok(value) => value,
            Err(_) => return Err(AstComputerReporter::report(&ast_compute_result, 3)),
        };

        if ast.is_finalized() {
            return Err(AstComputerReporter::report_finalization());
        }

        Ok(ast.fold())
    }

    pub fn folding_report(&self) -> String {
        match self.folding_ast() {
            Ok(folding_result) => AstFolderReporter::report(&folding_result),
            Err(error) => error,
        }
    }

    fn compute_ast_4(&self) -> Result<Result<AbstractSyntaxTree, AstError>, String> {
        let ast_folding_result = self.folding_ast()?;
        let ast = match ast_folding_result {
            Ok(value) => value,
            Err(_) => return Err(AstFolderReporter::report(&ast_folding_result)),
        };

        Ok(ast.compute())
    }

    pub fn compute_4_report(&self) -> String {
        match self.compute_ast_4() {
            Ok(compute_result) => AstComputerReporter::report(&compute_result, 4),
            Err(error) => error,
        }
    }
}
