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
}
