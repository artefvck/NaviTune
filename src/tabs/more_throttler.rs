use crate::card_layout::draw_card;
use crate::PowerPlayApp;
use eframe::egui;

const TEMP_LABELS: [&str; 9] = [
    "Edge Temp Limit (°C)", "Hotspot Temp Limit (°C)", "Mem Temp Limit (°C)",
    "VR GFX Temp Limit (°C)", "VR Mem0 Temp Limit (°C)", "VR Mem1 Temp Limit (°C)",
    "VR Soc Temp Limit (°C)", "Liquid0 Temp Limit (°C)", "Liquid1 Temp Limit (°C)",
];

impl PowerPlayApp {
    pub(crate) fn render_more_throttler_card(&mut self, ui: &mut egui::Ui) {
        draw_card(ui, "Thermal Throttler", |ui| {
            if self.raw_buffer.is_empty() { ui.label("Load a PPTable first."); return; }
            let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
            let compact_r1 = arch == crate::ppt_arch::PptArch::Rdna1 && self.raw_buffer.len() < 1924;
            if compact_r1 {
                ui.colored_label(
                    egui::Color32::from_rgb(160, 175, 195),
                    "Compact 1674 B RDNA 1 VBIOS tables pack thermal thresholds in SMU firmware (7-channel TempLimit requires 1924 B V4_5).",
                );
                ui.add_space(4.0);
            }
            let mut changed = false;
            let max_temps = arch.max_temp_entries().min(TEMP_LABELS.len());
            ui.add_enabled_ui(!compact_r1, |ui| {
                for (idx, lbl) in TEMP_LABELS.iter().take(max_temps).enumerate() {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        changed |= ui.add_sized([86.0, 20.0], egui::DragValue::new(&mut self.temp_limits[idx]).range(0..=150).suffix(" °C")).changed();
                        ui.label(*lbl);
                    });
                }
            });
            if changed { self.sync_fields_to_buffer(); }
        });
    }
}
