use crate::context::Context;
use crate::ui::styles::colors;

#[derive(Debug, Default)]
pub struct MainComponent {
    code: String,
    result: String,
}

impl MainComponent {
    pub fn show(&mut self, context: &mut Context, ui: &mut egui::Ui) {
        if let Some(result) = context.ui.get_output() {
            self.result = result;
        }

        ui.horizontal(|ui| {
            ui.label("Код:");

            if ui
                .add(
                    egui::TextEdit::singleline(&mut self.code)
                        .desired_width(600.0)
                        .text_color(colors::BLACK),
                )
                .changed()
            {
                context.compiler.code = self.code.clone();
            };

            if ui.button("Еквівалентні форми").clicked() {
                context
                    .ui
                    .set_output(context.compiler.equivalent_forms_report());
            }
        });

        ui.centered_and_justified(|ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut self.result).code_editor());
            });
        });
    }
}
