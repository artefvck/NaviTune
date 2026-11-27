use crate::card_layout::draw_card;
use crate::PowerPlayApp;
use eframe::egui;

#[derive(Clone, Debug)]
struct SmuCalcState {
    initialized: bool,
    sim_gfx_max_mhz: i32,
    sim_gfx_vmax_mv: i32,
    sim_offset_c_mv: i32,
    sim_uclk_mhz: i32,
    sim_fclk_mhz: i32,
    sim_ppt_w: i32,
    base_gfx_max_mhz: i32,
    base_gfx_vmax_mv: i32,
    base_uclk_mhz: i32,
    base_ppt_w: i32,
}

impl Default for SmuCalcState {
    fn default() -> Self {
        Self { initialized: false, sim_gfx_max_mhz: 2000, sim_gfx_vmax_mv: 1100, sim_offset_c_mv: 0, sim_uclk_mhz: 875, sim_fclk_mhz: 1500, sim_ppt_w: 150, base_gfx_max_mhz: 2000, base_gfx_vmax_mv: 1150, base_uclk_mhz: 875, base_ppt_w: 150 }
    }
}

impl PowerPlayApp {
    pub fn draw_calc_tab(&mut self, ui: &mut egui::Ui) {
        if self.raw_buffer.is_empty() {
            draw_card(ui, "Universal SMU V/F, DCBTC & Power Envelope Calculator", |ui| {
                ui.label("No table loaded. Click Load or Select GPU Path... first to analyze any AMD Radeon PPTable / VBIOS.");
            });
            return;
        }
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let (_, safe_uclk_cap, safe_v_cap, _, _) = arch.safe_tune_bounds();
        let state_id = egui::Id::new("smu_calc_sim_state");
        let gen_id = egui::Id::new("smu_calc_last_gen");
        let last_gen = ui.ctx().data(|d| d.get_temp::<u64>(gen_id).unwrap_or(u64::MAX));
        let mut st = ui.ctx().data(|d| d.get_temp::<SmuCalcState>(state_id).unwrap_or_default());
        if !st.initialized || last_gen != self.baseline_gen {
            let mut base_app = PowerPlayApp::default();
            if !self.baseline_buffer.is_empty() { base_app.apply_ppt_bytes(self.baseline_buffer.clone(), "baseline"); }
            let bg = if base_app.freq_gfx[1] > 0 { base_app.freq_gfx[1] } else { self.freq_gfx[1].max(500) };
            let bv = if base_app.volt_gfx[1] > 0 { base_app.volt_gfx[1] } else if self.volt_gfx[1] > 0 { self.volt_gfx[1] } else { safe_v_cap };
            let bu_raw = base_app.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(0);
            let bu = if bu_raw > 0 { bu_raw } else { self.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(875).max(300) };
            let bp = if base_app.power_limit_w > 0 { base_app.power_limit_w } else if self.power_limit_w > 0 { self.power_limit_w } else { 150 };
            st.base_gfx_max_mhz = bg; st.base_gfx_vmax_mv = bv; st.base_uclk_mhz = bu; st.base_ppt_w = bp;
            st.sim_gfx_max_mhz = if self.freq_gfx[1] > 0 { self.freq_gfx[1] } else { bg };
            st.sim_gfx_vmax_mv = if self.volt_gfx[1] > 0 { self.volt_gfx[1] } else { bv };
            st.sim_offset_c_mv = (self.static_off_c[0] * 1000.0).round() as i32;
            let cur_uclk = self.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(0);
            st.sim_uclk_mhz = if cur_uclk > 0 { cur_uclk } else { bu };
            st.sim_fclk_mhz = self.freq_fclk[1];
            st.sim_ppt_w = if self.power_limit_w > 0 { self.power_limit_w } else { bp };
            st.initialized = true;
            ui.ctx().data_mut(|d| d.insert_temp(gen_id, self.baseline_gen));
        }
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("Universal SMU Envelope Calculator — {} ({})", arch.label(), arch.smu_generation()));
            ui.separator();
            if ui.button("Sync From Active Table").clicked() { st.initialized = false; }
            if ui.button("Apply Values to Table").clicked() {
                self.freq_gfx[1] = st.sim_gfx_max_mhz;
                if self.volt_gfx[1] > 0 { self.volt_gfx[1] = st.sim_gfx_vmax_mv; }
                if arch.has_static_vid_coeffs() { self.static_off_c[0] = (st.sim_offset_c_mv as f64) / 1000.0; }
                if self.freq_fclk[1] > 0 { self.freq_fclk[1] = st.sim_fclk_mhz; self.fclk_boost_freq = st.sim_fclk_mhz; }
                if self.power_limit_w > 0 { self.power_limit_w = st.sim_ppt_w; }
                self.sync_fields_to_buffer();
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                self.status_msg = format!("Applied Calc ({}): GFX {} MHz, Vmax {} mV, Offset {} mV, PPT {} W", arch.label(), st.sim_gfx_max_mhz, st.sim_gfx_vmax_mv, st.sim_offset_c_mv, st.sim_ppt_w);
            }
        });
        ui.add_space(6.0);
        draw_card(ui, "1. Universal What-If Simulation Inputs", |ui| {
            egui::Grid::new("smu_calc_inputs").num_columns(4).spacing([14.0, 6.0]).show(ui, |ui| {
                ui.label("GFX Max Clock:");
                ui.add(egui::DragValue::new(&mut st.sim_gfx_max_mhz).range(300..=3600).speed(5).suffix(" MHz"));
                ui.label(if arch.is_rdna3() && self.volt_gfx[1] == 0 { "AVFS Virtual Vmax:" } else { "GFX Max Voltage:" });
                ui.add(egui::DragValue::new(&mut st.sim_gfx_vmax_mv).range(650..=1350).speed(5).suffix(" mV"));
                ui.end_row();
                ui.label("Memory UCLK (Peak):");
                ui.add(egui::DragValue::new(&mut st.sim_uclk_mhz).range(96..=2500).speed(5).suffix(" MHz"));
                if arch.has_static_vid_coeffs() {
                    ui.label("VID Curve Offset c:");
                    ui.add(egui::DragValue::new(&mut st.sim_offset_c_mv).range(-120..=60).speed(1).suffix(" mV"));
                } else if st.sim_fclk_mhz > 0 {
                    ui.label("Infinity Fabric FCLK:");
                    ui.add(egui::DragValue::new(&mut st.sim_fclk_mhz).range(300..=2600).speed(10).suffix(" MHz"));
                } else {
                    ui.label("Fabric FCLK:");
                    ui.weak("N/A on GCN4/5");
                }
                ui.end_row();
                ui.label("Socket PPT0 Limit:");
                ui.add(egui::DragValue::new(&mut st.sim_ppt_w).range(25..=600).speed(1).suffix(" W"));
                let rec_v = (st.base_gfx_vmax_mv - 75).clamp(975, safe_v_cap);
                if ui.button(format!("Auto Undervolt Sweet-Spot ({} mV)", rec_v)).clicked() {
                    st.sim_gfx_vmax_mv = rec_v;
                    if arch.has_static_vid_coeffs() && st.sim_offset_c_mv == 0 { st.sim_offset_c_mv = -35; }
                    if st.sim_fclk_mhz > 0 { st.sim_fclk_mhz = st.sim_fclk_mhz.max(st.sim_uclk_mhz * 2); }
                }
                ui.end_row();
            });
        });
        let f_ghz = (st.sim_gfx_max_mhz as f64) / 1000.0;
        let has_quad = self.static_volt_a[0].abs() > 1e-7 || self.static_volt_b[0].abs() > 1e-7 || self.static_volt_c[0].abs() > 1e-7;
        let (a, b, c) = (self.static_volt_a[0], self.static_volt_b[0], self.static_volt_c[0]);
        let v_curve_mv = (a * f_ghz * f_ghz + b * f_ghz + c) * 1000.0;
        let v_off_mv = if arch.has_static_vid_coeffs() { st.sim_offset_c_mv as f64 } else { 0.0 };
        let v_eff_mv = (st.sim_gfx_vmax_mv as f64 + v_off_mv).max(0.0);
        let ref_v = (st.base_gfx_vmax_mv as f64).max(850.0);
        let ref_f = (st.base_gfx_max_mhz as f64).max(500.0);
        let ref_u = (st.base_uclk_mhz as f64).max(300.0);
        let ref_ppt = (st.base_ppt_w as f64).max(50.0);
        let base_mem_w = (ref_ppt * 0.22).clamp(12.0, 55.0);
        let base_gfx_w = (ref_ppt - base_mem_w).max(25.0);
        let p_ratio = (st.sim_gfx_max_mhz as f64 / ref_f) * (v_eff_mv / ref_v).powi(2);
        let volt_sav_pct = (1.0 - (v_eff_mv / ref_v).powi(2)) * 100.0;
        let mem_w = base_mem_w * (st.sim_uclk_mhz as f64 / ref_u);
        let gfx_w = base_gfx_w * p_ratio;
        let tot_w = gfx_w + mem_w;
        let head_w = (st.sim_ppt_w as f64) - tot_w;
        let est_tdc = if v_eff_mv > 100.0 { (gfx_w * 1000.0) / v_eff_mv } else { 0.0 };
        draw_card(ui, "2. Voltage Curve (V/F) & Silicon Droop Window", |ui| {
            egui::Grid::new("smu_calc_avfs").num_columns(3).striped(true).spacing([14.0, 6.0]).show(ui, |ui| {
                ui.strong("SMU Metric"); ui.strong("Calculated Value"); ui.strong("Status"); ui.end_row();
                if has_quad {
                    ui.label("Quadratic AVFS V(f) @ Fmax");
                    ui.monospace(format!("{:.1} mV (a={:.4}, b={:.4}, c={:.5})", v_curve_mv, a, b, c));
                    ui.colored_label(egui::Color32::from_rgb(130, 195, 255), "0x74E COEFF"); ui.end_row();
                }
                ui.label("Effective Vmax vs Silicon Cap");
                ui.monospace(format!("{:.1} mV (offset {:+.1} mV | safe <= {} mV)", v_eff_mv, v_off_mv, safe_v_cap));
                let vok = v_eff_mv >= 700.0 && v_eff_mv <= (safe_v_cap as f64);
                ui.colored_label(if vok { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) }, if vok { "IN SAFE WINDOW" } else { "ABOVE CAP" }); ui.end_row();
                if self.dcbtc_max[0] > 0.0 {
                    let dcbtc_max = self.dcbtc_max[0];
                    let dcbtc_gb = if self.dcbtc_gb[0] > 0.0 { self.dcbtc_gb[0] } else { 6.25 };
                    let dctol_val = self.dctol[0];
                    let rail_floor = if st.sim_uclk_mhz >= 850 { 850.0 } else { 775.0 };
                    let droop_mv = (v_eff_mv - rail_floor).max(0.0) * 0.165;
                    let droop_margin = dcbtc_max - droop_mv;
                    ui.label("DCBTC Droop vs GuardBand");
                    ui.monospace(format!("{:.2} / {:.2} mV max (GB {:.2} mV, DcTol {})", droop_mv, dcbtc_max, dcbtc_gb, dctol_val));
                    let (lbl, col) = if droop_margin < dcbtc_gb || head_w < -2.0 { ("DPM LOCK RISK", egui::Color32::from_rgb(245, 75, 95)) } else if droop_margin < dcbtc_gb * 2.2 || head_w < 5.0 { ("MARGINAL", egui::Color32::from_rgb(245, 185, 65)) } else { ("BOOST SAFE", egui::Color32::from_rgb(80, 210, 120)) };
                    ui.colored_label(col, lbl); ui.end_row();
                }
            });
        });
        draw_card(ui, "3. Memory / Fabric Coupling & Power Budget (vs Factory Baseline)", |ui| {
            egui::Grid::new("smu_calc_pwr").num_columns(3).striped(true).spacing([14.0, 6.0]).show(ui, |ui| {
                ui.strong("Subsystem Check"); ui.strong("Formula / Estimate"); ui.strong("Verdict"); ui.end_row();
                ui.label("UCLK Ceiling vs Silicon Spec");
                ui.monospace(format!("{} MHz (factory {} MHz | safe <= {} MHz)", st.sim_uclk_mhz, st.base_uclk_mhz, safe_uclk_cap));
                ui.colored_label(if st.sim_uclk_mhz <= safe_uclk_cap { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) }, if st.sim_uclk_mhz <= safe_uclk_cap { "WITHIN SPEC" } else { "OC / STRAP" }); ui.end_row();
                if st.sim_fclk_mhz > 0 {
                    let fclk_req = st.sim_uclk_mhz * 2;
                    ui.label("Fabric Coupling (FCLK >= 2 * UCLK)");
                    ui.monospace(format!("{} MHz >= {} MHz (2 x {} MHz)", st.sim_fclk_mhz, fclk_req, st.sim_uclk_mhz));
                    ui.colored_label(if st.sim_fclk_mhz >= fclk_req { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 75, 95) }, if st.sim_fclk_mhz >= fclk_req { "SYNCED" } else { "BOTTLENECK" }); ui.end_row();
                }
                ui.label(format!("Dynamic Power Delta vs {} mV", st.base_gfx_vmax_mv));
                ui.monospace(format!("{:+.1}% GFX ({:.1} W GFX + {:.1} W Mem)", -volt_sav_pct, gfx_w, mem_w));
                ui.colored_label(if volt_sav_pct >= 0.0 { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) }, if volt_sav_pct >= 0.0 { "EFFICIENT" } else { "HIGHER DRAW" }); ui.end_row();
                ui.label("Estimated Board Power vs PPT0");
                ui.monospace(format!("{:.1} W / {} W (Headroom: {:+.1} W)", tot_w, st.sim_ppt_w, head_w));
                ui.colored_label(if head_w >= 4.0 { egui::Color32::from_rgb(80, 210, 120) } else if head_w >= 0.0 { egui::Color32::from_rgb(245, 185, 65) } else { egui::Color32::from_rgb(245, 75, 95) }, if head_w >= 4.0 { "HEADROOM OK" } else if head_w >= 0.0 { "TIGHT" } else { "PPT THROTTLE" }); ui.end_row();
                if self.tdc_gfx > 0 {
                    ui.label("Estimated Core Current vs TDC");
                    ui.monospace(format!("{:.1} A / {} A TDC GFX", est_tdc, self.tdc_gfx));
                    ui.colored_label(if est_tdc <= self.tdc_gfx as f64 { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 75, 95) }, if est_tdc <= self.tdc_gfx as f64 { "WITHIN TDC" } else { "TDC CAP" }); ui.end_row();
                }
            });
        });
        ui.ctx().data_mut(|d| d.insert_temp(state_id, st));
    }
}
