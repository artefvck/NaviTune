mod polaris;
mod rdna1;
mod rdna3;
mod vega;

use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::PowerPlayApp;
use rfd::FileDialog;

impl PowerPlayApp {

    #[inline]
    fn build_smu_mask(smu_features: &[bool; 64]) -> u64 {
        let mut mask = 0u64;
        for (bit, &enabled) in smu_features.iter().enumerate() {
            if enabled {
                mask |= 1u64 << bit;
            }
        }
        mask
    }

    #[inline]
    fn load_od8_flags_10(&mut self, f: &[u8]) {
        if f.len() < 10 { return; }
        self.gpu_clock_limit = f[0] != 0;
        self.gpu_clock_curve = f[1] != 0;
        self.memory_clock_limit = f[2] != 0;
        self.zero_rpm = f[3] != 0;
        self.min_fan_speed = f[4] != 0;
        self.fan_target_temp_limit = f[5] != 0;
        self.operating_temp_limit = f[6] != 0;
        self.fan_acoustic_rpm = f[7] != 0;
        self.fan_curve = f[8] != 0;
        self.power_limit_feat = f[9] != 0;
    }


    #[inline]
    fn load_od8_settings_5(&mut self, od_min: &[u32], od_max: &[u32]) {
        if od_min.len() < 5 || od_max.len() < 5 { return; }
        self.oc_gfx_clock = [od_min[0] as i32, od_max[0] as i32];
        self.oc_mem_clock = [od_min[1] as i32, od_max[1] as i32];
        self.oc_power_limit = [od_min[2] as i32, od_max[2] as i32];
        self.oc_fan_rpm = [od_min[3] as i32, od_max[3] as i32];
        self.oc_fan_acoustic_rpm = [od_min[4] as i32, od_max[4] as i32];
    }

}

impl PowerPlayApp {
    pub(crate) fn load_bytes_for_path(&mut self, path: &std::path::Path, raw: Vec<u8>, label: &str) -> Result<Option<usize>, String> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let mut found_off = None;
        let bytes = match ext.as_str() {
            "reg" => {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                crate::import_formats::parse_reg_ppt(&raw)?
            }
            "mpt" => {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                crate::import_formats::parse_mpt_ppt(&raw)?
            }
            "plist" => {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                crate::import_formats::parse_plist_ppt(&raw)?
            }
            _ if raw.starts_with(&[0x55, 0xAA]) || (ext == "rom" && raw.len() >= 65536) => {
                let (ppt, off) = crate::vbios::extract_ppt_from_rom(&raw)?;
                self.vbios_buffer = Some(raw);
                self.vbios_ppt_offset = Some(off);
                found_off = Some(off);
                ppt
            }
            _ => {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                raw
            }
        };
        self.apply_ppt_bytes(bytes, label);
        Ok(found_off)
    }

    pub fn load_ppt_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("All PowerPlay & UPP Dumps", &["bin", "rom", "reg", "mpt", "plist", "pp_table", "ppt", "dat"])
            .add_filter("All Files (*)", &["*"])
            .pick_file()
        {
            let raw = match crate::io::read_bounded(&path) {
                Ok(d) => d,
                Err(e) => {
                    self.show_error("File Read Error", &e.to_string());
                    return;
                }
            };
            let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
            match self.load_bytes_for_path(&path, raw, &name) {
                Ok(_) => {
                    self.loaded_file = Some(path.clone());
                    self.selected_gpu = usize::MAX;
                    crate::ui_theme::log_debug("LOAD", &format!("Loaded {} ({} B) | {}", path.display(), self.raw_buffer.len(), self.status_msg));
                }
                Err(e) => self.show_error("Import Error", &e),
            }
        }
    }

    pub fn apply_ppt_bytes(&mut self, b: Vec<u8>, label: &str) {
        self.baseline_gen = self.baseline_gen.wrapping_add(1);
        if b.len() < 64 {
            self.vbios_buffer = None;
            self.vbios_ppt_offset = None;
            self.loaded_file = None;
            self.show_error(
                "PP_PhmSoftPowerPlayTable Header Not Found",
                &format!("Could not find a valid PP_PhmSoftPowerPlayTable header in '{}' (buffer size {} bytes is too small).", label, b.len()),
            );
            return;
        }
        let hdr_size = u16::from_le_bytes([b[0], b[1]]) as usize;
        let frev = b[2];
        let crev = b[3];
        let trev = b[4];
        let known_frev = matches!(frev, 0x07 | 0x08 | 0x0B | 0x0C | 0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16);
        if hdr_size != b.len() || !known_frev || trev > 0x06 {
            self.vbios_buffer = None;
            self.vbios_ppt_offset = None;
            self.loaded_file = None;
            self.show_error(
                "PP_PhmSoftPowerPlayTable Header Not Found",
                &format!(
                    "No valid PP_PhmSoftPowerPlayTable header found in '{}'.\n\n\
                     Header bytes: size={} (actual={} B), format_rev=0x{:02X}.{:02X}, table_rev=0x{:02X}.",
                    label, hdr_size, b.len(), frev, crev, trev
                ),
            );
            return;
        }
        let arch_label = match (frev, b.len()) {
            (0x0F, 2470) => "RDNA 2 (Navi 21 Sienna Cichlid)",
            (0x10 | 0x11, 2470) => "RDNA 2 (Navi 22 Navy Flounder)",
            (0x12, _) => "RDNA 2 (Navi 21/22/23)",
            (0x13, _) => "RDNA 2 (Navi 24 Beige Goby)",
            (0x0C, 1674) | (0x0D..=0x0F, _) => "RDNA 1 (Navi 10/12/14)",
            (0x14..=0x16, _) => "RDNA 3 (Navi 31/32/33)",
            (0x0B | 0x0C, _) => "Vega 10/20",
            (0x07 | 0x08, _) => "Polaris (RX 400/500)",
            _ => "Unknown AMD Architecture",
        };
        if self.try_apply_rdna3(&b, label, frev, crev, trev) { return; }
        if self.try_apply_rdna1(&b, label, frev, crev, trev) { return; }
        if self.try_apply_vega20(&b, label, frev, crev, trev) { return; }
        if self.try_apply_polaris(&b, label, frev, crev, trev) { return; }
        if self.try_apply_vega10(&b, label, frev, crev, trev) { return; }

        let supported_rdna2 = matches!(frev, 0x10..=0x13) || (frev == 0x0F && b.len() == 2470);
        if !supported_rdna2 {
            self.show_error(
                "Unsupported PPTable Architecture",
                &format!(
                    "Detected {} (format_revision 0x{:02X}.{:02X}, {} bytes).\n\n\
                     NaviTune supports RDNA 2 (0x0F–0x13) and RDNA 3 (0x14–0x16, 5424 B).",
                    arch_label, frev, crev, b.len()
                ),
            );
            return;
        }

        // Clean all UI fields to default before populating from the newly loaded table
        let saved_vbios = self.vbios_buffer.take();
        let saved_voff = self.vbios_ppt_offset.take();
        let saved_file = self.loaded_file.take();
        let saved_gpu = self.selected_gpu;
        self.reset_all_values();
        self.vbios_buffer = saved_vbios;
        self.vbios_ppt_offset = saved_voff;
        self.loaded_file = saved_file;
        self.selected_gpu = saved_gpu;

        let Some(t) = AtomSmcDpmInfoV4_9::from_bytes(&b) else {
            self.show_error("Invalid RDNA 2 PPTable", &format!("Expected at least {} bytes (AtomSmcDpmInfoV4_9), got {} bytes.", std::mem::size_of::<crate::pptable_rdna2::AtomSmcDpmInfoV4_9>(), b.len()));
            return;
        };

        let f = t.od_feature_flags;
        self.gpu_clock_limit = f[0] != 0; self.gpu_clock_curve = f[1] != 0;
        self.memory_clock_limit = f[2] != 0; self.zero_rpm = f[3] != 0;
        self.min_fan_speed = f[4] != 0; self.fan_target_temp_limit = f[5] != 0;
        self.operating_temp_limit = f[6] != 0; self.fan_acoustic_rpm = f[7] != 0;
        self.fan_curve = f[8] != 0; self.power_limit_feat = f[9] != 0;
        self.timing_tuning = f[10] != 0; self.power_mode = f[11] != 0;
        self.auto_undervolt_gpu = f[12] != 0; self.auto_overclock_gpu = f[13] != 0;
        self.auto_overclock_mem = f[14] != 0; self.auto_fan_acoustic_limit = f[15] != 0;

        let mask = t.features_to_run;
        for (bit, feature) in self.smu_features.iter_mut().enumerate() {
                *feature = (mask & (1u64 << bit)) != 0;
        }

        self.oc_gfx_clock = [t.od_gfx_min[0] as i32, t.od_gfx_max[0] as i32];
        self.oc_mem_clock = [t.od_mem_min[0] as i32, t.od_mem_max[0] as i32];
        self.oc_power_limit = [t.od_pwr_min as i32, t.od_pwr_max as i32];
        self.oc_fan_rpm = [t.od_fan_min as i32, t.od_fan_max as i32];
        self.oc_fan_acoustic_rpm = [t.od_acoustic_min as i32, t.od_acoustic_max as i32];
        self.oc_mem_timing_ctrl = t.od_mem_timing_ctrl as i32;
        self.oc_power_mode_ctrl = t.od_pwr_mode_ctrl as i32;
        self.oc_zero_rpm_ctrl = t.od_zero_rpm_ctrl as i32;

        self.pm_quiet = [t.pm_quiet_limit as i32, t.pm_quiet_temp as i32];
        self.pm_balanced = [t.pm_balanced_limit as i32, t.pm_balanced_temp as i32];
        self.pm_turbo = [t.pm_turbo_limit as i32, t.pm_turbo_temp as i32];
        self.pm_rage = [(t.pm_rage_limit & 0xFFFF) as i32, t.pm_rage_temp as i32];
        self.ac_quiet = [t.ac_quiet[0] as i32, t.ac_quiet[1] as i32];
        self.ac_balanced = [t.ac_balanced[0] as i32, t.ac_balanced[1] as i32];
        self.ac_turbo = [t.ac_turbo[0] as i32, t.ac_turbo[1] as i32];
        self.ac_rage = [t.ac_rage[0] as i32, t.ac_rage[1] as i32];

        self.volt_gfx = [t.min_voltage_gfx as i32 / 4, t.max_voltage_gfx as i32 / 4];
        self.volt_soc = [t.min_voltage_soc as i32 / 4, t.max_voltage_soc as i32 / 4];
        self.power_limit_w = t.socket_power_limit_ac[0] as i32;
        self.probe_offset = format!("0x{:03X}", std::mem::offset_of!(crate::pptable_rdna2::AtomSmcDpmInfoV4_9, socket_power_limit_ac));
        self.probe_write_val = self.power_limit_w.to_string();
        self.tdc_gfx = t.tdc_limit_gfx as i32;
        self.tdc_soc = t.tdc_limit_soc as i32;
        self.ulv_gfx = [t.ulv_voltage_vmin_gfx as i32 / 4, t.ulv_voltage_vmax_gfx as i32 / 4];
        self.ulv_soc = [t.ulv_voltage_vmin_soc as i32 / 4, t.ulv_voltage_vmax_soc as i32 / 4];
        self.tvmin_gfx = [t.tvmin_gfx as i32, t.tvmin_hyst_gfx as i32];
        self.tvmin_soc = [t.tvmin_soc as i32, t.tvmin_hyst_soc as i32];
        self.vmin_lh_gfx = [t.vmin_low_gfx as i32 / 4, t.vmin_high_gfx as i32 / 4];
        self.vmin_lh_soc = [t.vmin_low_soc as i32 / 4, t.vmin_high_soc as i32 / 4];
        for i in 0..4 {
            self.mem_dpm_volt[i] = [t.mem_vddci[i] as i32 / 4, t.mem_mvdd[i] as i32 / 4];
            self.mem_dpm_freq[i] = [t.freq_uclk[i] as i32, t.mem_clk_div[i] as i32];
        }

        self.freq_gfx = [t.freq_gfx_min as i32, t.freq_gfx_max as i32];
        self.freq_soc = [t.freq_soc[0] as i32, t.freq_soc[1] as i32];
        self.freq_vclk = [t.freq_vclk[0] as i32, t.freq_vclk[1] as i32];
        self.freq_dclk = [t.freq_dclk[0] as i32, t.freq_dclk[1] as i32];
        self.freq_dcefclk = [t.freq_dcef[0] as i32, t.freq_dcef[1] as i32];
        self.freq_dispclk = [t.freq_disp[0] as i32, t.freq_disp[1] as i32];
        self.freq_pixclk = [t.freq_pix[0] as i32, t.freq_pix[1] as i32];
        self.freq_phyclk = [t.freq_phy[0] as i32, t.freq_phy[1] as i32];
        self.freq_dtbclk = [t.freq_dtb[0] as i32, t.freq_dtb[1] as i32];
        self.freq_fclk = [t.freq_fclk[0] as i32, t.freq_fclk[1] as i32];
        self.fclk_boost_freq = t.freq_fclk[1] as i32;
        self.pcie_gen_speed = [t.pcie_gen_speed[0] as i32, t.pcie_gen_speed[1] as i32];
        self.pcie_lane_count = [t.pcie_lane_count[0] as i32, t.pcie_lane_count[1] as i32];
        self.lclk_mhz = [t.lclk_freq[0] as i32, t.lclk_freq[1] as i32];

        self.static_volt_a = [t.static_volt_gfx.a as f64, t.static_volt_soc.a as f64];
        self.static_volt_b = [t.static_volt_gfx.b as f64, t.static_volt_soc.b as f64];
        self.static_volt_c = [t.static_volt_gfx.c as f64, t.static_volt_soc.c as f64];
        self.static_off_a = [t.static_volt_offset_gfx.a as f64, t.static_volt_offset_soc.a as f64];
        self.static_off_b = [t.static_volt_offset_gfx.b as f64, t.static_volt_offset_soc.b as f64];
        self.static_off_c = [t.static_volt_offset_gfx.c as f64, t.static_volt_offset_soc.c as f64];
        self.dcbtc_gfx_en = t.dcbtc_enable_gfx != 0;
        self.dcbtc_soc_en = t.dcbtc_enable_soc != 0;
        self.dcbtc_min = [t.dcbtc_min_gfx as f64 / 4.0, t.dcbtc_min_soc as f64 / 4.0];
        self.dcbtc_max = [t.dcbtc_max_gfx as f64 / 4.0, t.dcbtc_max_soc as f64 / 4.0];
        self.dcbtc_gb = [t.dcbtc_gb_gfx as f64 / 4.0, t.dcbtc_gb_soc as f64 / 4.0];
        self.dctol = [t.dctol_gfx as i32, t.dctol_soc as i32];

        self.fan_pwm_min = t.fan_pwm_min as i32;
        self.fan_acoustic_limit = t.fan_acoustic_limit as i32;
        self.fan_throttling_rpm = t.fan_throttling_rpm as i32;
        self.fan_max_rpm = t.fan_max_rpm as i32;
        self.fan_target_temp = t.fan_target_temp as i32;
        self.fan_target_gfx_clock = t.fan_target_gfx_clock as i32;
        self.fan_mode = t.fan_mode as i32;
        self.fan_hyst_ctrl = t.fan_hyst_ctrl as i32;
        self.fan_zero_rpm_enable = t.fan_zero_rpm_enable != 0;
        self.fan_stop_temp = t.fan_stop_temp as i32;
        self.fan_start_temp = t.fan_start_temp as i32;
        self.od_fan_temp = [t.od_fan_curve_min[0].temp as i32, t.od_fan_curve_max[0].temp as i32];
        self.od_fan_speed = [t.od_fan_curve_min[0].pwm as i32, t.od_fan_curve_max[0].pwm as i32];
        for i in 0..5 {
            self.fan_curve_temp[i] = t.od_fan_curve_min[i].temp as i32;
            self.fan_curve_pwm[i] = t.od_fan_curve_min[i].pwm as i32;
        }
        for i in 0..9 {
            self.temp_limits[i] = t.temp_limit[i] as i32;
        }
        for i in 0..4 {
            self.power_limit_ac[i] = t.power_limit_ac[i] as i32;
        }

        let vbios_tag = if let Some(voff) = self.vbios_ppt_offset {
            format!(" | VBIOS PP_PhmSoftPowerPlayTable @ 0x{:04X}", voff)
        } else {
            String::new()
        };
        self.status_msg = format!("Loaded {} [{} | rev 0x{:02X}.{:02X} | {} B{}]", label, arch_label, frev, crev, b.len(), vbios_tag);
        self.baseline_buffer = b.clone();
            self.raw_buffer = b;
    }

    pub fn sync_fields_to_buffer(&mut self) {
        if self.try_sync_rdna3() { return; }
        if self.try_sync_rdna1() { return; }
        if self.try_sync_vega20() { return; }
        if self.try_sync_polaris() { return; }
        if self.try_sync_vega10() { return; }

        let Some(t) = AtomSmcDpmInfoV4_9::from_bytes_mut(&mut self.raw_buffer) else {
            return;
        };

        t.od_feature_flags = [
            self.gpu_clock_limit as u8, self.gpu_clock_curve as u8, self.memory_clock_limit as u8, self.zero_rpm as u8,
            self.min_fan_speed as u8, self.fan_target_temp_limit as u8, self.operating_temp_limit as u8, self.fan_acoustic_rpm as u8,
            self.fan_curve as u8, self.power_limit_feat as u8, self.timing_tuning as u8, self.power_mode as u8,
            self.auto_undervolt_gpu as u8, self.auto_overclock_gpu as u8, self.auto_overclock_mem as u8, self.auto_fan_acoustic_limit as u8,
        ];

        let mut mask = 0u64;
        for (bit, &en) in self.smu_features.iter().enumerate() {
            if en { mask |= 1u64 << bit; }
        }
        t.features_to_run = mask;

        t.od_gfx_min[0] = self.oc_gfx_clock[0] as u32; t.od_gfx_max[0] = self.oc_gfx_clock[1] as u32;
        t.od_mem_min[0] = self.oc_mem_clock[0] as u32; t.od_mem_max[0] = self.oc_mem_clock[1] as u32;
        t.od_pwr_min = self.oc_power_limit[0] as u32; t.od_pwr_max = self.oc_power_limit[1] as u32;
        t.od_fan_min = self.oc_fan_rpm[0] as u32; t.od_fan_max = self.oc_fan_rpm[1] as u32;
        t.od_acoustic_min = self.oc_fan_acoustic_rpm[0] as u32; t.od_acoustic_max = self.oc_fan_acoustic_rpm[1] as u32;
        t.od_mem_timing_ctrl = self.oc_mem_timing_ctrl as u32;
        t.od_pwr_mode_ctrl = self.oc_power_mode_ctrl as u32;
        t.od_zero_rpm_ctrl = self.oc_zero_rpm_ctrl as u32;

        t.pm_quiet_limit = self.pm_quiet[0] as u32; t.pm_quiet_temp = self.pm_quiet[1] as u16;
        t.pm_balanced_limit = self.pm_balanced[0] as u32; t.pm_balanced_temp = self.pm_balanced[1] as u16;
        t.pm_turbo_limit = self.pm_turbo[0] as u32; t.pm_turbo_temp = self.pm_turbo[1] as u16;
        t.pm_rage_limit = (t.pm_rage_limit & 0xFFFF_0000) | (self.pm_rage[0] as u16 as u32); t.pm_rage_temp = self.pm_rage[1] as u16;
        t.ac_quiet = [self.ac_quiet[0] as u16, self.ac_quiet[1] as u16];
        t.ac_balanced = [self.ac_balanced[0] as u16, self.ac_balanced[1] as u16];
        t.ac_turbo = [self.ac_turbo[0] as u16, self.ac_turbo[1] as u16];
        t.ac_rage = [self.ac_rage[0] as u16, self.ac_rage[1] as u16];

        let qmv = |ui_mv: i32, orig: u16| -> u16 {
            if (orig / 4) as i32 == ui_mv { orig } else { (ui_mv.max(0) * 4) as u16 }
        };
        t.ulv_voltage_vmin_gfx = qmv(self.ulv_gfx[0], t.ulv_voltage_vmin_gfx);
        t.ulv_voltage_vmin_soc = qmv(self.ulv_soc[0], t.ulv_voltage_vmin_soc);
        t.ulv_voltage_vmax_gfx = qmv(self.ulv_gfx[1], t.ulv_voltage_vmax_gfx);
        t.ulv_voltage_vmax_soc = qmv(self.ulv_soc[1], t.ulv_voltage_vmax_soc);
        t.min_voltage_gfx = qmv(self.volt_gfx[0], t.min_voltage_gfx);
        t.min_voltage_soc = qmv(self.volt_soc[0], t.min_voltage_soc);
        t.max_voltage_gfx = qmv(self.volt_gfx[1], t.max_voltage_gfx);
        t.max_voltage_soc = qmv(self.volt_soc[1], t.max_voltage_soc);
        t.tvmin_gfx = self.tvmin_gfx[0] as u16; t.tvmin_hyst_gfx = self.tvmin_gfx[1] as u16;
        t.tvmin_soc = self.tvmin_soc[0] as u16; t.tvmin_hyst_soc = self.tvmin_soc[1] as u16;
        t.vmin_low_gfx = qmv(self.vmin_lh_gfx[0], t.vmin_low_gfx);
        t.vmin_high_gfx = qmv(self.vmin_lh_gfx[1], t.vmin_high_gfx);
        t.vmin_low_soc = qmv(self.vmin_lh_soc[0], t.vmin_low_soc);
        t.vmin_high_soc = qmv(self.vmin_lh_soc[1], t.vmin_high_soc);
        for i in 0..4 {
            t.mem_vddci[i] = (self.mem_dpm_volt[i][0] * 4) as u16;
            t.mem_mvdd[i] = (self.mem_dpm_volt[i][1] * 4) as u16;
            t.freq_uclk[i] = self.mem_dpm_freq[i][0] as u16;
            t.mem_clk_div[i] = self.mem_dpm_freq[i][1] as u8;
        }

        t.freq_gfx_min = self.freq_gfx[0] as u16; t.freq_gfx_max = self.freq_gfx[1] as u16; t.freq_gfx_max_mirror = self.freq_gfx[1] as u16;
        t.freq_soc = [self.freq_soc[0] as u16, self.freq_soc[1] as u16];
        t.freq_vclk = [self.freq_vclk[0] as u16, self.freq_vclk[1] as u16];
        t.freq_dclk = [self.freq_dclk[0] as u16, self.freq_dclk[1] as u16];
        t.freq_dcef = [self.freq_dcefclk[0] as u16, self.freq_dcefclk[1] as u16];
        t.freq_disp = [self.freq_dispclk[0] as u16, self.freq_dispclk[1] as u16];
        t.freq_pix = [self.freq_pixclk[0] as u16, self.freq_pixclk[1] as u16];
        t.freq_phy[1] = self.freq_phyclk[1] as u16;
        t.freq_dtb = [self.freq_dtbclk[0] as u16, self.freq_dtbclk[1] as u16];
        t.freq_fclk = [self.freq_fclk[0] as u16, self.freq_fclk[1] as u16];
        t.pcie_gen_speed = [self.pcie_gen_speed[0] as u8, self.pcie_gen_speed[1] as u8];
        t.pcie_lane_count = [self.pcie_lane_count[0] as u8, self.pcie_lane_count[1] as u8];
        t.lclk_freq = [self.lclk_mhz[0] as u16, self.lclk_mhz[1] as u16];

        t.static_volt_gfx.a = self.static_volt_a[0] as f32; t.static_volt_soc.a = self.static_volt_a[1] as f32;
        t.static_volt_gfx.b = self.static_volt_b[0] as f32; t.static_volt_soc.b = self.static_volt_b[1] as f32;
        t.static_volt_gfx.c = self.static_volt_c[0] as f32; t.static_volt_soc.c = self.static_volt_c[1] as f32;
        t.static_volt_offset_gfx.a = self.static_off_a[0] as f32; t.static_volt_offset_soc.a = self.static_off_a[1] as f32;
        t.static_volt_offset_gfx.b = self.static_off_b[0] as f32; t.static_volt_offset_soc.b = self.static_off_b[1] as f32;
        t.static_volt_offset_gfx.c = self.static_off_c[0] as f32; t.static_volt_offset_soc.c = self.static_off_c[1] as f32;
        t.dcbtc_enable_gfx = self.dcbtc_gfx_en as u8; t.dcbtc_enable_soc = self.dcbtc_soc_en as u8;
        t.dcbtc_min_gfx = (self.dcbtc_min[0] * 4.0).round() as i16; t.dcbtc_min_soc = (self.dcbtc_min[1] * 4.0).round() as i16;
        t.dcbtc_max_gfx = (self.dcbtc_max[0] * 4.0).round() as i16; t.dcbtc_max_soc = (self.dcbtc_max[1] * 4.0).round() as i16;
        t.dcbtc_gb_gfx = (self.dcbtc_gb[0] * 4.0).round() as i16; t.dcbtc_gb_soc = (self.dcbtc_gb[1] * 4.0).round() as i16;
        t.dctol_gfx = self.dctol[0] as i16; t.dctol_soc = self.dctol[1] as i16;

        t.fan_pwm_min = self.fan_pwm_min as u16; t.fan_acoustic_limit = self.fan_acoustic_limit as u16;
        t.fan_throttling_rpm = self.fan_throttling_rpm as u16; t.fan_max_rpm = self.fan_max_rpm as u16;
        t.fan_target_temp = self.fan_target_temp as u16; t.fan_target_gfx_clock = self.fan_target_gfx_clock as u16;
        t.fan_mode = self.fan_mode as u8;
        t.fan_hyst_ctrl = self.fan_hyst_ctrl as u8;
        t.fan_zero_rpm_enable = self.fan_zero_rpm_enable as u8;
        t.fan_stop_temp = self.fan_stop_temp as u16; t.fan_start_temp = self.fan_start_temp as u16;
        for i in 0..5 {
            t.od_fan_curve_min[i].temp = self.fan_curve_temp[i] as u32;
            t.od_fan_curve_min[i].pwm = self.fan_curve_pwm[i] as u32;
        }
        t.od_fan_curve_min[0].temp = self.od_fan_temp[0] as u32;
        t.od_fan_curve_max[0].temp = self.od_fan_temp[1] as u32;
        for i in 0..9 {
            t.temp_limit[i] = self.temp_limits[i] as u16;
        }
        for i in 0..4 {
            t.power_limit_ac[i] = self.power_limit_ac[i] as u16;
        }
        t.socket_power_limit_ac[0] = self.power_limit_w as u16;
    }


    pub fn save_ppt_file(&mut self) {
        let default_name = if self.vbios_buffer.is_some() { "patched_vbios.rom" } else { "custom_ppt.bin" };
        self.save_ppt_file_with_default(default_name);
    }

    pub fn save_ppt_file_with_default(&mut self, default_name: &str) {
        if self.raw_buffer.is_empty() {
            self.show_error("Nothing to Save", "Please load a PowerPlay table (.bin, .rom, .reg, .mpt) or select a GPU from ioreg first.");
            return;
        }
        if let Some(path) = FileDialog::new()
            .set_file_name(default_name)
            .add_filter("PowerPlay Binary (*.bin)", &["bin"])
            .add_filter("Patched VBIOS ROM (*.rom)", &["rom"])
            .add_filter("Windows Registry (*.reg)", &["reg"])
            .add_filter("MorePowerTool Profile (*.mpt)", &["mpt"])
            .add_filter("OpenCore DeviceProperties (*.plist)", &["plist"])
            .add_filter("macOS Injector Kext (*.kext)", &["kext"])
            .save_file()
        {
            self.sync_fields_to_buffer();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("bin").to_ascii_lowercase();
            if ext == "kext" {
                match crate::advance_os::export_macos_injector_kext(&self.raw_buffer, &path) {
                    Ok(msg) => {
                        crate::ui_theme::log_debug("SAVE", &msg);
                        self.status_msg = format!("Saved to {}", path.file_name().unwrap_or_default().to_string_lossy());
                    }
                    Err(e) => self.show_error("macOS Kext Export Error", &e),
                }
                return;
            }
            let bytes = match crate::export_formats::serialize_export_bytes(
                &ext,
                &self.raw_buffer,
                self.vbios_buffer.as_deref(),
                self.vbios_ppt_offset,
                "PciRoot(0x0)/Pci(0x1,0x0)/Pci(0x0,0x0)",
            ) {
                Ok(b) => b,
                Err(err) => {
                    let title = if ext == "rom" && self.vbios_buffer.is_none() { "Missing Base VBIOS" } else { "VBIOS Patch Error" };
                    self.show_error(title, &err);
                    return;
                }
            };
            let write_res = crate::io::write_atomic(&path, &bytes);
            match write_res {
                Ok(_) => {
                    crate::ui_theme::log_debug("SAVE", &format!("Exported {} ({} B) | GFX={:?} MHz, Power={} W, Volt={:?} mV", path.display(), self.raw_buffer.len(), self.freq_gfx, self.power_limit_w, self.volt_gfx));
                    self.status_msg = format!("Saved to {}", path.file_name().unwrap_or_default().to_string_lossy());
                }
                Err(e) => self.show_error("File Save Error", &e.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod rdna2_sync_roundtrip_tests {
    use crate::PowerPlayApp;

    #[test]
    fn rdna2_sync_persists_temp_limits_and_power_limit_ac() {
        let mut raw = vec![0u8; 2470];
        raw[0..2].copy_from_slice(&2470u16.to_le_bytes());
        raw[2] = 0x0F;
        let mut app = PowerPlayApp::default();
        app.apply_ppt_bytes(raw, "rdna2_test.bin");
        app.temp_limits[0] = 98;
        app.temp_limits[1] = 108;
        app.temp_limits[2] = 102;
        app.power_limit_w = 235;
        app.sync_fields_to_buffer();

        let mut app2 = PowerPlayApp::default();
        app2.apply_ppt_bytes(app.raw_buffer, "rdna2_reloaded.bin");
        assert_eq!(app2.temp_limits[0], 98);
        assert_eq!(app2.temp_limits[1], 108);
        assert_eq!(app2.temp_limits[2], 102);
        assert_eq!(app2.power_limit_w, 235);
    }
}
