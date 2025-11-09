use crate::context::Context;

#[derive(Debug, Default)]
pub struct MainComponent;

impl MainComponent {
    pub fn show(&self, _context: &mut Context, _ui: &mut egui::Ui) {}
}
