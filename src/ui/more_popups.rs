use crate::PowerPlayApp;
use eframe::egui;

#[derive(Default)]
pub struct MorePopupsState;

pub fn format_raw_hex_nospace(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let _ = write!(s, "{:02X}", b);
    }
    s
}

impl PowerPlayApp {
    pub fn render_more_viewports(&mut self, ctx: &egui::Context) {
        if !self.show_opencore_copy_modal {
            return;
        }
        let mut open = true;
        let b64 = crate::export_formats::encode_base64(&self.raw_buffer);
        let hex_raw = format_raw_hex_nospace(&self.raw_buffer);
        egui::Window::new("OpenCore Base64 & Raw Hex (No-Space)")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(540.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!("Synced PPTable payload ({} bytes):", self.raw_buffer.len()));
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Copy Base64").clicked() {
                        ui.ctx().copy_text(b64.clone());
                        self.status_msg = format!("Copied OpenCore Base64 ({} chars)", b64.len());
                    }
                    if ui.button("Copy Hex Raw (No Spaces)").clicked() {
                        ui.ctx().copy_text(hex_raw.clone());
                        self.status_msg = format!("Copied Raw No-Space Hex ({} chars)", hex_raw.len());
                    }
                    if ui.button("Export Base64 (.txt)").clicked() {
                        if let Some(path) = rfd::FileDialog::new().set_file_name("PPTable_base64.txt").save_file() {
                            match crate::io::write_atomic(&path, b64.as_bytes()) {
                                Ok(_) => self.status_msg = format!("Exported Base64 to {}", path.display()),
                                Err(e) => self.show_error("Base64 Export Error", &e.to_string()),
                            }
                        }
                    }
                    if ui.button("Export Hex Raw (.txt)").clicked() {
                        if let Some(path) = rfd::FileDialog::new().set_file_name("PPTable_hex_raw.txt").save_file() {
                            match crate::io::write_atomic(&path, hex_raw.as_bytes()) {
                                Ok(_) => self.status_msg = format!("Exported Raw Hex to {}", path.display()),
                                Err(e) => self.show_error("Raw Hex Export Error", &e.to_string()),
                            }
                        }
                    }
                });
                ui.add_space(8.0);
                ui.strong("Base64 (OpenCore <data>):");
                let mut b64_view = b64;
                egui::ScrollArea::vertical().id_salt("oc_b64_scroll").max_height(90.0).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut b64_view).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
                });
                ui.add_space(6.0);
                ui.strong("Hex Raw (No Spaces — ProperTree / OCC Data):");
                let mut hex_view = hex_raw;
                egui::ScrollArea::vertical().id_salt("oc_hex_scroll").max_height(90.0).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut hex_view).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
                });
            });
        if !open {
            self.show_opencore_copy_modal = false;
        }
    }
}

#[cfg(test)]
mod more_popups_tests {
    use super::format_raw_hex_nospace;

    #[test]
    fn format_raw_hex_nospace_encodes_continuous_uppercase_hex() {
        assert_eq!(format_raw_hex_nospace(&[0x8A, 0x06, 0x0C, 0x01, 0xFF]), "8A060C01FF");
    }
}
