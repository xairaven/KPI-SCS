pub fn compile(source: &str) -> String {
    let tokens = tokenizer::tokenize(source);
    dbg!(tokens);

    // TODO: Implement lexical analysis and syntax analysis here
    source.to_owned()
}

pub mod tokenizer;
