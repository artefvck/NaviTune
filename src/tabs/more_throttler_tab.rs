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
            let mut changed = false;
            for (idx, lbl) in TEMP_LABELS.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    changed |= ui.add_sized([86.0, 20.0], egui::DragValue::new(&mut self.temp_limits[idx]).range(0..=150).suffix(" °C")).changed();
                    ui.label(*lbl);
                });
            }
            if changed { self.sync_fields_to_buffer(); }
        });
    }
}
