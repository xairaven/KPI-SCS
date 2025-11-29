use crate::config::Config;
use crate::context::Context;
use crate::ui::components::main::MainComponent;
use crate::ui::modals::Modal;
use crate::ui::modals::error::ErrorModal;
use crate::ui::styles;
use egui::{CentralPanel, Visuals};

pub struct App {
    context: Context,

    main_component: MainComponent,

    errors: Vec<ErrorModal>,
}

impl App {
    pub fn new(_: &eframe::CreationContext<'_>, config: Config) -> Self {
        let context = Context::new(config);

        Self {
            context,

            main_component: Default::default(),

            errors: vec![],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = Visuals::light();
        visuals.panel_fill = styles::colors::BEIGE;
        visuals.override_text_color = Some(styles::colors::BLACK);
        visuals.widgets.inactive.bg_stroke =
            egui::Stroke::new(1.0, styles::colors::BLACK);
        ctx.set_visuals(visuals);

        CentralPanel::default().show(ctx, |ui| {
            CentralPanel::default().show_inside(ui, |ui| {
                self.main_component.show(&mut self.context, ui);
            });

            // Getting modals from the channels (in context).
            if let Ok(modal) = self.context.ui.errors_rx.try_recv() {
                self.errors.push(modal);
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

        for (index, modal) in self.errors.iter_mut().enumerate() {
            modal.show(ui, &mut self.context);

            if modal.is_closed() {
                closed_modals.push(index);
            }
        }

        closed_modals.iter().for_each(|index| {
            self.errors.remove(*index);
        });
    }
}
