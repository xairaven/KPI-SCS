use crate::context::Context;
use crate::errors::Error;
use crate::io::IoError;
use crate::ui::modals::error::ErrorModal;
use crossbeam::channel::Sender;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct MainComponent {
    code: String,
    result: String,

    opened_file: Option<PathBuf>,
}

impl MainComponent {
    pub fn show(&mut self, context: &mut Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Code:");

            ui.add(egui::TextEdit::singleline(&mut self.code).desired_width(500.0));

            // Clear code field
            if ui.button("⟲").clicked() {
                self.code = String::new();
            }

            // Open File
            if ui.button("📁").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("text", &["txt", "xai"])
                    .pick_file()
            {
                self.read_file(path, &context.ui.errors_tx);
            }

            if let Some(path) = &self.opened_file {
                // Reload file
                if ui.button("↺").clicked() {
                    self.read_file(path.clone(), &context.ui.errors_tx);
                }
                // Close file
                if ui.button("⊗").clicked() {
                    self.opened_file = None;
                }
            }
        });

        ui.separator();

        ui.centered_and_justified(|ui| {
            ui.add(egui::TextEdit::multiline(&mut self.result).interactive(false));
        });
    }

    fn read_file(&mut self, path: PathBuf, tx: &Sender<ErrorModal>) {
        match fs::read_to_string(&path) {
            Ok(text) => {
                self.code = text;
                self.opened_file = Some(path.clone());
            },
            Err(error) => {
                let error: Error = IoError::ReadFile(error).into();
                ErrorModal::new(error).try_send_by(tx);
            },
        }
    }
}
