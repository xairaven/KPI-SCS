use crate::config::Config;
use crate::ui::modals::Modal;
use crossbeam::channel::{Receiver, Sender, unbounded};

pub struct UIContext {
    pub modals_tx: Sender<Box<dyn Modal>>,
    pub modals_rx: Receiver<Box<dyn Modal>>,
}

impl UIContext {
    pub fn new(_: &Config) -> Self {
        let (modals_tx, modals_rx) = unbounded::<Box<dyn Modal>>();

        Self {
            modals_tx,
            modals_rx,
        }
    }
}
