use crate::PowerPlayApp;
use eframe::egui;

impl PowerPlayApp {
    pub(crate) fn draw_debug_capture_log(&mut self, ui: &mut egui::Ui) {
        let dump_text = self.build_full_debug_dump();
        ui.horizontal(|ui| {
            if ui.button("Copy Full Dump").clicked() {
                ui.ctx().copy_text(dump_text.clone());
                self.status_msg = "Copied full system & table debug dump to clipboard.".to_string();
            }
            if ui.button("Save Log (.log)").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("navitune_debug.log")
                    .add_filter("Log / Text", &["log", "txt"])
                    .save_file()
                {
                    match std::fs::write(&path, &dump_text) {
                        Ok(_) => self.status_msg = format!("Saved debug dump to {}", path.display()),
                        Err(e) => self.status_msg = format!("Failed to save debug dump: {}", e),
                    }
                }
            }
        });
        ui.add_space(6.0);
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            let mut buf = dump_text;
            ui.add(
                egui::TextEdit::multiline(&mut buf)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY),
            );
        });
    }

}
