use crate::card_layout::draw_card;
use crate::PowerPlayApp;
use eframe::egui;

#[derive(Clone, Debug)]
struct SmuCalcState {
    initialized: bool,
    sim_gfx_max_mhz: i32,
    sim_gfx_vmax_mv: i32,
    sim_uclk_mhz: i32,
    sim_fclk_mhz: i32,
    sim_ppt_w: i32,
}

impl Default for SmuCalcState {
    fn default() -> Self {
        Self { initialized: false, sim_gfx_max_mhz: 2400, sim_gfx_vmax_mv: 1050, sim_uclk_mhz: 875, sim_fclk_mhz: 1800, sim_ppt_w: 100 }
    }
}

impl PowerPlayApp {
    pub fn draw_calc_tab(&mut self, ui: &mut egui::Ui) {
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        let state_id = egui::Id::new("smu_calc_sim_state");
        let gen_id = egui::Id::new("smu_calc_last_gen");
        let last_gen = ui.ctx().data(|d| d.get_temp::<u64>(gen_id).unwrap_or(u64::MAX));
        let mut st = ui.ctx().data(|d| d.get_temp::<SmuCalcState>(state_id).unwrap_or_default());
        if !st.initialized || last_gen != self.baseline_gen {
            st.sim_gfx_max_mhz = if self.freq_gfx[1] > 0 { self.freq_gfx[1] } else { 2400 };
            st.sim_gfx_vmax_mv = if self.volt_gfx[1] > 0 { self.volt_gfx[1] } else { 1050 };
            let peak_uclk = self.mem_dpm_freq.iter().map(|d| d[0]).max().unwrap_or(0);
            st.sim_uclk_mhz = if peak_uclk > 0 { peak_uclk } else { 875 };
            st.sim_fclk_mhz = if self.freq_fclk[1] > 0 { self.freq_fclk[1] } else { 1800 };
            st.sim_ppt_w = if self.power_limit_w > 0 { self.power_limit_w } else { 100 };
            st.initialized = true;
            ui.ctx().data_mut(|d| d.insert_temp(gen_id, self.baseline_gen));
        }
        ui.horizontal_wrapped(|ui| {
            ui.strong(format!("SMU V/F, DCBTC & Power Calculator ({})", arch.label()));
            ui.separator();
            if ui.button("Sync From Table").clicked() { st.initialized = false; }
            if !self.raw_buffer.is_empty() && ui.button("Apply to Table").clicked() {
                self.freq_gfx[1] = st.sim_gfx_max_mhz;
                if self.volt_gfx[1] > 0 { self.volt_gfx[1] = st.sim_gfx_vmax_mv; }
                self.freq_fclk[1] = st.sim_fclk_mhz;
                self.fclk_boost_freq = st.sim_fclk_mhz;
                if self.power_limit_w > 0 { self.power_limit_w = st.sim_ppt_w; }
                self.sync_fields_to_buffer();
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("ppt_dirty"), true));
                self.status_msg = format!("Applied Calc: GFX {} MHz, Vmax {} mV, FCLK {} MHz, PPT {} W", st.sim_gfx_max_mhz, st.sim_gfx_vmax_mv, st.sim_fclk_mhz, st.sim_ppt_w);
            }
        });
        ui.add_space(6.0);
        draw_card(ui, "1. Simulation Inputs (What-If SMU Envelope)", |ui| {
            egui::Grid::new("smu_calc_inputs").num_columns(4).spacing([14.0, 6.0]).show(ui, |ui| {
                ui.label("GFX Max Clock:"); ui.add(egui::DragValue::new(&mut st.sim_gfx_max_mhz).range(500..=3400).speed(5).suffix(" MHz"));
                ui.label("GFX Max Voltage:"); ui.add(egui::DragValue::new(&mut st.sim_gfx_vmax_mv).range(650..=1300).speed(5).suffix(" mV")); ui.end_row();
                ui.label("Memory UCLK (DPM3):"); ui.add(egui::DragValue::new(&mut st.sim_uclk_mhz).range(96..=1500).speed(5).suffix(" MHz"));
                ui.label("Infinity Fabric FCLK:"); ui.add(egui::DragValue::new(&mut st.sim_fclk_mhz).range(400..=2400).speed(10).suffix(" MHz")); ui.end_row();
                ui.label("Socket PPT0 Limit:"); ui.add(egui::DragValue::new(&mut st.sim_ppt_w).range(30..=500).speed(1).suffix(" W"));
                let rec_v = if st.sim_ppt_w <= 110 && st.sim_uclk_mhz >= 850 { 1050 } else { 1100 };
                if ui.button(format!("Auto Sweet-Spot ({} mV)", rec_v)).clicked() { st.sim_gfx_vmax_mv = rec_v; st.sim_fclk_mhz = st.sim_fclk_mhz.max(st.sim_uclk_mhz * 2); }
                ui.end_row();
            });
        });
        let f_ghz = (st.sim_gfx_max_mhz as f64) / 1000.0;
        let (a, b, c) = if self.static_volt_a[0].abs() > 1e-7 || self.static_volt_b[0].abs() > 1e-7 || self.static_volt_c[0].abs() > 1e-7 {
            (self.static_volt_a[0], self.static_volt_b[0], self.static_volt_c[0])
        } else { (0.0654, -0.0590, 0.09231) };
        let (oa, ob, oc) = (self.static_off_a[0], self.static_off_b[0], self.static_off_c[0]);
        let v_curve_mv = (a * f_ghz * f_ghz + b * f_ghz + c) * 1000.0;
        let v_off_mv = (oa * f_ghz * f_ghz + ob * f_ghz + oc) * 1000.0;
        let v_eff_mv = (st.sim_gfx_vmax_mv as f64 + v_off_mv).max(0.0);
        let dcbtc_max = if self.dcbtc_max[0] > 0.0 { self.dcbtc_max[0] } else { 56.25 };
        let dcbtc_gb = if self.dcbtc_gb[0] > 0.0 { self.dcbtc_gb[0] } else { 6.25 };
        let dctol_val = if self.dctol[0] != 0 { self.dctol[0] } else { 225 };
        let p_ratio = (st.sim_gfx_max_mhz as f64 / 2400.0) * (v_eff_mv / 1150.0).powi(2);
        let volt_sav_pct = (1.0 - (v_eff_mv / 1150.0).powi(2)) * 100.0;
        let mem_w = 10.0 + (st.sim_uclk_mhz as f64 / 875.0) * 16.0;
        let gfx_w = 78.0 * p_ratio;
        let tot_w = gfx_w + mem_w;
        let head_w = (st.sim_ppt_w as f64) - tot_w;
        let est_tdc = if v_eff_mv > 100.0 { (gfx_w * 1000.0) / v_eff_mv } else { 0.0 };
        let floor_mv = if st.sim_uclk_mhz >= 850 { 850.0 } else { 775.0 };
        let droop_mv = (v_eff_mv - floor_mv).max(0.0) * 0.165;
        let droop_margin = dcbtc_max - droop_mv;
        let fclk_req = st.sim_uclk_mhz * 2;
        draw_card(ui, "2. AVFS Quadratic V(f) & DCBTC 500 MHz Lock Analyzer", |ui| {
            egui::Grid::new("smu_calc_avfs").num_columns(3).striped(true).spacing([14.0, 6.0]).show(ui, |ui| {
                ui.strong("SMU Metric"); ui.strong("Calculated Value"); ui.strong("Status"); ui.end_row();
                ui.label("Quadratic AVFS V(f) @ Fmax"); ui.monospace(format!("{:.1} mV (a={:.4}, b={:.4}, c={:.5})", v_curve_mv, a, b, c)); ui.colored_label(egui::Color32::from_rgb(130, 195, 255), "0x74E COEFF"); ui.end_row();
                ui.label("Effective Vmax (+Offset c)"); ui.monospace(format!("{:.1} mV (offset {:+.1} mV)", v_eff_mv, v_off_mv));
                let vok = (750.0..=1150.0).contains(&v_eff_mv);
                ui.colored_label(if vok { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) }, if vok { "IN WINDOW" } else { "OUT OF RANGE" }); ui.end_row();
                ui.label("DCBTC Droop vs GuardBand"); ui.monospace(format!("{:.2} / {:.2} mV max (GB {:.2} mV, DcTol {})", droop_mv, dcbtc_max, dcbtc_gb, dctol_val));
                let (lbl, col) = if droop_margin < dcbtc_gb || head_w < -2.0 { ("500 MHz LOCK RISK", egui::Color32::from_rgb(245, 75, 95)) } else if droop_margin < dcbtc_gb * 2.2 || head_w < 5.0 { ("MARGINAL", egui::Color32::from_rgb(245, 185, 65)) } else { ("BOOST SAFE", egui::Color32::from_rgb(80, 210, 120)) };
                ui.colored_label(col, lbl); ui.end_row();
            });
        });
        draw_card(ui, "3. Fabric Ratio (FCLK >= 2x UCLK) & PPT0 / TDC Budget", |ui| {
            egui::Grid::new("smu_calc_pwr").num_columns(3).striped(true).spacing([14.0, 6.0]).show(ui, |ui| {
                ui.strong("Subsystem Check"); ui.strong("Formula / Estimate"); ui.strong("Verdict"); ui.end_row();
                ui.label("Fabric Ratio (FCLK >= 2 * UCLK)"); ui.monospace(format!("{} MHz >= {} MHz (2 x {} MHz)", st.sim_fclk_mhz, fclk_req, st.sim_uclk_mhz));
                ui.colored_label(if st.sim_fclk_mhz >= fclk_req { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 75, 95) }, if st.sim_fclk_mhz >= fclk_req { "SYNCED" } else { "BOTTLENECK" }); ui.end_row();
                ui.label("Dynamic Power Delta vs 1150 mV"); ui.monospace(format!("{:+.1}% GFX ({:.1} W GFX + {:.1} W Mem)", -volt_sav_pct, gfx_w, mem_w));
                ui.colored_label(if volt_sav_pct >= 0.0 { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 185, 65) }, if volt_sav_pct >= 0.0 { "EFFICIENT" } else { "HIGHER DRAW" }); ui.end_row();
                ui.label("Total Board Power vs PPT0"); ui.monospace(format!("{:.1} W / {} W (Headroom: {:+.1} W)", tot_w, st.sim_ppt_w, head_w));
                ui.colored_label(if head_w >= 4.0 { egui::Color32::from_rgb(80, 210, 120) } else if head_w >= 0.0 { egui::Color32::from_rgb(245, 185, 65) } else { egui::Color32::from_rgb(245, 75, 95) }, if head_w >= 4.0 { "HEADROOM OK" } else if head_w >= 0.0 { "TIGHT" } else { "PPT THROTTLE" }); ui.end_row();
                let tdc_lim = if self.tdc_gfx > 0 { self.tdc_gfx } else { 100 };
                ui.label("Estimated Core Current vs TDC"); ui.monospace(format!("{:.1} A / {} A TDC GFX", est_tdc, tdc_lim));
                ui.colored_label(if est_tdc <= tdc_lim as f64 { egui::Color32::from_rgb(80, 210, 120) } else { egui::Color32::from_rgb(245, 75, 95) }, if est_tdc <= tdc_lim as f64 { "WITHIN TDC" } else { "TDC CAP" }); ui.end_row();
            });
        });
        ui.ctx().data_mut(|d| d.insert_temp(state_id, st));
    }
}
