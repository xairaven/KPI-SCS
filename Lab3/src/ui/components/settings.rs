use crate::context::Context;

#[derive(Default)]
pub struct SettingsComponent;

impl SettingsComponent {
    pub fn show(&self, _context: &mut Context, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Settings");
        });

        ui.add_space(10.0);
    }
}
