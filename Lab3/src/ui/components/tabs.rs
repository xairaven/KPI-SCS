use crate::context::Context;
use crate::ui::styles;
use egui::RichText;

#[derive(Default)]
pub struct TabsComponent;

impl TabsComponent {
    pub fn show(&self, _context: &mut Context, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Tabs");
        });

        ui.add_space(10.0);

        ui.vertical_centered_justified(|ui| {
            if ui
                .button(RichText::new("RUN").color(styles::colors::GREEN))
                .clicked()
            {}
        });

        ui.add_space(10.0);

        ui.vertical_centered_justified(|ui| {
            if ui.button("Tokenizer").clicked() {}

            if ui.button("Syntax check").clicked() {}

            if ui.button("AST").clicked() {}

            if ui.button("Balance AST").clicked() {}
        });
    }
}
