use eframe::egui;
use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::pptable_rdna3::AtomSmcDpmInfoV4_12;
use crate::{PowerPlayApp, dec_box, float_box};

impl PowerPlayApp {
    pub fn draw_curve_tab(&mut self, ui: &mut egui::Ui) {
        use crate::card_layout::*;

        let is_rdna3 = AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer).is_some();
        let is_rdna2 = !is_rdna3 && AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer).is_some();

        let mut issues: Vec<&'static str> = Vec::new();
        for &t in self.temp_limits.iter() {
            if !(0..=130).contains(&t) {
                issues.push("Thermal limit out of 0..130°C range");
                break;
            }
        }
        if is_rdna2 {
            if self.static_off_c[0].abs() > 0.25 || self.static_off_c[1].abs() > 0.25 {
                issues.push("Static Voltage Offset (c) exceeds ±250 mV");
            }
            if self.dcbtc_min[0] > self.dcbtc_max[0] || self.dcbtc_min[1] > self.dcbtc_max[1] {
                issues.push("DcBtc Min > Max");
            }
        }


        two_col_cards(ui, "crv_cols", |ui| {
            draw_card(ui, "Voltage Curve (VID)", |ui| {
                if is_rdna3 {
                    ui.colored_label(
                        egui::Color32::from_rgb(160, 175, 195),
                        "RDNA 3 (SMU 13.0) uses AVFS per-die fused V/F curves instead of static quadratic (a, b, c) coefficients.",
                    );
                } else {
                    let w2 = dynamic_box_w(ui, 76.0, 2);
                    kv_header_2(ui, "GFX", "Soc", 76.0);
                    for (lbl, tag, pair) in [
                        ("Coeff a", "curve_a (0x74E)", &mut self.static_volt_a),
                        ("Coeff b", "curve_b (0x752)", &mut self.static_volt_b),
                        ("Coeff c", "curve_c (0x756)", &mut self.static_volt_c),
                    ] {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            label_with_var(ui, lbl, tag);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                float_box(ui, &mut pair[1], w2);
                                float_box(ui, &mut pair[0], w2);
                            });
                        });
                    }
                }
            });
            draw_card(ui, "Voltage Offset", |ui| {
                if is_rdna3 {
                    ui.colored_label(
                        egui::Color32::from_rgb(160, 175, 195),
                        "Use Power -> GFX / Soc Vmin & Vmax (0x3B4..0x3BC) to adjust RDNA 3 voltage bounds.",
                    );
                } else {
                    let w2 = dynamic_box_w(ui, 76.0, 2);
                    kv_header_2(ui, "GFX", "Soc", 76.0);
                    for (lbl, tag, pair) in [
                        ("Coeff a", "off_a (0x766)", &mut self.static_off_a),
                        ("Coeff b", "off_b (0x76A)", &mut self.static_off_b),
                        ("Coeff c (V)", "off_c (0x76E)", &mut self.static_off_c),
                    ] {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            label_with_var(ui, lbl, tag);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                float_box(ui, &mut pair[1], w2);
                                float_box(ui, &mut pair[0], w2);
                            });
                        });
                    }
                }
            });
        }, |ui| {
            draw_card(ui, "Thermal Limits", |ui| {
                let names = [
                    ("Edge", "temp_limit (0x372)"),
                    ("Hotspot", "temp_limit (0x374)"),
                    ("Memory", "temp_limit (0x376)"),
                    ("VR GFX", "temp_limit (0x378)"),
                    ("VR Mem0", "temp_limit (0x37A)"),
                    ("VR Mem1", "temp_limit (0x37C)"),
                    ("VR Soc", "temp_limit (0x37E)"),
                    ("Liquid0", "temp_limit (0x380)"),
                    ("Liquid1", "temp_limit (0x382)"),
                ];
                for (i, (lbl, tag)) in names.iter().enumerate() {
                    kv_row_1(ui, lbl, tag, &mut self.temp_limits[i], 76.0);
                }
            });
            if is_rdna2 || !is_rdna3 {
                draw_card(ui, "DcBtc & DcTol (mV)", |ui| {
                    let w2 = dynamic_box_w(ui, 76.0, 2);
                    kv_header_2(ui, "GFX", "Soc", 76.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        label_with_var(ui, "DcBtc Enable", "dcbtc_en (0x7C6)");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.allocate_ui_with_layout(egui::vec2(w2, 20.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                crate::ui_theme::green_checkbox(ui, &mut self.dcbtc_soc_en, "");
                            });
                            ui.allocate_ui_with_layout(egui::vec2(w2, 20.0), egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                crate::ui_theme::green_checkbox(ui, &mut self.dcbtc_gfx_en, "");
                            });
                        });
                    });
                    for (lbl, tag, pair) in [
                        ("DcBtc Min", "dcbtc_min (0x7CA)", &mut self.dcbtc_min),
                        ("DcBtc Max", "dcbtc_max (0x7CE)", &mut self.dcbtc_max),
                        ("DcBtc GB", "dcbtc_gb (0x7D2)", &mut self.dcbtc_gb),
                    ] {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            label_with_var(ui, lbl, tag);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                dec_box(ui, &mut pair[1], w2);
                                dec_box(ui, &mut pair[0], w2);
                            });
                        });
                    }
                    kv_row_2(ui, "DcTol", "dctol (0x7C2)", &mut self.dctol, 76.0);
                });
            }
        });

    }
}

