use crate::compiler::lexer::{Lexer, LexerReporter};
use crate::compiler::syntax::{SyntaxAnalyzer, SyntaxReporter};
use crate::compiler::tokenizer::Tokenizer;
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

    pub fn tokenize_report(&self) -> String {
        let tokens = Tokenizer::process(&self.code);
        Tokenizer::report(&tokens)
    }

    pub fn syntax_report(&self) -> String {
        let tokens = Tokenizer::process(&self.code);
        let syntax_errors = SyntaxAnalyzer::new(&tokens).analyze();
        SyntaxReporter::new(&self.code, &syntax_errors, self.pretty_output).report()
    }

    pub fn lexer_report(&self) -> String {
        let tokens = Tokenizer::process(&self.code);
        let syntax_errors = SyntaxAnalyzer::new(&tokens).analyze();
        if !syntax_errors.is_empty() {
            return SyntaxReporter::new(&self.code, &syntax_errors, self.pretty_output)
                .report();
        }

        let lexemes = Lexer::new(tokens).run();
        LexerReporter::report(&lexemes)
    }
}
