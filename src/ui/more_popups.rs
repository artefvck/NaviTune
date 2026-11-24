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
        let b64 = crate::export_formats::encode_base64(&self.raw_buffer);
        let hex_raw = format_raw_hex_nospace(&self.raw_buffer);
        let payload_len = self.raw_buffer.len();
        let vp_id = egui::ViewportId::from_hash_of("opencore_copy_external_viewport");
        let builder = egui::ViewportBuilder::default()
            .with_title("OpenCore Base64 & Raw Hex (No-Space)")
            .with_inner_size([560.0, 340.0])
            .with_min_inner_size([400.0, 260.0])
            .with_resizable(true);

        ctx.show_viewport_immediate(vp_id, builder, |vp_ctx, _class| {
            if vp_ctx.input(|i| i.viewport().close_requested()) {
                self.show_opencore_copy_modal = false;
            }
            egui::CentralPanel::default().show(vp_ctx, |ui| {
                ui.label(format!("Synced PPTable payload ({} bytes):", payload_len));
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Copy Base64").clicked() {
                        vp_ctx.copy_text(b64.clone());
                        self.status_msg = format!("Copied OpenCore Base64 ({} chars)", b64.len());
                    }
                    if ui.button("Copy Hex Raw (No Spaces)").clicked() {
                        vp_ctx.copy_text(hex_raw.clone());
                        self.status_msg = format!("Copied Raw No-Space Hex ({} chars)", hex_raw.len());
                    }
                    if ui.button("Close").clicked() {
                        self.show_opencore_copy_modal = false;
                    }
                });
                ui.add_space(8.0);
                ui.strong("Base64 (OpenCore <data>):");
                let mut b64_view = b64.clone();
                egui::ScrollArea::vertical().id_salt("oc_b64_scroll").max_height(96.0).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut b64_view).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
                });
                ui.add_space(6.0);
                ui.strong("Hex Raw (No Spaces — ProperTree / OCC Data):");
                let mut hex_view = hex_raw.clone();
                egui::ScrollArea::vertical().id_salt("oc_hex_scroll").max_height(96.0).show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut hex_view).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
                });
            });
        });
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
