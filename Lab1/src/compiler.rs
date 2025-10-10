pub fn compile(source: &str) -> String {
    let tokens = tokenizer::tokenize(source);
    let syntax_errors = syntax::analyze(tokens);
    dbg!(syntax_errors);

    // TODO: Implement lexical analysis and syntax analysis here
    source.to_owned()
}

pub mod syntax;
pub mod tokenizer;
