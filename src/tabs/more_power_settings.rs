use crate::card_layout::draw_card;
use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::PowerPlayApp;
use eframe::egui;
use std::mem::offset_of;

impl PowerPlayApp {
    pub(crate) fn render_more_power_settings_card(&mut self, ui: &mut egui::Ui) {
        draw_card(ui, "Secondary Power (PPT)", |ui| {
            if let Some(t3) = AtomSmcDpmInfoV4_12::from_bytes_mut(&mut self.raw_buffer) {
                let off1 = offset_of!(AtomSmcDpmInfoV4_12, socket_power_limit_ac_2);
                egui::Grid::new("card_pwr_grid").spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.strong("Limit (RDNA 3)"); ui.strong("Value (W)"); ui.strong("Offset"); ui.end_row();
                    let mut v1 = { t3.socket_power_limit_ac_2 } as i32;
                    ui.label("PPT1 (Short)");
                    if ui.add(egui::DragValue::new(&mut v1).range(0..=1000)).changed() { t3.socket_power_limit_ac_2 = v1 as u16; }
                    ui.monospace(format!("0x{:03X}", off1)); ui.end_row();
                });
            } else if let Some(r1) = crate::pptable_rdna1::Rdna1ParsedState::parse(&self.raw_buffer) {
                let base = r1.outer_sz + 0x00C;
                egui::Grid::new("card_pwr_grid").spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.strong("Limit (RDNA 1 · 1674 B)"); ui.strong("Value (W)"); ui.strong("Offset"); ui.end_row();
                    for (idx, lbl) in [(1usize, "PPT1 (Short)"), (2, "PPT2 (Peak)"), (3, "PPT3 (Fast)")] {
                        let off = base + idx * 2;
                        ui.label(lbl);
                        let mut val = u16::from_le_bytes([self.raw_buffer[off], self.raw_buffer[off + 1]]) as i32;
                        if ui.add(egui::DragValue::new(&mut val).range(0..=1000)).changed() {
                            self.raw_buffer[off..off + 2].copy_from_slice(&(val as u16).to_le_bytes());
                        }
                        ui.monospace(format!("0x{:03X}", off));
                        ui.end_row();
                    }
                });
            } else if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes_mut(&mut self.raw_buffer) {
                let base = offset_of!(crate::pptable_rdna1::AtomSmcDpmInfoV4_5, socket_power_limit_ac);
                egui::Grid::new("card_pwr_grid").spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.strong("Limit (RDNA 1)"); ui.strong("Value (W)"); ui.strong("Offset"); ui.end_row();
                    for (idx, lbl) in [(1, "PPT1 (Short)"), (2, "PPT2 (Peak)"), (3, "PPT3 (Fast)")] {
                        ui.label(lbl);
                        let mut val = { t1.socket_power_limit_ac[idx] } as i32;
                        if ui.add(egui::DragValue::new(&mut val).range(0..=1000)).changed() {
                            t1.socket_power_limit_ac[idx] = val as u16;
                        }
                        ui.monospace(format!("0x{:03X}", base + idx * 2));
                        ui.end_row();
                    }
                });
            } else if let Some(v20) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes_mut(&mut self.raw_buffer) {
                let base = offset_of!(crate::pptable_vega::AtomVega20PowerPlayTable, socket_power_limit_ac);
                egui::Grid::new("card_pwr_grid").spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.strong("Limit (Vega 20)"); ui.strong("Value (W)"); ui.strong("Offset"); ui.end_row();
                    for (idx, lbl) in [(1, "PPT1 (Short)"), (2, "PPT2 (Peak)"), (3, "PPT3 (Fast)")] {
                        ui.label(lbl);
                        let mut val = { v20.socket_power_limit_ac[idx] } as i32;
                        if ui.add(egui::DragValue::new(&mut val).range(0..=1000)).changed() {
                            v20.socket_power_limit_ac[idx] = val as u16;
                        }
                        ui.monospace(format!("0x{:03X}", base + idx * 2));
                        ui.end_row();
                    }
                });
            } else if let Some(t) = AtomSmcDpmInfoV4_9::from_bytes_mut(&mut self.raw_buffer) {
                let base = offset_of!(AtomSmcDpmInfoV4_9, socket_power_limit_ac);
                egui::Grid::new("card_pwr_grid").spacing([10.0, 6.0]).show(ui, |ui| {
                    ui.strong("Limit (RDNA 2)"); ui.strong("Value (W)"); ui.strong("Offset"); ui.end_row();
                    for (idx, lbl) in [(1, "PPT1 (Short)"), (2, "PPT2 (Peak)"), (3, "PPT3 (Fast)")] {
                        let mut val = { t.socket_power_limit_ac[idx] } as i32;
                        ui.label(lbl);
                        if ui.add(egui::DragValue::new(&mut val).range(0..=1000)).changed() { t.socket_power_limit_ac[idx] = val as u16; }
                        ui.monospace(format!("0x{:03X}", base + idx * 2)); ui.end_row();
                    }
                });
            } else { ui.colored_label(egui::Color32::from_rgb(160, 175, 195), "Secondary PPT1–PPT3 limits are not present in this compact/GCN PPTable revision."); }
        });
    }
}
