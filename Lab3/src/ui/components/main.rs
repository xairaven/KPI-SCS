use crate::context::Context;

#[derive(Debug, Default)]
pub struct MainComponent {
    code: String,
    result: String,

    is_file_opened: bool,
}

impl MainComponent {
    pub fn show(&mut self, _context: &mut Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Code:");

            ui.add(egui::TextEdit::singleline(&mut self.code).desired_width(500.0));

            // Open File
            if ui.button("📁").clicked() {
                self.is_file_opened = true;
            }

            // Reload
            if self.is_file_opened {
                if ui.button("↺").clicked() {}
                if ui.button("⊗").clicked() {
                    self.is_file_opened = false;
                }
            }
        });

        ui.separator();

        ui.centered_and_justified(|ui| {
            ui.add(egui::TextEdit::multiline(&mut self.result).interactive(false));
        });
    }
}
