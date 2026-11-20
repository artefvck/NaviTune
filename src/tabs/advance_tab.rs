use eframe::egui;
use crate::{AdvanceOsTab, PowerPlayApp};

impl PowerPlayApp {
    pub fn draw_advance_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.advance_os_tab, AdvanceOsTab::MacOS, "macOS");
            ui.selectable_value(&mut self.advance_os_tab, AdvanceOsTab::Windows, "Windows");
            ui.selectable_value(&mut self.advance_os_tab, AdvanceOsTab::Linux, "Linux");
        });
        ui.separator();
        ui.add_space(4.0);

        if self.raw_buffer.is_empty() {
            crate::card_layout::draw_card(ui, "OS Optimization Presets", |ui| {
                ui.label("Load a PowerPlay table (.bin, .rom, .reg, .mpt, or ioreg) first.");
            });
            return;
        }

        match self.advance_os_tab {
            AdvanceOsTab::MacOS => self.draw_advance_macos(ui),
            AdvanceOsTab::Windows => self.draw_advance_windows(ui),
            AdvanceOsTab::Linux => self.draw_advance_linux(ui),
        }

        if self.show_opencore_copy_modal {
            let vp_id = egui::ViewportId::from_hash_of("opencore_copy_modal_viewport");
            let builder = egui::ViewportBuilder::default()
                .with_title("Copy PP_PhmSoftPowerPlayTable")
                .with_inner_size([400.0, 175.0])
                .with_resizable(false);
            ui.ctx().show_viewport_immediate(vp_id, builder, |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) { self.show_opencore_copy_modal = false; }
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.sync_fields_to_buffer();
                    let b64 = crate::export_formats::encode_base64(&self.raw_buffer);
                    let hex_raw: String = self.raw_buffer.iter().map(|b| format!("{:02X}", b)).collect();
                    ui.label(format!("Synced PP_PhmSoftPowerPlayTable ({} bytes):", self.raw_buffer.len()));
                    ui.monospace(format!("Base64: {}...", &b64[..b64.len().min(36)]));
                    ui.monospace(format!("Hex   : {}...", &hex_raw[..hex_raw.len().min(36)]));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Copy Base64").clicked() { ui.ctx().copy_text(b64); self.show_opencore_copy_modal = false; }
                        if ui.button("Copy Hex (No Space)").clicked() { ui.ctx().copy_text(hex_raw); self.show_opencore_copy_modal = false; }
                        if ui.button("Close").clicked() { self.show_opencore_copy_modal = false; }
                    });
                });
            });
        }
    }
}
