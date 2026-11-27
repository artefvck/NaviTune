use crate::card_layout::draw_card;
use crate::PowerPlayApp;
use eframe::egui;

#[derive(Clone, Debug)]
struct ClassicWattmanState {
    initialized: bool,
    f_pts: [i32; 3],
    v_pts: [i32; 3],
    base_v_pts: [f64; 3],
    write_to_offset_table: bool,
    couple_fan_to_vf: bool,
}

impl Default for ClassicWattmanState {
    fn default() -> Self {
        Self {
            initialized: false,
            f_pts: [500, 1400, 2400],
            v_pts: [775, 920, 1050],
            base_v_pts: [775.0, 950.0, 1150.0],
            write_to_offset_table: true,
            couple_fan_to_vf: true,
        }
    }
}

fn solve_quadratic_3pts(f_mhz: [i32; 3], v_mv: [f64; 3]) -> (f64, f64, f64) {
    let x0 = (f_mhz[0] as f64) / 1000.0;
    let x1 = (f_mhz[1] as f64) / 1000.0;
    let x2 = (f_mhz[2] as f64) / 1000.0;
    let y0 = v_mv[0] / 1000.0;
    let y1 = v_mv[1] / 1000.0;
    let y2 = v_mv[2] / 1000.0;
    let d0 = (x0 - x1) * (x0 - x2);
    let d1 = (x1 - x0) * (x1 - x2);
    let d2 = (x2 - x0) * (x2 - x1);
    if d0.abs() < 1e-9 || d1.abs() < 1e-9 || d2.abs() < 1e-9 {
        return (0.0, 0.0, y2);
    }
    let l0 = y0 / d0;
    let l1 = y1 / d1;
    let l2 = y2 / d2;
    let a = l0 + l1 + l2;
    let b = -(l0 * (x1 + x2) + l1 * (x0 + x2) + l2 * (x0 + x1));
    let c = l0 * x1 * x2 + l1 * x0 * x2 + l2 * x0 * x1;
    (a, b, c)
}

impl PowerPlayApp {
    pub fn draw_classic_tab(&mut self, ui: &mut egui::Ui) {
        if self.raw_buffer.is_empty() {
            draw_card(ui, "Classic Adrenalin WattMan — 3-Point V/F & Fan Curve", |ui| {
                ui.label("No table loaded. Click Load or Select GPU Path... first to tune Min / Mid / Max V/F and Fan curves.");
            });
            return;
        }
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let (_, _, safe_v_cap, _, _) = arch.safe_tune_bounds();
        let state_id = egui::Id::new("classic_wattman_state");
        let gen_id = egui::Id::new("classic_wattman_gen");
        let last_gen = ui.ctx().data(|d| d.get_temp::<u64>(gen_id).unwrap_or(u64::MAX));
        let mut st = ui.ctx().data(|d| d.get_temp::<ClassicWattmanState>(state_id).unwrap_or_default());

        if !st.initialized || last_gen != self.baseline_gen {
            let f0 = self.freq_gfx[0].max(300);
            let f2 = self.freq_gfx[1].max(f0 + 200);
            let f1 = (f0 + f2) / 2;
            let v0 = if self.volt_gfx[0] > 0 { self.volt_gfx[0] } else { 750 };
            let v2 = if self.volt_gfx[1] > 0 { self.volt_gfx[1] } else { safe_v_cap };
            let off_c = (self.static_off_c[0] * 1000.0).round() as i32;
            let v2_eff = (v2 + off_c).clamp(v0, 1300);
            let v1_eff = v0 + ((v2_eff - v0) as f32 * 0.44).round() as i32;
            st.f_pts = [f0, f1, f2];
            st.v_pts = [v0, v1_eff, v2_eff];
            st.base_v_pts = [v0 as f64, (v0 as f64) + ((v2 as f64) - (v0 as f64)) * 0.48, v2 as f64];
            st.initialized = true;
            ui.ctx().data_mut(|d| d.insert_temp(gen_id, self.baseline_gen));
        }
        let target_v_f64 = [st.v_pts[0] as f64, st.v_pts[1] as f64, st.v_pts[2] as f64];
        let delta_v_f64 = [
            target_v_f64[0] - st.base_v_pts[0],
            target_v_f64[1] - st.base_v_pts[1],
            target_v_f64[2] - st.base_v_pts[2],
        ];
        let (sol_a, sol_b, sol_c) = if st.write_to_offset_table {
            solve_quadratic_3pts(st.f_pts, delta_v_f64)
        } else {
            solve_quadratic_3pts(st.f_pts, target_v_f64)
        };

        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("Classic Adrenalin WattMan — 3-Point V/F & Fan Curve ({})", arch.label()));
            ui.separator();
            if ui.button("Sync From Table").clicked() { st.initialized = false; }
            if ui.button("Apply Classic Curve (a, b, c + Fan)").clicked() {
                self.freq_gfx = [st.f_pts[0], st.f_pts[2]];
                if self.volt_gfx[1] > 0 {
                    self.volt_gfx = [st.v_pts[0], st.v_pts[2]];
                }
                self.gpu_clock_curve = true;
                if arch.has_static_vid_coeffs() {
                    if st.write_to_offset_table {
                        self.static_off_a[0] = sol_a;
                        self.static_off_b[0] = sol_b;
                        self.static_off_c[0] = sol_c;
                    } else {
                        self.static_volt_a[0] = sol_a;
                        self.static_volt_b[0] = sol_b;
                        self.static_volt_c[0] = sol_c;
                    }
                }
                if st.couple_fan_to_vf {
                    self.fan_curve = true;
                    self.fan_target_gfx_clock = st.f_pts[2];
                }
                self.sync_fields_to_buffer();
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                self.status_msg = format!(
                    "Applied Classic 3-Point V/F ({}..{}..{} MHz @ {}..{}..{} mV) | a={:.4}, b={:.4}, c={:.5}",
                    st.f_pts[0], st.f_pts[1], st.f_pts[2],
                    st.v_pts[0], st.v_pts[1], st.v_pts[2],
                    sol_a, sol_b, sol_c
                );
            }
        });
        ui.add_space(6.0);

        let vmax_snapshot = st.v_pts[2];
        crate::card_layout::two_col_cards_adaptive(ui, "classic_wattman_cols", true, true, |ui| {
            draw_card(ui, "1. Adrenalin 3-Point V/F Curve (Min / Mid / Max)", |ui| {
                egui::Grid::new("classic_vf_3pt_grid").num_columns(3).striped(true).spacing([12.0, 6.0]).show(ui, |ui| {
                    ui.strong("DPM State Point"); ui.strong("Frequency (MHz)"); ui.strong("Voltage (mV)"); ui.end_row();
                    for (idx, lbl) in [(0usize, "Point 1 (Min / Idle)"), (1, "Point 2 (Mid / Game)"), (2, "Point 3 (Max / Boost)")] {
                        ui.label(lbl);
                        ui.add(egui::DragValue::new(&mut st.f_pts[idx]).range(300..=3500).speed(5).suffix(" MHz"));
                        ui.add(egui::DragValue::new(&mut st.v_pts[idx]).range(650..=1350).speed(5).suffix(" mV"));
                        ui.end_row();
                    }
                });
                st.f_pts[1] = st.f_pts[1].clamp(st.f_pts[0] + 50, (st.f_pts[2] - 50).max(st.f_pts[0] + 50));
                st.v_pts[1] = st.v_pts[1].clamp(st.v_pts[0], st.v_pts[2].max(st.v_pts[0]));
                ui.add_space(6.0);
                if arch.has_static_vid_coeffs() {
                    ui.horizontal_wrapped(|ui| {
                        ui.radio_value(&mut st.write_to_offset_table, true, "Write to Voltage Offset (0x7AA — Safe AVFS)");
                        ui.radio_value(&mut st.write_to_offset_table, false, "Write to Base VID Curve (0x74E)");
                    });
                    ui.add_space(4.0);
                    ui.monospace(format!("Solved Quadratic: a = {:.6} | b = {:.6} | c = {:.6}", sol_a, sol_b, sol_c));
                } else {
                    ui.colored_label(egui::Color32::from_rgb(160, 175, 195), "This architecture uses hardware AVFS/DPM tables; Apply updates Min/Max clock & voltage envelope.");
                }
            });
        }, |ui| {
            draw_card(ui, "2. Coupled 5-Point Fan Speed Curve (WattMan Style)", |ui| {
                ui.horizontal_wrapped(|ui| {
                    crate::ui_theme::green_checkbox(ui, &mut st.couple_fan_to_vf, "Link Fan Target to V/F Max");
                    crate::ui_theme::green_checkbox(ui, &mut self.fan_zero_rpm_enable, "Zero RPM");
                    if ui.button("Auto-Scale Fan to V/F Load").clicked() {
                        let v_ratio = ((vmax_snapshot as f32 - 900.0) / 250.0).clamp(0.0, 1.2);
                        let peak_pwm = (52.0 + v_ratio * 28.0).round() as i32;
                        let base_t = if self.fan_start_temp > 0 { self.fan_start_temp } else { 50 };
                        let span_t = (self.fan_target_temp.max(base_t + 20) - base_t).max(20);
                        for p_i in 0..5 {
                            self.fan_curve_temp[p_i] = base_t + (span_t * (p_i as i32)) / 4;
                            self.fan_curve_pwm[p_i] = (self.fan_pwm_min.max(20) + ((peak_pwm - self.fan_pwm_min.max(20)) * (p_i as i32)) / 4).clamp(0, 100);
                        }
                        self.fan_curve = true;
                        ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                    }
                });
                ui.add_space(4.0);
                if arch.has_5point_fan_curve_slots() {
                    egui::Grid::new("classic_fan_5pt_grid").num_columns(3).striped(true).spacing([12.0, 4.0]).show(ui, |ui| {
                        ui.strong("Fan Point"); ui.strong("Temp (°C)"); ui.strong("PWM (%)"); ui.end_row();
                        for i in 0..5 {
                            ui.label(format!("P{}", i + 1));
                            if ui.add(egui::DragValue::new(&mut self.fan_curve_temp[i]).range(25..=110).suffix(" °C")).changed() {
                                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                            }
                            if ui.add(egui::DragValue::new(&mut self.fan_curve_pwm[i]).range(0..=100).suffix(" %")).changed() {
                                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                            }
                            ui.end_row();
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Min PWM:"); ui.add(egui::DragValue::new(&mut self.fan_pwm_min).range(0..=100).suffix(" %"));
                        ui.label("Target Temp:"); ui.add(egui::DragValue::new(&mut self.fan_target_temp).range(40..=105).suffix(" °C"));
                    });
                }
            });
        });

        draw_card(ui, "3. Classic 3-Point Quadratic V/F Curve Preview (Min -> Mid -> Max)", |ui| {
            let desired = egui::vec2(ui.available_width().max(260.0), 165.0);
            let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(18, 23, 33));
            painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(52, 66, 92)));
            let plot = egui::Rect::from_min_max(egui::pos2(rect.left() + 44.0, rect.top() + 14.0), egui::pos2(rect.right() - 18.0, rect.bottom() - 22.0));
            if plot.width() > 20.0 && plot.height() > 20.0 {
                let f_min = st.f_pts[0] as f32;
                let f_max = (st.f_pts[2] as f32).max(f_min + 100.0);
                let v_lo = 650.0_f32;
                let v_hi = (st.v_pts[2] as f32 + 60.0).max(1150.0);
                let to_scr = |f: f32, v: f32| -> egui::Pos2 {
                    let nx = ((f - f_min) / (f_max - f_min)).clamp(0.0, 1.0);
                    let ny = ((v - v_lo) / (v_hi - v_lo)).clamp(0.0, 1.0);
                    egui::pos2(plot.left() + nx * plot.width(), plot.bottom() - ny * plot.height())
                };
                let (qa, qb, qc) = solve_quadratic_3pts(st.f_pts, target_v_f64);
                let mut pts = Vec::with_capacity(25);
                for s in 0..=24 {
                    let f = f_min + (s as f32 / 24.0) * (f_max - f_min);
                    let fg = (f as f64) / 1000.0;
                    let v = ((qa * fg * fg + qb * fg + qc) * 1000.0) as f32;
                    pts.push(to_scr(f, v));
                }
                for w in pts.windows(2) {
                    painter.line_segment([w[0], w[1]], egui::Stroke::new(2.2_f32, egui::Color32::from_rgb(240, 88, 108)));
                }
                for (idx, tag) in [(0usize, "Min"), (1, "Mid"), (2, "Max")] {
                    let p = to_scr(st.f_pts[idx] as f32, st.v_pts[idx] as f32);
                    painter.circle_filled(p, 5.5, egui::Color32::from_rgb(255, 200, 75));
                    painter.text(egui::pos2(p.x, p.y - 8.0), egui::Align2::CENTER_BOTTOM, format!("{} ({}MHz/{}mV)", tag, st.f_pts[idx], st.v_pts[idx]), egui::FontId::proportional(10.0), egui::Color32::from_rgb(230, 238, 250));
                }
            }
        });

        ui.ctx().data_mut(|d| d.insert_temp(state_id, st));
    }
}
