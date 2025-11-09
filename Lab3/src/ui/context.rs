use crate::config::Config;
use crate::ui::components::main::MainComponent;
use crate::ui::components::side_panel::SidePanel;
use crate::ui::modals::Modal;
use crossbeam::channel::{Receiver, Sender, unbounded};

pub struct UIContext {
    pub main_component: MainComponent,
    pub side_panel: SidePanel,

    pub modals_tx: Sender<Box<dyn Modal>>,
    pub modals_rx: Receiver<Box<dyn Modal>>,
}

impl UIContext {
    pub fn new(_: &Config) -> Self {
        let (modals_tx, modals_rx) = unbounded::<Box<dyn Modal>>();

        Self {
            main_component: Default::default(),
            side_panel: Default::default(),

            modals_tx,
            modals_rx,
        }
    }
}
