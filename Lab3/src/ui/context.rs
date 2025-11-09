use crate::config::Config;
use crate::ui::modals::error::ErrorModal;
use crossbeam::channel::{Receiver, Sender, unbounded};

pub struct UIContext {
    pub errors_tx: Sender<ErrorModal>,
    pub errors_rx: Receiver<ErrorModal>,
}

impl UIContext {
    pub fn new(_: &Config) -> Self {
        let (errors_tx, errors_rx) = unbounded::<ErrorModal>();

        Self {
            errors_tx,
            errors_rx,
        }
    }
}
