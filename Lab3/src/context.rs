use crate::compiler::context::CompilerContext;
use crate::config::Config;
use crate::ui::context::UIContext;

pub struct Context {
    pub compiler: CompilerContext,
    pub ui: UIContext,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            compiler: CompilerContext::new(&config),
            ui: UIContext::new(&config),
        }
    }
}
