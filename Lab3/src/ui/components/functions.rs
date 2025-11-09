use crate::context::Context;

#[derive(Default)]
pub struct FunctionsComponent;

impl FunctionsComponent {
    pub fn show(&self, _context: &mut Context, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Run:");
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
