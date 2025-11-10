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

    pub fn create_ast(&self) -> Result<Result<AbstractSyntaxTree, AstError>, String> {
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
}
