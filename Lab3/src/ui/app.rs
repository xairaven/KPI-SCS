use crate::config::Config;
use crate::context::Context;
use crate::ui::modals::Modal;

pub struct App {
    context: Context,

    modals: Vec<Box<dyn Modal>>,
}

impl App {
    pub fn new(_: &eframe::CreationContext<'_>, config: Config) -> Self {
        let context = Context::new(config);

        Self {
            context,
            modals: vec![],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Getting modals from the channels (in context).
            if let Ok(modal) = self.context.ui.modals_rx.try_recv() {
                self.modals.push(modal);
            }

            // Showing modals.
            self.show_opened_modals(ui);
        });

        ctx.request_repaint();
    }
}

impl App {
    fn show_opened_modals(&mut self, ui: &egui::Ui) {
        let mut closed_modals: Vec<usize> = vec![];

        for (index, modal) in self.modals.iter_mut().enumerate() {
            modal.show(ui, &mut self.context);

            if modal.is_closed() {
                closed_modals.push(index);
            }
        }

        closed_modals.iter().for_each(|index| {
            self.modals.remove(*index);
        });
    }
}
