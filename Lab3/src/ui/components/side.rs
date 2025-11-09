use crate::context::Context;

#[derive(Debug, Default)]
pub struct SideComponent;

impl SideComponent {
    pub fn show(&self, _context: &mut Context, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Tabs");
            });
        });
    }
}
