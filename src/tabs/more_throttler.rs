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
            let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
            let r1_1674 = crate::pptable_rdna1::Rdna1ParsedState::parse(&self.raw_buffer);
            let max_temps = if r1_1674.is_some() { 9 } else { arch.max_temp_entries() }.min(TEMP_LABELS.len());
            let base_off: Option<usize> = if let Some(ref r1) = r1_1674 {
                Some(r1.outer_sz + 0x034)
            } else {
                match arch {
                    crate::ppt_arch::PptArch::Rdna1 => Some(0x66B),
                    crate::ppt_arch::PptArch::Vega20 if self.raw_buffer.len() >= 1924 => Some(0x655),
                    crate::ppt_arch::PptArch::Rdna2 => Some(0x356),
                    crate::ppt_arch::PptArch::Rdna3 => Some(0x372),
                    _ => None,
                }
            };
            for (idx, lbl) in TEMP_LABELS.iter().take(max_temps).enumerate() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    changed |= ui.add_sized([86.0, 20.0], egui::DragValue::new(&mut self.temp_limits[idx]).range(0..=150).suffix(" °C")).changed();
                    if let Some(bo) = base_off {
                        ui.monospace(format!("0x{:03X}", bo + idx * 2));
                    }
                    ui.label(*lbl);
                });
            }
            if changed { self.sync_fields_to_buffer(); }
        });
    }
}
