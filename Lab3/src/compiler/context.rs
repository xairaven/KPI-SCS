use crate::config::Config;

pub struct CompilerContext {
    pub pretty_output: bool,
}

impl CompilerContext {
    pub fn new(config: &Config) -> Self {
        Self {
            pretty_output: config.pretty_output,
        }
    }
}
