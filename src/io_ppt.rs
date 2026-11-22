use crate::pptable_rdna2::AtomSmcDpmInfoV4_9;
use crate::PowerPlayApp;
use rfd::FileDialog;
use std::fs;

impl PowerPlayApp {
    #[inline]
    fn load_smu_mask(&mut self, mask: u64) {
        for bit in 0..64 {
            self.smu_features[bit] = (mask & (1u64 << bit)) != 0;
        }
    }

    #[inline]
    fn build_smu_mask(smu_features: &[bool; 64]) -> u64 {
        let mut mask = 0u64;
        for bit in 0..64 {
            if smu_features[bit] {
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
    fn save_od8_flags_10(&self, f: &mut [u8]) {
        if f.len() < 10 { return; }
        f[0] = u8::from(self.gpu_clock_limit);
        f[1] = u8::from(self.gpu_clock_curve);
        f[2] = u8::from(self.memory_clock_limit);
        f[3] = u8::from(self.zero_rpm);
        f[4] = u8::from(self.min_fan_speed);
        f[5] = u8::from(self.fan_target_temp_limit);
        f[6] = u8::from(self.operating_temp_limit);
        f[7] = u8::from(self.fan_acoustic_rpm);
        f[8] = u8::from(self.fan_curve);
        f[9] = u8::from(self.power_limit_feat);
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

    #[inline]
    fn save_od8_settings_5(&self, od_min: &mut [u32], od_max: &mut [u32]) {
        if od_min.len() < 5 || od_max.len() < 5 { return; }
        od_min[0] = self.oc_gfx_clock[0].max(0) as u32;
        od_max[0] = self.oc_gfx_clock[1].max(0) as u32;
        od_min[1] = self.oc_mem_clock[0].max(0) as u32;
        od_max[1] = self.oc_mem_clock[1].max(0) as u32;
        od_min[2] = self.oc_power_limit[0] as u32;
        od_max[2] = self.oc_power_limit[1] as u32;
        od_min[3] = self.oc_fan_rpm[0].max(0) as u32;
        od_max[3] = self.oc_fan_rpm[1].max(0) as u32;
        od_min[4] = self.oc_fan_acoustic_rpm[0].max(0) as u32;
        od_max[4] = self.oc_fan_acoustic_rpm[1].max(0) as u32;
    }
}

impl PowerPlayApp {
    pub fn load_ppt_file(&mut self) {
        if let Some(path) = FileDialog::new()
            .add_filter("All PowerPlay & UPP Dumps", &["bin", "rom", "reg", "mpt", "plist", "pp_table", "ppt", "dat"])
            .add_filter("All Files (*)", &["*"])
            .pick_file()
        {
            let raw = match fs::read(&path) {
                Ok(d) => d,
                Err(e) => {
                    self.show_error("File Read Error", &e.to_string());
                    return;
                }
            };
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
            let parsed = if ext == "reg" {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                crate::import_formats::parse_reg_ppt(&raw)
            } else if ext == "mpt" {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                crate::import_formats::parse_mpt_ppt(&raw)
            } else if ext == "plist" {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                crate::import_formats::parse_plist_ppt(&raw)
            } else if ext != "mpt" && ext != "reg" && (raw.starts_with(&[0x55, 0xAA]) || (ext == "rom" && raw.len() >= 65536)) {
                match crate::vbios::extract_ppt_from_rom(&raw) {
                    Ok((ppt, off)) => {
                        self.vbios_buffer = Some(raw);
                        self.vbios_ppt_offset = Some(off);
                        Ok(ppt)
                    }
                    Err(e) => Err(e),
                }
            } else {
                self.vbios_buffer = None;
                self.vbios_ppt_offset = None;
                Ok(raw)
            };

            match parsed {
                Ok(bytes) => {
                    let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                    self.loaded_file = Some(path.clone());
                    self.selected_gpu = usize::MAX;
                    self.apply_ppt_bytes(bytes, &name);
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
            (0x0F, _) => "RDNA 1 (Navi 10/12/14)",
            (0x14, _) => "RDNA 3 (Navi 31/32/33)",
            (0x0B | 0x0C, _) => "Vega 10/20",
            (0x07 | 0x08, _) => "Polaris (RX 400/500)",
            _ => "Unknown AMD Architecture",
        };
        if let Some(t3) = crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(&b) {
            let chip = t3.detect_chip(0);
            let saved_vbios = self.vbios_buffer.take();
            let saved_voff = self.vbios_ppt_offset.take();
            let saved_file = self.loaded_file.take();
            let saved_gpu = self.selected_gpu;
            self.reset_all_values();
            self.vbios_buffer = saved_vbios;
            self.vbios_ppt_offset = saved_voff;
            self.loaded_file = saved_file;
            self.selected_gpu = saved_gpu;

            let f = t3.od_feature_flags;
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
            self.timing_tuning = f[10] != 0;
            self.power_mode = f[11] != 0;

            let mask = { t3.features_to_run };
            for bit in 0..64 {
                self.smu_features[bit] = (mask & (1u64 << bit)) != 0;
            }

            self.oc_gfx_clock = [0, { t3.od_gfx_max } as i32];
            self.oc_mem_clock = [0, { t3.od_mem_max } as i32];
            self.oc_power_limit = [0, { t3.od_pwr_max } as i32];
            self.oc_fan_rpm = [0, { t3.od_fan_max } as i32];
            self.oc_fan_acoustic_rpm = [0, { t3.od_acoustic_max } as i32];

            self.power_limit_w = { t3.socket_power_limit_ac } as i32;
            self.volt_gfx = [{ t3.min_voltage_gfx } as i32 / 4, { t3.max_voltage_gfx } as i32 / 4];
            self.volt_soc = [{ t3.min_voltage_soc } as i32 / 4, { t3.max_voltage_soc } as i32 / 4];
            self.ulv_gfx = [{ t3.ulv_voltage_vmin_gfx } as i32 / 4, 0];
            self.ulv_soc = [{ t3.ulv_voltage_vmin_soc } as i32 / 4, 0];

            self.freq_gfx = [{ t3.freq_gfx_min } as i32, { t3.freq_gfx_max } as i32];
            self.freq_soc = [500, { t3.freq_soc_max } as i32];
            self.freq_fclk = [{ t3.freq_fclk_min } as i32, { t3.freq_fclk_max } as i32];
            self.tdc_gfx = { t3.tdc_limit_gfx } as i32;
            self.tdc_soc = { t3.tdc_limit_soc } as i32;
            for i in 0..4 {
                self.mem_dpm_freq[i] = [t3.freq_uclk[i] as i32, 1];
                self.mem_dpm_volt[i] = [(t3.mem_vddci[i] / 4) as i32, (t3.mem_mvdd[i] / 4) as i32];
            }
            self.pcie_gen_speed = [t3.pcie_gen_speed[0] as i32, t3.pcie_gen_speed[2] as i32];
            self.pcie_lane_count = [t3.pcie_lane_count[0] as i32, t3.pcie_lane_count[2] as i32];
            self.lclk_mhz = [t3.lclk_freq[0] as i32, t3.lclk_freq[2] as i32];
            self.fan_throttling_rpm = { t3.fan_throttling_rpm } as i32;
            self.fan_pwm_min = { t3.fan_pwm_min } as i32;
            self.fan_acoustic_limit = { t3.fan_acoustic_limit } as i32;
            self.fan_max_rpm = { t3.fan_max_rpm } as i32;
            self.fan_target_gfx_clock = { t3.fan_target_gfx_clock } as i32;
            let raw_tt = { t3.fan_target_temp } as i32;
            self.fan_target_temp = if raw_tt > 200 { raw_tt / 10 } else { raw_tt };
            self.fan_zero_rpm_enable = t3.fan_zero_rpm_enable != 0;
            self.fan_stop_temp = { t3.fan_stop_temp } as i32;
            self.fan_start_temp = { t3.fan_start_temp } as i32;
            for i in 0..9 {
                self.temp_limits[i] = t3.temp_limit[i] as i32;
            }

            let vbios_tag = if let Some(voff) = self.vbios_ppt_offset {
                format!(" | VBIOS PowerPlayInfo @ 0x{:04X}", voff)
            } else {
                String::new()
            };
            self.status_msg = format!(
                "Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B{}]",
                label, chip.display_name(), frev, crev, trev, b.len(), vbios_tag
            );
            self.raw_buffer = b;
            return;
        }

        if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(&b) {
            let did = self.vbios_buffer.as_deref()
                .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id)
                .unwrap_or(0);
            let chip1 = t1.detect_chip(did);

            let f = { t1.od_feature_flags };
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
            self.timing_tuning = f[10] != 0;
            self.power_mode = f[11] != 0;

            let mask = { t1.features_to_run };
            for bit in 0..64 {
                self.smu_features[bit] = (mask & (1u64 << bit)) != 0;
            }

            let od_min = { t1.od_settings_min };
            let od_max = { t1.od_settings_max };
            self.oc_gfx_clock = [od_min[0] as i32, od_max[0] as i32];
            self.oc_mem_clock = [od_min[1] as i32, od_max[1] as i32];
            self.oc_power_limit = [od_min[2] as i32, od_max[2] as i32];
            self.oc_fan_rpm = [od_min[3] as i32, od_max[3] as i32];
            self.oc_fan_acoustic_rpm = [od_min[4] as i32, od_max[4] as i32];

            let ppt0 = { t1.socket_power_limit_ac[0] } as i32;
            self.power_limit_w = if ppt0 > 0 { ppt0 } else { ({ t1.boost_power_limit }) as i32 };
            self.tdc_gfx = { t1.tdc_limit_gfx } as i32;
            self.tdc_soc = { t1.tdc_limit_soc } as i32;

            self.volt_gfx = [{ t1.min_voltage_gfx } as i32 / 4, { t1.max_voltage_gfx } as i32 / 4];
            self.volt_soc = [{ t1.min_voltage_soc } as i32 / 4, { t1.max_voltage_soc } as i32 / 4];
            self.ulv_gfx = [{ t1.min_voltage_ulv_gfx } as i32 / 4, 0];
            self.ulv_soc = [{ t1.min_voltage_ulv_soc } as i32 / 4, 0];

            let desc = { t1.dpm_descriptor };
            let gfx_n = (desc[0] as usize).clamp(1, 16);
            let soc_n = (desc[1] as usize).clamp(1, 8);
            let fclk_n = (desc[3] as usize).clamp(1, 8);
            let dclk_n = (desc[4] as usize).clamp(1, 8);
            let vclk_n = (desc[5] as usize).clamp(1, 8);
            let dcef_n = (desc[6] as usize).clamp(1, 8);
            let disp_n = (desc[7] as usize).clamp(1, 8);
            let pix_n = (desc[8] as usize).clamp(1, 8);
            let phy_n = (desc[9] as usize).clamp(1, 8);

            let ft_gfx = { t1.freq_table_gfx };
            let ft_soc = { t1.freq_table_socclk };
            let ft_uclk = { t1.freq_table_uclk };
            let ft_fclk = { t1.freq_table_fclk };
            let ft_vclk = { t1.freq_table_vclk };
            let ft_dclk = { t1.freq_table_dclk };
            let ft_dcef = { t1.freq_table_dcefclk };
            let ft_disp = { t1.freq_table_dispclk };
            let ft_pix = { t1.freq_table_pixclk };
            let ft_phy = { t1.freq_table_phyclk };

            self.freq_gfx = [ft_gfx[0] as i32, ft_gfx[gfx_n - 1] as i32];
            self.freq_soc = [ft_soc[0] as i32, ft_soc[soc_n - 1] as i32];
            self.freq_fclk = [ft_fclk[0] as i32, ft_fclk[fclk_n - 1] as i32];
            self.freq_vclk = [ft_vclk[0] as i32, ft_vclk[vclk_n - 1] as i32];
            self.freq_dclk = [ft_dclk[0] as i32, ft_dclk[dclk_n - 1] as i32];
            self.freq_dcefclk = [ft_dcef[0] as i32, ft_dcef[dcef_n - 1] as i32];
            self.freq_dispclk = [ft_disp[0] as i32, ft_disp[disp_n - 1] as i32];
            self.freq_pixclk = [ft_pix[0] as i32, ft_pix[pix_n - 1] as i32];
            self.freq_phyclk = [ft_phy[0] as i32, ft_phy[phy_n - 1] as i32];

            let vddci_hi = { t1.vddci_max } as i32 / 4;
            let mvdd_hi = { t1.mvdd_max } as i32 / 4;
            for i in 0..4 {
                self.mem_dpm_freq[i] = [ft_uclk[i] as i32, 1];
                self.mem_dpm_volt[i] = [vddci_hi, mvdd_hi];
            }

            self.fan_pwm_min = { t1.fan_pwm_min } as i32;
            self.fan_acoustic_limit = { t1.fan_acoustic_limit_rpm } as i32;
            self.fan_throttling_rpm = { t1.fan_throttling_rpm } as i32;
            self.fan_max_rpm = { t1.fan_maximum_rpm } as i32;
            self.fan_target_temp = { t1.fan_target_temperature } as i32;
            self.fan_target_gfx_clock = { t1.fan_target_gfxclk } as i32;
            self.fan_zero_rpm_enable = t1.fan_zero_rpm_enable != 0;
            self.fan_stop_temp = { t1.fan_stop_temp } as i32;
            self.fan_start_temp = { t1.fan_start_temp } as i32;

            let tl = { t1.temp_limit };
            for i in 0..7 {
                self.temp_limits[i] = tl[i] as i32;
            }

            let vbios_tag = if let Some(voff) = self.vbios_ppt_offset {
                format!(" | VBIOS PowerPlayInfo @ 0x{:04X}", voff)
            } else {
                String::new()
            };
            self.status_msg = format!(
                "Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B{}]",
                label, chip1.display_name(), frev, crev, trev, b.len(), vbios_tag
            );
            self.raw_buffer = b;
            return;
        }

        if let Some(v20) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes(&b) {
            let did = self.vbios_buffer.as_deref()
                .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id)
                .unwrap_or(0);
            let chip_v = crate::pptable_vega::VegaChip::from_did_and_rev(did, frev);

            let f = { v20.od_feature_flags };
            self.load_od8_flags_10(&f);
            let od_min = { v20.od_settings_min };
            let od_max = { v20.od_settings_max };
            self.load_od8_settings_5(&od_min, &od_max);

            let ppt0 = { v20.socket_power_limit_ac[0] } as i32;
            self.power_limit_w = if ppt0 > 0 { ppt0 } else { ({ v20.boost_power_limit }) as i32 };
            self.tdc_gfx = { v20.tdc_limit_gfx } as i32;
            self.tdc_soc = { v20.tdc_limit_soc } as i32;
            self.volt_gfx = [{ v20.min_voltage_gfx } as i32 / 4, { v20.max_voltage_gfx } as i32 / 4];
            self.volt_soc = [{ v20.min_voltage_soc } as i32 / 4, { v20.max_voltage_soc } as i32 / 4];

            let desc = { v20.dpm_descriptor };
            let gfx_n = (desc[0] as usize).clamp(1, 16);
            let soc_n = (desc[1] as usize).clamp(1, 8);
            let fclk_n = (desc[3] as usize).clamp(1, 8);
            let ft_gfx = { v20.freq_table_gfx };
            let ft_soc = { v20.freq_table_socclk };
            let ft_uclk = { v20.freq_table_uclk };
            let ft_fclk = { v20.freq_table_fclk };
            self.freq_gfx = [ft_gfx[0] as i32, ft_gfx[gfx_n - 1] as i32];
            self.freq_soc = [ft_soc[0] as i32, ft_soc[soc_n - 1] as i32];
            self.freq_fclk = [ft_fclk[0] as i32, ft_fclk[fclk_n - 1] as i32];
            let vddci_hi = { v20.vddci_max } as i32 / 4;
            let mvdd_hi = { v20.mvdd_max } as i32 / 4;
            for i in 0..4 {
                self.mem_dpm_freq[i] = [ft_uclk[i] as i32, 1];
                self.mem_dpm_volt[i] = [vddci_hi, mvdd_hi];
            }
            self.fan_pwm_min = { v20.fan_pwm_min } as i32;
            self.fan_acoustic_limit = { v20.fan_acoustic_limit_rpm } as i32;
            self.fan_throttling_rpm = { v20.fan_throttling_rpm } as i32;
            self.fan_max_rpm = { v20.fan_maximum_rpm } as i32;
            self.fan_target_temp = { v20.fan_target_temperature } as i32;
            self.fan_target_gfx_clock = { v20.fan_target_gfxclk } as i32;
            self.ulv_gfx = [{ v20.min_voltage_ulv_gfx } as i32 / 4, 0];
            self.ulv_soc = [{ v20.min_voltage_ulv_soc } as i32 / 4, 0];
            self.fan_zero_rpm_enable = v20.fan_zero_rpm_enable != 0;
            self.fan_stop_temp = { v20.fan_stop_temp } as i32;
            self.fan_start_temp = { v20.fan_start_temp } as i32;
            let mask = { v20.features_to_run };
            for bit in 0..64 {
                self.smu_features[bit] = (mask & (1u64 << bit)) != 0;
            }
            let tl = { v20.temp_limit };
            for i in 0..7 { self.temp_limits[i] = tl[i] as i32; }

            self.status_msg = format!("Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B]", label, chip_v.display_name(), frev, crev, trev, b.len());
            self.raw_buffer = b;
            return;
        }

        if let Some(pol) = crate::pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(&b) {
            let did = self.vbios_buffer.as_deref()
                .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id)
                .unwrap_or(0);
            let chip_p = crate::pptable_polaris::PolarisChip::from_did(did);
            self.oc_gfx_clock = [0, ({ pol.max_od_engine_clock } / 100) as i32];
            self.oc_mem_clock = [0, ({ pol.max_od_memory_clock } / 100) as i32];
            self.oc_power_limit = [-50, { pol.power_control_limit } as i32];
            self.oc_fan_rpm = [0, 0];
            self.oc_fan_acoustic_rpm = [0, 0];
            self.power_limit_w = 0;
            self.tdc_gfx = 0;
            self.tdc_soc = 0;
            for i in 0..4 {
                self.mem_dpm_freq[i] = [0, 1];
                self.mem_dpm_volt[i] = [0, 0];
            }
            if let Some((gfx_hi, mem_hi, vddc_hi)) = pol.read_hard_limits(&b) {
                self.volt_gfx = [0, vddc_hi];
                self.volt_soc = [0, 0];
                self.freq_gfx = [0, gfx_hi];
                self.freq_soc = [0, 0];
                self.mem_dpm_freq[3] = [mem_hi, 1];
                if let Some((vddci_hi, mvdd_hi)) = pol.read_hard_limit_mem_volt(&b) { self.mem_dpm_volt[3] = [vddci_hi, mvdd_hi]; }
            } else {
                self.volt_gfx = [0, 0];
                self.volt_soc = [0, 0];
                self.freq_gfx = [0, 0];
                self.freq_soc = [0, 0];
            }
            if let Some((pwm_min, max_rpm, tgt_temp, tgt_gfx, zrpm, stop_t, start_t)) = pol.read_fan_subtable(&b) {
                self.fan_pwm_min = pwm_min;
                self.fan_max_rpm = max_rpm;
                self.fan_target_temp = tgt_temp;
                self.fan_target_gfx_clock = tgt_gfx;
                self.fan_zero_rpm_enable = zrpm;
                self.fan_stop_temp = stop_t;
                self.fan_start_temp = start_t;
            }
            self.status_msg = format!("Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B]", label, chip_p.display_name(), frev, crev, trev, b.len());
            self.raw_buffer = b;
            return;
        }

        if let Some(v10) = crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes(&b) {
            let did = self.vbios_buffer.as_deref()
                .and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id)
                .unwrap_or(0);
            let chip_v = crate::pptable_vega::VegaChip::from_did_and_rev(did, frev);
            self.oc_gfx_clock = [0, ({ v10.max_od_engine_clock } / 100) as i32];
            self.oc_mem_clock = [0, ({ v10.max_od_memory_clock } / 100) as i32];
            self.oc_power_limit = [-50, { v10.power_control_limit } as i32];
            self.oc_fan_rpm = [0, 0];
            self.oc_fan_acoustic_rpm = [0, 0];
            self.power_limit_w = 0;
            self.tdc_gfx = 0;
            self.tdc_soc = 0;
            if let Some((soc_hi, gfx_hi, mem_hi, vddc_hi)) = v10.read_hard_limits(&b) {
                self.volt_gfx = [0, vddc_hi];
                self.volt_soc = [0, 0];
                self.freq_gfx = [0, gfx_hi];
                self.freq_soc = [0, soc_hi];
                self.mem_dpm_freq[3] = [mem_hi, 1];
                if let Some((vddci_hi, mvdd_hi)) = v10.read_hard_limit_mem_volt(&b) { self.mem_dpm_volt[3] = [vddci_hi, mvdd_hi]; }
            } else {
                self.volt_gfx = [0, 0];
                self.volt_soc = [0, 0];
            }
            self.ulv_gfx = [0, 0];
            self.ulv_soc = [0, 0];
            self.freq_gfx = [0, 0];
            self.freq_soc = [0, 0];
            self.freq_fclk = [0, 0];
            self.freq_vclk = [0, 0];
            self.freq_dclk = [0, 0];
            self.freq_dcefclk = [0, 0];
            self.freq_dispclk = [0, 0];
            self.freq_pixclk = [0, 0];
            self.freq_phyclk = [0, 0];
            for i in 0..4 {
                self.mem_dpm_freq[i] = [0, 1];
                self.mem_dpm_volt[i] = [0, 0];
            }
            if let Some((pwm_min, max_rpm, tgt_temp, tgt_gfx, zrpm, stop_t, start_t)) = v10.read_fan_subtable(&b) {
                self.fan_pwm_min = pwm_min;
                self.fan_acoustic_limit = 0;
                self.fan_throttling_rpm = 0;
                self.fan_max_rpm = max_rpm;
                self.fan_target_temp = tgt_temp;
                self.fan_target_gfx_clock = tgt_gfx;
                self.fan_zero_rpm_enable = zrpm;
                self.fan_stop_temp = stop_t;
                self.fan_start_temp = start_t;
            } else {
                self.fan_pwm_min = 0;
                self.fan_acoustic_limit = 0;
                self.fan_throttling_rpm = 0;
                self.fan_max_rpm = 0;
                self.fan_target_temp = 0;
                self.fan_target_gfx_clock = 0;
                self.fan_zero_rpm_enable = false;
                self.fan_stop_temp = 0;
                self.fan_start_temp = 0;
            }
            for i in 0..9 { self.temp_limits[i] = 0; }
            self.status_msg = format!("Loaded {} [{} | rev 0x{:02X}.{:02X}.{:02X} | {} B]", label, chip_v.display_name(), frev, crev, trev, b.len());
            self.raw_buffer = b;
            return;
        }

        let supported_rdna2 = matches!(frev, 0x10 | 0x11 | 0x12 | 0x13) || (frev == 0x0F && b.len() == 2470);
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
        for bit in 0..64 {
            self.smu_features[bit] = (mask & (1u64 << bit)) != 0;
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
        self.ulv_gfx = [t.ulv_voltage_vmin_gfx as i32 / 4, t.ulv_voltage_offset_gfx as i32 / 4];
        self.ulv_soc = [t.ulv_voltage_vmin_soc as i32 / 4, t.ulv_voltage_offset_soc as i32 / 4];
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
        self.raw_buffer = b;
    }

    pub fn sync_fields_to_buffer(&mut self) {
        if let Some(t3) = crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes_mut(&mut self.raw_buffer) {
            t3.od_feature_flags[0..12].copy_from_slice(&[
                self.gpu_clock_limit as u8, self.gpu_clock_curve as u8, self.memory_clock_limit as u8, self.zero_rpm as u8,
                self.min_fan_speed as u8, self.fan_target_temp_limit as u8, self.operating_temp_limit as u8, self.fan_acoustic_rpm as u8,
                self.fan_curve as u8, self.power_limit_feat as u8, self.timing_tuning as u8, self.power_mode as u8,
            ]);
            let mut mask = 0u64;
            for (bit, &en) in self.smu_features.iter().enumerate() {
                if en { mask |= 1u64 << bit; }
            }
            t3.features_to_run = mask;
            t3.od_gfx_max = self.oc_gfx_clock[1] as u32;
            t3.od_mem_max = self.oc_mem_clock[1] as u32;
            t3.od_pwr_max = self.oc_power_limit[1] as u32;
            t3.od_fan_max = self.oc_fan_rpm[1] as u32;
            t3.od_acoustic_max = self.oc_fan_acoustic_rpm[1] as u32;
            t3.socket_power_limit_ac = self.power_limit_w as u16;
            t3.min_voltage_gfx = (self.volt_gfx[0] * 4) as u16;
            t3.max_voltage_gfx = (self.volt_gfx[1] * 4) as u16;
            t3.min_voltage_soc = (self.volt_soc[0] * 4) as u16;
            t3.max_voltage_soc = (self.volt_soc[1] * 4) as u16;
            t3.freq_gfx_min = self.freq_gfx[0] as u16;
            t3.freq_gfx_max = self.freq_gfx[1] as u16;
            t3.freq_soc_max = self.freq_soc[1] as u16;
            t3.freq_fclk_min = self.freq_fclk[0] as u16;
            t3.freq_fclk_max = self.freq_fclk[1] as u16;
            t3.tdc_limit_gfx = self.tdc_gfx as u16;
            t3.tdc_limit_soc = self.tdc_soc as u16;
            for i in 0..4 {
                t3.freq_uclk[i] = self.mem_dpm_freq[i][0] as u16;
                t3.mem_vddci[i] = (self.mem_dpm_volt[i][0] * 4) as u16;
                t3.mem_mvdd[i] = (self.mem_dpm_volt[i][1] * 4) as u16;
            }
            t3.pcie_gen_speed[0] = self.pcie_gen_speed[0] as u8;
            t3.pcie_gen_speed[2] = self.pcie_gen_speed[1] as u8;
            t3.pcie_lane_count[0] = self.pcie_lane_count[0] as u8;
            t3.pcie_lane_count[2] = self.pcie_lane_count[1] as u8;
            t3.lclk_freq[0] = self.lclk_mhz[0] as u16;
            t3.lclk_freq[2] = self.lclk_mhz[1] as u16;
            t3.fan_throttling_rpm = self.fan_throttling_rpm as u16;
            t3.fan_pwm_min = self.fan_pwm_min as u16;
            t3.fan_acoustic_limit = self.fan_acoustic_limit as u16;
            t3.fan_max_rpm = self.fan_max_rpm as u16;
            t3.fan_target_gfx_clock = self.fan_target_gfx_clock as u16;
            let orig_tt = { t3.fan_target_temp };
            t3.fan_target_temp = if orig_tt > 200 { (self.fan_target_temp * 10) as u16 } else { self.fan_target_temp as u16 };
            t3.fan_zero_rpm_enable = self.fan_zero_rpm_enable as u8;
            t3.fan_stop_temp = self.fan_stop_temp as u16;
            t3.fan_start_temp = self.fan_start_temp as u16;
            for i in 0..9 {
                t3.temp_limit[i] = self.temp_limits[i] as u16;
            }
            return;
        }
        if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes_mut(&mut self.raw_buffer) {
            t1.boost_power_limit = self.power_limit_w.max(0) as u16;
            t1.socket_power_limit_ac[0] = self.power_limit_w.max(0) as u16;
            t1.tdc_limit_gfx = self.tdc_gfx.max(0) as u16;
            t1.tdc_limit_soc = self.tdc_soc.max(0) as u16;

            t1.min_voltage_gfx = (self.volt_gfx[0].max(0) * 4) as u16;
            t1.max_voltage_gfx = (self.volt_gfx[1].max(0) * 4) as u16;
            t1.min_voltage_soc = (self.volt_soc[0].max(0) * 4) as u16;
            t1.max_voltage_soc = (self.volt_soc[1].max(0) * 4) as u16;
            t1.min_voltage_ulv_gfx = (self.ulv_gfx[0].max(0) * 4) as u16;
            t1.min_voltage_ulv_soc = (self.ulv_soc[0].max(0) * 4) as u16;

            t1.od_settings_min[0] = self.oc_gfx_clock[0].max(0) as u32;
            t1.od_settings_max[0] = self.oc_gfx_clock[1].max(0) as u32;
            t1.od_settings_min[1] = self.oc_mem_clock[0].max(0) as u32;
            t1.od_settings_max[1] = self.oc_mem_clock[1].max(0) as u32;
            t1.od_settings_min[2] = self.oc_power_limit[0] as u32;
            t1.od_settings_max[2] = self.oc_power_limit[1].max(0) as u32;
            t1.od_settings_min[3] = self.oc_fan_rpm[0].max(0) as u32;
            t1.od_settings_max[3] = self.oc_fan_rpm[1].max(0) as u32;
            t1.od_settings_min[4] = self.oc_fan_acoustic_rpm[0].max(0) as u32;
            t1.od_settings_max[4] = self.oc_fan_acoustic_rpm[1].max(0) as u32;
            t1.od_settings_max[5] = self.fan_target_temp.max(0) as u32;

            let desc = { t1.dpm_descriptor };
            let gfx_n = (desc[0] as usize).clamp(1, 16);
            let soc_n = (desc[1] as usize).clamp(1, 8);
            let fclk_n = (desc[3] as usize).clamp(1, 8);
            let dclk_n = (desc[4] as usize).clamp(1, 8);
            let vclk_n = (desc[5] as usize).clamp(1, 8);
            let dcef_n = (desc[6] as usize).clamp(1, 8);
            let disp_n = (desc[7] as usize).clamp(1, 8);
            let pix_n = (desc[8] as usize).clamp(1, 8);
            let phy_n = (desc[9] as usize).clamp(1, 8);

            t1.freq_table_gfx[0] = self.freq_gfx[0].max(0) as u16;
            t1.freq_table_gfx[gfx_n - 1] = self.freq_gfx[1].max(0) as u16;
            t1.freq_table_socclk[0] = self.freq_soc[0].max(0) as u16;
            t1.freq_table_socclk[soc_n - 1] = self.freq_soc[1].max(0) as u16;
            t1.freq_table_fclk[0] = self.freq_fclk[0].max(0) as u16;
            t1.freq_table_fclk[fclk_n - 1] = self.freq_fclk[1].max(0) as u16;
            t1.freq_table_vclk[0] = self.freq_vclk[0].max(0) as u16;
            t1.freq_table_vclk[vclk_n - 1] = self.freq_vclk[1].max(0) as u16;
            t1.freq_table_dclk[0] = self.freq_dclk[0].max(0) as u16;
            t1.freq_table_dclk[dclk_n - 1] = self.freq_dclk[1].max(0) as u16;
            t1.freq_table_dcefclk[0] = self.freq_dcefclk[0].max(0) as u16;
            t1.freq_table_dcefclk[dcef_n - 1] = self.freq_dcefclk[1].max(0) as u16;
            t1.freq_table_dispclk[0] = self.freq_dispclk[0].max(0) as u16;
            t1.freq_table_dispclk[disp_n - 1] = self.freq_dispclk[1].max(0) as u16;
            t1.freq_table_pixclk[0] = self.freq_pixclk[0].max(0) as u16;
            t1.freq_table_pixclk[pix_n - 1] = self.freq_pixclk[1].max(0) as u16;
            t1.freq_table_phyclk[0] = self.freq_phyclk[0].max(0) as u16;
            t1.freq_table_phyclk[phy_n - 1] = self.freq_phyclk[1].max(0) as u16;

            for i in 0..4 {
                t1.freq_table_uclk[i] = self.mem_dpm_freq[i][0].max(0) as u16;
            }
            t1.vddci_max = (self.mem_dpm_volt[3][0].max(0) * 4) as u16;
            t1.mvdd_max = (self.mem_dpm_volt[3][1].max(0) * 4) as u16;

            t1.fan_pwm_min = self.fan_pwm_min.max(0) as u16;
            t1.fan_acoustic_limit_rpm = self.fan_acoustic_limit.max(0) as u16;
            t1.fan_throttling_rpm = self.fan_throttling_rpm.max(0) as u16;
            t1.fan_maximum_rpm = self.fan_max_rpm.max(0) as u16;
            t1.fan_target_temperature = self.fan_target_temp.max(0) as u16;
            t1.fan_target_gfxclk = self.fan_target_gfx_clock.max(0) as u16;
            t1.fan_zero_rpm_enable = u8::from(self.fan_zero_rpm_enable);
            t1.fan_stop_temp = self.fan_stop_temp.max(0) as u16;
            t1.fan_start_temp = self.fan_start_temp.max(0) as u16;

            for i in 0..7 {
                t1.temp_limit[i] = self.temp_limits[i].max(0) as u16;
            }

            let mut mask = 0u64;
            for bit in 0..64 {
                if self.smu_features[bit] {
                    mask |= 1u64 << bit;
                }
            }
            t1.features_to_run = mask;

            t1.od_feature_flags[0] = u8::from(self.gpu_clock_limit);
            t1.od_feature_flags[1] = u8::from(self.gpu_clock_curve);
            t1.od_feature_flags[2] = u8::from(self.memory_clock_limit);
            t1.od_feature_flags[3] = u8::from(self.zero_rpm);
            t1.od_feature_flags[4] = u8::from(self.min_fan_speed);
            t1.od_feature_flags[5] = u8::from(self.fan_target_temp_limit);
            t1.od_feature_flags[6] = u8::from(self.operating_temp_limit);
            t1.od_feature_flags[7] = u8::from(self.fan_acoustic_rpm);
            t1.od_feature_flags[8] = u8::from(self.fan_curve);
            t1.od_feature_flags[9] = u8::from(self.power_limit_feat);
            t1.od_feature_flags[10] = u8::from(self.timing_tuning);
            t1.od_feature_flags[11] = u8::from(self.power_mode);
            return;
        }

        if let Some(v20) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes_mut(&mut self.raw_buffer) {
            v20.boost_power_limit = self.power_limit_w.max(0) as u16;
            v20.socket_power_limit_ac[0] = self.power_limit_w.max(0) as u16;
            v20.tdc_limit_gfx = self.tdc_gfx.max(0) as u16;
            v20.tdc_limit_soc = self.tdc_soc.max(0) as u16;
            v20.min_voltage_gfx = (self.volt_gfx[0].max(0) * 4) as u16;
            v20.max_voltage_gfx = (self.volt_gfx[1].max(0) * 4) as u16;
            v20.min_voltage_soc = (self.volt_soc[0].max(0) * 4) as u16;
            v20.max_voltage_soc = (self.volt_soc[1].max(0) * 4) as u16;

            v20.od_settings_min[0] = self.oc_gfx_clock[0].max(0) as u32;
            v20.od_settings_max[0] = self.oc_gfx_clock[1].max(0) as u32;
            v20.od_settings_min[1] = self.oc_mem_clock[0].max(0) as u32;
            v20.od_settings_max[1] = self.oc_mem_clock[1].max(0) as u32;
            v20.od_settings_min[2] = self.oc_power_limit[0] as u32;
            v20.od_settings_max[2] = self.oc_power_limit[1].max(0) as u32;
            v20.od_settings_min[3] = self.oc_fan_rpm[0].max(0) as u32;
            v20.od_settings_max[3] = self.oc_fan_rpm[1].max(0) as u32;
            v20.od_settings_min[4] = self.oc_fan_acoustic_rpm[0].max(0) as u32;
            v20.od_settings_max[4] = self.oc_fan_acoustic_rpm[1].max(0) as u32;

            let desc = { v20.dpm_descriptor };
            let gfx_n = (desc[0] as usize).clamp(1, 16);
            let soc_n = (desc[1] as usize).clamp(1, 8);
            let fclk_n = (desc[3] as usize).clamp(1, 8);
            v20.freq_table_gfx[0] = self.freq_gfx[0].max(0) as u16;
            v20.freq_table_gfx[gfx_n - 1] = self.freq_gfx[1].max(0) as u16;
            v20.freq_table_socclk[0] = self.freq_soc[0].max(0) as u16;
            v20.freq_table_socclk[soc_n - 1] = self.freq_soc[1].max(0) as u16;
            v20.freq_table_fclk[0] = self.freq_fclk[0].max(0) as u16;
            v20.freq_table_fclk[fclk_n - 1] = self.freq_fclk[1].max(0) as u16;
            for i in 0..4 { v20.freq_table_uclk[i] = self.mem_dpm_freq[i][0].max(0) as u16; }
            v20.vddci_max = (self.mem_dpm_volt[3][0].max(0) * 4) as u16;
            v20.mvdd_max = (self.mem_dpm_volt[3][1].max(0) * 4) as u16;

            v20.fan_pwm_min = self.fan_pwm_min.max(0) as u16;
            v20.fan_acoustic_limit_rpm = self.fan_acoustic_limit.max(0) as u16;
            v20.fan_throttling_rpm = self.fan_throttling_rpm.max(0) as u16;
            v20.fan_maximum_rpm = self.fan_max_rpm.max(0) as u16;
            v20.fan_target_temperature = self.fan_target_temp.max(0) as u16;
            v20.fan_target_gfxclk = self.fan_target_gfx_clock.max(0) as u16;
            v20.min_voltage_ulv_gfx = (self.ulv_gfx[0].max(0) * 4) as u16;
            v20.min_voltage_ulv_soc = (self.ulv_soc[0].max(0) * 4) as u16;
            v20.fan_zero_rpm_enable = u8::from(self.fan_zero_rpm_enable);
            v20.fan_stop_temp = self.fan_stop_temp.max(0) as u16;
            v20.fan_start_temp = self.fan_start_temp.max(0) as u16;
            for i in 0..7 { v20.temp_limit[i] = self.temp_limits[i].max(0) as u16; }

            v20.features_to_run = Self::build_smu_mask(&self.smu_features);
            v20.od_feature_flags[0] = u8::from(self.gpu_clock_limit);
            v20.od_feature_flags[1] = u8::from(self.gpu_clock_curve);
            v20.od_feature_flags[2] = u8::from(self.memory_clock_limit);
            v20.od_feature_flags[3] = u8::from(self.zero_rpm);
            v20.od_feature_flags[4] = u8::from(self.min_fan_speed);
            v20.od_feature_flags[5] = u8::from(self.fan_target_temp_limit);
            v20.od_feature_flags[6] = u8::from(self.operating_temp_limit);
            v20.od_feature_flags[7] = u8::from(self.fan_acoustic_rpm);
            v20.od_feature_flags[8] = u8::from(self.fan_curve);
            v20.od_feature_flags[9] = u8::from(self.power_limit_feat);
            return;
        }

        if let Some(pol) = crate::pptable_polaris::AtomPolarisPowerPlayTable::from_bytes_mut(&mut self.raw_buffer) {
            pol.max_od_engine_clock = (self.oc_gfx_clock[1].max(0) * 100) as u32;
            pol.max_od_memory_clock = (self.oc_mem_clock[1].max(0) * 100) as u32;
            pol.power_control_limit = self.oc_power_limit[1].max(0) as u16;
            crate::pptable_polaris::AtomPolarisPowerPlayTable::write_fan_subtable(
                &mut self.raw_buffer,
                self.fan_pwm_min,
                self.fan_max_rpm,
                self.fan_target_temp,
                self.fan_target_gfx_clock,
                self.fan_zero_rpm_enable,
                self.fan_stop_temp,
                self.fan_start_temp,
            );
            crate::pptable_polaris::AtomPolarisPowerPlayTable::write_hard_limits_full(
                &mut self.raw_buffer,
                self.freq_gfx[1],
                self.mem_dpm_freq[3][0],
                self.volt_gfx[1],
                self.mem_dpm_volt[3][0],
                self.mem_dpm_volt[3][1],
            );
            return;
        }

        if let Some(v10) = crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes_mut(&mut self.raw_buffer) {
            v10.max_od_engine_clock = (self.oc_gfx_clock[1].max(0) * 100) as u32;
            v10.max_od_memory_clock = (self.oc_mem_clock[1].max(0) * 100) as u32;
            v10.power_control_limit = self.oc_power_limit[1].max(0) as u16;
            crate::pptable_vega::AtomVega10PowerPlayTable::write_hard_limits_full(
                &mut self.raw_buffer,
                self.freq_soc[1],
                self.freq_gfx[1],
                self.mem_dpm_freq[3][0],
                self.volt_gfx[1],
                self.mem_dpm_volt[3][0],
                self.mem_dpm_volt[3][1],
            );
            crate::pptable_vega::AtomVega10PowerPlayTable::write_fan_subtable(
                &mut self.raw_buffer,
                self.fan_pwm_min,
                self.fan_max_rpm,
                self.fan_target_temp,
                self.fan_target_gfx_clock,
                self.fan_zero_rpm_enable,
                self.fan_stop_temp,
                self.fan_start_temp,
            );
            return;
        }

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
        t.pm_rage_limit = self.pm_rage[0] as u32; t.pm_rage_temp = self.pm_rage[1] as u16;
        t.ac_quiet = [self.ac_quiet[0] as u16, self.ac_quiet[1] as u16];
        t.ac_balanced = [self.ac_balanced[0] as u16, self.ac_balanced[1] as u16];
        t.ac_turbo = [self.ac_turbo[0] as u16, self.ac_turbo[1] as u16];
        t.ac_rage = [self.ac_rage[0] as u16, self.ac_rage[1] as u16];

        t.min_voltage_gfx = (self.volt_gfx[0] * 4) as u16; t.max_voltage_gfx = (self.volt_gfx[1] * 4) as u16;
        t.min_voltage_soc = (self.volt_soc[0] * 4) as u16; t.max_voltage_soc = (self.volt_soc[1] * 4) as u16;
        let pwr = self.power_limit_w as u16;
        t.socket_power_limit_ac[0] = pwr;
        t.socket_power_limit_dc = pwr;
        t.tdc_limit_gfx = self.tdc_gfx as u16; t.tdc_limit_soc = self.tdc_soc as u16;
        t.ulv_voltage_vmin_gfx = (self.ulv_gfx[0] * 4) as u16; t.ulv_voltage_offset_gfx = (self.ulv_gfx[1] * 4) as u16;
        t.ulv_voltage_vmin_soc = (self.ulv_soc[0] * 4) as u16; t.ulv_voltage_offset_soc = (self.ulv_soc[1] * 4) as u16;
        t.tvmin_hyst_gfx = self.tvmin_gfx[0] as u16; t.tvmin_gfx = self.tvmin_gfx[1] as u16;
        t.tvmin_hyst_soc = self.tvmin_soc[0] as u16; t.tvmin_soc = self.tvmin_soc[1] as u16;
        t.vmin_low_gfx = (self.vmin_lh_gfx[0] * 4) as u16;
        t.vmin_high_gfx = (self.vmin_lh_gfx[1] * 4) as u16;
        t.vmin_low_soc = (self.vmin_lh_soc[0] * 4) as u16;
        t.vmin_high_soc = (self.vmin_lh_soc[1] * 4) as u16;
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
            .save_file()
        {
            self.sync_fields_to_buffer();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("bin").to_ascii_lowercase();
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
            let write_res = fs::write(&path, bytes);
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
