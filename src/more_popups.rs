use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::PowerPlayApp;
use eframe::egui;
use std::mem::offset_of;

#[derive(Default)]
pub struct MorePopupsState {
    pub power_settings: bool,
    pub curve_settings: bool,
    pub throttler: bool,
    pub dpm_overrides: bool,
}

impl PowerPlayApp {
    pub fn render_more_popup_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("Power Settings").clicked() {
                self.more_popups.power_settings = !self.more_popups.power_settings;
            }
            if ui.button("Curve Settings").clicked() {
                self.more_popups.curve_settings = !self.more_popups.curve_settings;
            }
            if ui.button("Throttler").clicked() {
                self.more_popups.throttler = !self.more_popups.throttler;
            }
            if ui.button("DPM Overrides").clicked() {
                self.more_popups.dpm_overrides = !self.more_popups.dpm_overrides;
            }
        });
    }

    pub fn render_more_viewports(&mut self, ctx: &egui::Context) {
        if self.more_popups.power_settings {
            let vp = egui::ViewportId::from_hash_of("vp_more_power_settings");
            let builder = egui::ViewportBuilder::default()
                .with_title("Power Settings (PPT0..3 & Boost)")
                .with_inner_size([340.0, 260.0])
                .with_resizable(true);
            ctx.show_viewport_immediate(vp, builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.more_popups.power_settings = false;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    let Some(t) = AtomSmcDpmInfoV4_9::from_bytes_mut(&mut self.raw_buffer) else {
                        ui.label("Load a PPTable first.");
                        return;
                    };
                    let base = offset_of!(AtomSmcDpmInfoV4_9, socket_power_limit_ac);
                    egui::Grid::new("vp_pwr_grid").spacing([10.0, 6.0]).show(ui, |ui| {
                        ui.strong("Limit"); ui.strong("Value (W)"); ui.strong("Offset"); ui.end_row();
                        for (idx, lbl) in [(0, "PPT0 (Sustained)"), (1, "PPT1 (Short)"), (2, "PPT2 (Peak)"), (3, "PPT3 (Fast)")] {
                            let mut val = { t.socket_power_limit_ac[idx] } as i32;
                            ui.label(lbl);
                            if ui.add(egui::DragValue::new(&mut val).range(0..=1000)).changed() {
                                t.socket_power_limit_ac[idx] = val as u16;
                                if idx == 0 {
                                    self.power_limit_w = val;
                                }
                            }
                            ui.monospace(format!("0x{:03X}", base + idx * 2));
                            ui.end_row();
                        }
                    });
                });
            });
        }

        if self.more_popups.curve_settings {
            let vp = egui::ViewportId::from_hash_of("vp_more_curve_settings");
            let builder = egui::ViewportBuilder::default()
                .with_title("Curve Settings (V/F & AVFS)")
                .with_inner_size([360.0, 270.0])
                .with_resizable(true);
            ctx.show_viewport_immediate(vp, builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.more_popups.curve_settings = false;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::Grid::new("vp_crv_grid").spacing([10.0, 6.0]).show(ui, |ui| {
                        ui.strong("Parameter"); ui.strong("GFX"); ui.strong("Soc"); ui.end_row();
                        ui.label("Quad Coeff a");
                        ui.add(egui::DragValue::new(&mut self.static_volt_a[0]).speed(0.0001).fixed_decimals(6));
                        ui.add(egui::DragValue::new(&mut self.static_volt_a[1]).speed(0.0001).fixed_decimals(6));
                        ui.end_row();
                        ui.label("Quad Coeff b");
                        ui.add(egui::DragValue::new(&mut self.static_volt_b[0]).speed(0.0001).fixed_decimals(6));
                        ui.add(egui::DragValue::new(&mut self.static_volt_b[1]).speed(0.0001).fixed_decimals(6));
                        ui.end_row();
                        ui.label("Quad Coeff c");
                        ui.add(egui::DragValue::new(&mut self.static_volt_c[0]).speed(0.001).fixed_decimals(6));
                        ui.add(egui::DragValue::new(&mut self.static_volt_c[1]).speed(0.001).fixed_decimals(6));
                        ui.end_row();
                        ui.label("DcTol (mV)");
                        ui.add(egui::DragValue::new(&mut self.dctol[0]));
                        ui.add(egui::DragValue::new(&mut self.dctol[1]));
                        ui.end_row();
                    });
                });
            });
        }

        if self.more_popups.throttler {
            let vp = egui::ViewportId::from_hash_of("vp_more_throttler");
            let builder = egui::ViewportBuilder::default()
                .with_title("Thermal & Current Throttler Limits")
                .with_inner_size([370.0, 300.0])
                .with_resizable(true);
            ctx.show_viewport_immediate(vp, builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.more_popups.throttler = false;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    let Some(t) = AtomSmcDpmInfoV4_9::from_bytes_mut(&mut self.raw_buffer) else {
                        ui.label("Load a PPTable first.");
                        return;
                    };
                    let base = offset_of!(AtomSmcDpmInfoV4_9, temp_limit);
                    let labels = [
                        "Edge Temp Limit (°C)",
                        "Hotspot Temp Limit (°C)",
                        "Mem Temp Limit (°C)",
                        "VR GFX Temp Limit (°C)",
                        "VR Mem0 Temp Limit (°C)",
                        "VR Mem1 Temp Limit (°C)",
                        "VR Soc Temp Limit (°C)",
                        "Liquid0 Temp Limit (°C)",
                        "Liquid1 Temp Limit (°C)",
                    ];
                    egui::Grid::new("vp_thr_grid").striped(true).spacing([12.0, 6.0]).show(ui, |ui| {
                        ui.strong("Sensor / Limit"); ui.strong("Value"); ui.strong("Offset"); ui.end_row();
                        for (idx, lbl) in labels.iter().enumerate() {
                            let mut val = { t.temp_limit[idx] } as i32;
                            ui.label(*lbl);
                            if ui.add(egui::DragValue::new(&mut val).range(0..=150)).changed() {
                                t.temp_limit[idx] = val as u16;
                                if idx < self.temp_limits.len() {
                                    self.temp_limits[idx] = val;
                                }
                            }
                            ui.monospace(format!("0x{:03X}", base + idx * 2));
                            ui.end_row();
                        }
                    });
                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        self.more_popups.throttler = false;
                    }
                });
            });
        }

        if self.more_popups.dpm_overrides {
            let vp = egui::ViewportId::from_hash_of("vp_more_dpm_overrides");
            let builder = egui::ViewportBuilder::default()
                .with_title("Memory & Core DPM Overrides")
                .with_inner_size([360.0, 280.0])
                .with_resizable(true);
            ctx.show_viewport_immediate(vp, builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.more_popups.dpm_overrides = false;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::Grid::new("vp_dpm_grid").spacing([10.0, 6.0]).show(ui, |ui| {
                        ui.strong("State"); ui.strong("UCLK (MHz)"); ui.strong("VDDCI (mV)"); ui.strong("MVDD (mV)"); ui.end_row();
                        for i in 0..4 {
                            ui.label(format!("DPM {}", i));
                            ui.add(egui::DragValue::new(&mut self.mem_dpm_freq[i][0]));
                            ui.add(egui::DragValue::new(&mut self.mem_dpm_volt[i][0]));
                            ui.add(egui::DragValue::new(&mut self.mem_dpm_volt[i][1]));
                            ui.end_row();
                        }
                    });
                });
            });
        }
    }
}
