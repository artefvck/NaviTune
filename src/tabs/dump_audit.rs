use crate::PowerPlayApp;

impl PowerPlayApp {
    pub(crate) fn dump_audit_to_debug_log(&mut self) {
        self.sync_fields_to_buffer();
        let mut dump = format!(
            "=== FULL PPTABLE SNAPSHOT ({} bytes, source={}) ===\n",
            self.raw_buffer.len(),
            self.status_msg
        );
        dump.push_str(&format!(
            "GFX Clock: {:?} MHz | MEM Clock: {:?} MHz | Power: {} W | TDC GFX/Soc: {}/{} A\n",
            self.freq_gfx, self.freq_soc, self.power_limit_w, self.tdc_gfx, self.tdc_soc
        ));
        dump.push_str(&format!(
            "Volt GFX: {:?} mV | Volt Soc: {:?} mV | TVmin GFX/Soc: {:?}/{:?} | Vmin LH GFX/Soc: {:?}/{:?}\n",
            self.volt_gfx, self.volt_soc, self.tvmin_gfx, self.tvmin_soc, self.vmin_lh_gfx, self.vmin_lh_soc
        ));
        for (i, chunk) in self.raw_buffer.chunks(16).enumerate() {
            let hex: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
            dump.push_str(&format!("0x{:04X}: {}\n", i * 16, hex.join(" ")));
        }
        crate::ui_theme::log_debug("SNAPSHOT", &dump);
        if let Some(lp) = crate::ui_theme::debug_log_path() {
            self.status_msg = format!("Dumped full audit & hex snapshot to {}", lp.display());
        }
    }

    pub(crate) fn build_full_debug_dump(&self) -> String {
        let mut smu_mask = 0u64;
        for (bit, &en) in self.smu_features.iter().enumerate() {
            if en { smu_mask |= 1u64 << bit; }
        }
        let _enabled_bits: Vec<String> = (0..64)
            .filter(|&b| self.smu_features[b])
            .map(|b| format!("{:02}", b))
            .collect();
        let gpu_desc = format!("[{}]", self.selected_gpu);

        let mut vars: Vec<(String, String)> = vec![
            ("vid_curve_gfx_a".into(), format!("{:.6}", self.static_volt_a[0])),
            ("vid_curve_gfx_b".into(), format!("{:.6}", self.static_volt_b[0])),
            ("vid_curve_gfx_c".into(), format!("{:.6}", self.static_volt_c[0])),
            ("vid_curve_soc_a".into(), format!("{:.6}", self.static_volt_a[1])),
            ("vid_curve_soc_b".into(), format!("{:.6}", self.static_volt_b[1])),
            ("vid_curve_soc_c".into(), format!("{:.6}", self.static_volt_c[1])),
            ("curve_gfx_a".into(), format!("{:.6}", self.static_off_a[0])),
            ("curve_gfx_b".into(), format!("{:.6}", self.static_off_b[0])),
            ("curve_gfx_c".into(), format!("{:.6}", self.static_off_c[0])),
            ("curve_soc_a".into(), format!("{:.6}", self.static_off_a[1])),
            ("curve_soc_b".into(), format!("{:.6}", self.static_off_b[1])),
            ("curve_soc_c".into(), format!("{:.6}", self.static_off_c[1])),
            ("dctol_gfx".into(), format!("{}", self.dctol[0])),
            ("dctol_soc".into(), format!("{}", self.dctol[1])),
            ("dcbtc_en_gfx".into(), format!("{}", self.dcbtc_gfx_en as u8)),
            ("dcbtc_en_soc".into(), format!("{}", self.dcbtc_soc_en as u8)),
            ("fan_pwm_min".into(), format!("{}", self.fan_pwm_min)),
            ("fan_acoustic_lim".into(), format!("{}", self.fan_acoustic_limit)),
            ("fan_throttling".into(), format!("{}", self.fan_throttling_rpm)),
            ("fan_max_rpm".into(), format!("{}", self.fan_max_rpm)),
            ("fan_target_temp".into(), format!("{}", self.fan_target_temp)),
            ("fan_target_gfx".into(), format!("{}", self.fan_target_gfx_clock)),
            ("fan_zero_rpm".into(), format!("{}", self.fan_zero_rpm_enable as u8)),
            ("fan_stop_temp".into(), format!("{}", self.fan_stop_temp)),
            ("fan_start_temp".into(), format!("{}", self.fan_start_temp)),
            ("freq_gfx_min".into(), format!("{}", self.freq_gfx[0])),
            ("freq_gfx_max".into(), format!("{}", self.freq_gfx[1])),
            ("freq_soc_min".into(), format!("{}", self.freq_soc[0])),
            ("freq_soc_max".into(), format!("{}", self.freq_soc[1])),
            ("freq_fclk_min".into(), format!("{}", self.freq_fclk[0])),
            ("freq_fclk_max".into(), format!("{}", self.freq_fclk[1])),
            ("lclk_min".into(), format!("{}", self.lclk_mhz[0])),
            ("lclk_max".into(), format!("{}", self.lclk_mhz[1])),
            ("pcie_gen_min".into(), format!("{}", self.pcie_gen_speed[0])),
            ("pcie_gen_max".into(), format!("{}", self.pcie_gen_speed[1])),
            ("pcie_lane_min".into(), format!("{}", self.pcie_lane_count[0])),
            ("pcie_lane_max".into(), format!("{}", self.pcie_lane_count[1])),
            ("power_limit_gpu".into(), format!("{}", self.power_limit_w)),
            ("tdc_gfx".into(), format!("{}", self.tdc_gfx)),
            ("tdc_soc".into(), format!("{}", self.tdc_soc)),
            ("vmin_gfx".into(), format!("{}", self.volt_gfx[0])),
            ("vmax_gfx".into(), format!("{}", self.volt_gfx[1])),
            ("vmin_soc".into(), format!("{}", self.volt_soc[0])),
            ("vmax_soc".into(), format!("{}", self.volt_soc[1])),
            ("smu_features_mask".into(), format!("0x{:016X}", smu_mask)),
        ];
        for i in 0..5 {
            vars.push((format!("fan_curve_pwm_{}", i), format!("{}", self.fan_curve_pwm[i])));
            vars.push((format!("fan_curve_temp_{}", i), format!("{}", self.fan_curve_temp[i])));
        }
        for i in 0..4 {
            vars.push((format!("mem_div{}", i), format!("{}", self.mem_dpm_freq[i][1])));
            vars.push((format!("mem_dpm{}", i), format!("{}", self.mem_dpm_freq[i][0])));
            vars.push((format!("vddci_dpm{}", i), format!("{}", self.mem_dpm_volt[i][0])));
            vars.push((format!("mvdd_dpm{}", i), format!("{}", self.mem_dpm_volt[i][1])));
            vars.push((format!("power_limit_ac_{}", i), format!("{}", self.power_limit_ac[i])));
        }
        for (i, limit) in self.temp_limits.iter().enumerate().take(9) {
            vars.push((format!("temp_limit_{}", i), format!("{}", limit)));
        }
        vars.push(("gfx_dpm_range".to_string(), crate::ui::cards::format_dpm_range(&self.freq_gfx)));
        vars.push(("modified_bytes".to_string(), self.modified_byte_count().to_string()));
        vars.push(("validation_warnings".to_string(), self.validate_tunables().len().to_string()));
        let (_, diff_preview_total) = crate::ui::save_diff_modal::collect_bounded_byte_diffs(&self.baseline_buffer, &self.raw_buffer, 64);
        vars.push(("diff_preview_total".to_string(), diff_preview_total.to_string()));
        let gfx_span_norm = crate::ui::curve_plot::normalize_curve_axis(
            self.freq_gfx[1] as f32,
            self.freq_gfx[0] as f32,
            3500.0,
        );
        vars.push(("gfx_span_norm".to_string(), format!("{:.3}", gfx_span_norm)));
        vars.sort_by(|a, b| a.0.cmp(&b.0));

        let mut out = String::new();
        out.push_str("=== NaviTune FULL SYSTEM & TABLE DEBUG DUMP ===\n");
        out.push_str(&format!("OS / Arch        : {} / {}\n", std::env::consts::OS, std::env::consts::ARCH));
        out.push_str(&format!("Selected GPU     : {}\n", gpu_desc));
        out.push_str(&format!("Telemetry Active : {}\n", self.telemetry_data.lock().is_ok()));
        let did = self.vbios_buffer.as_deref().and_then(|vb| crate::vbios::inspect_vbios_info(vb, self.vbios_ppt_offset.unwrap_or(0))).map(|vi| vi.device_id).unwrap_or(0);
        let chip_name = if let Some(t3) = crate::pptable_rdna3::AtomSmcDpmInfoV4_12::from_bytes(&self.raw_buffer) {
            t3.detect_chip(did).display_name()
        } else if matches!(crate::ppt_arch::PptArch::detect(&self.raw_buffer), crate::ppt_arch::PptArch::Vega10 | crate::ppt_arch::PptArch::Vega20) {
            let frev_v = self.raw_buffer.get(2).copied().unwrap_or(0);
            crate::pptable_vega::VegaChip::from_did_and_rev(did, frev_v).display_name()
        } else if let Some(t1) = crate::pptable_rdna1::AtomSmcDpmInfoV4_5::from_bytes(&self.raw_buffer) {
            t1.detect_chip(did).display_name()
        } else {
            crate::pptable_rdna2::AtomSmcDpmInfoV4_9::from_bytes(&self.raw_buffer).map(|t| t.detect_chip(&self.raw_buffer).display_name()).unwrap_or(if crate::ppt_arch::PptArch::detect(&self.raw_buffer) == crate::ppt_arch::PptArch::Polaris { "Polaris (SMU 8.0, rev 0x07)" } else { "Unknown" })
        };
        out.push_str(&format!("Detected Silicon : {}\n", chip_name));
        let arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        out.push_str(&format!("Table Struct     : {} ({}, default {} B)\n", arch.table_struct_name(), arch.smu_generation(), arch.default_table_size()));
        out.push_str(&format!("Encoded Size     : {} bytes\n", self.raw_buffer.len()));
        if let Some(v20) = crate::pptable_vega::AtomVega20PowerPlayTable::from_bytes(&self.raw_buffer) {
            out.push_str(&format!("Vega 20 Offsets  : PPT0={}W (0x11D), TDC={}/{}A (0x13D), GFX_V={}..{}mV (0x159)\n", {v20.socket_power_limit_ac[0]}, {v20.tdc_limit_gfx}, {v20.tdc_limit_soc}, {v20.min_voltage_gfx}/4, {v20.max_voltage_gfx}/4));
        } else if let Some(pol) = crate::pptable_polaris::AtomPolarisPowerPlayTable::from_bytes(&self.raw_buffer) {
            out.push_str(&format!("Polaris Offsets  : OD_GFX={}MHz (0x017), OD_MEM={}MHz (0x01B), FanOff=0x{:03X}, HardLimOff=0x{:03X}\n", {pol.max_od_engine_clock}/100, {pol.max_od_memory_clock}/100, {pol.fan_table_offset}, {pol.hard_limit_table_offset}));
        } else if let Some(v10) = crate::pptable_vega::AtomVega10PowerPlayTable::from_bytes(&self.raw_buffer) {
            out.push_str(&format!("Vega 10 Offsets  : OD_GFX={}MHz (0x017), OD_MEM={}MHz (0x01B), FanOff=0x{:03X}, HardLimOff=0x{:03X}\n", {v10.max_od_engine_clock}/100, {v10.max_od_memory_clock}/100, {v10.fan_table_offset}, {v10.hard_limit_table_offset}));
        }
        let feat_mask: u64 = if arch.is_gcn_subtable() && self.raw_buffer.len() >= 0x17 {
            u32::from_le_bytes([self.raw_buffer[0x13], self.raw_buffer[0x14], self.raw_buffer[0x15], self.raw_buffer[0x16]]) as u64
        } else {
            (0..64usize).fold(0u64, |acc, bit| if self.smu_features[bit] { acc | (1u64 << bit) } else { acc })
        };
        let decoded_feats: Vec<String> = arch.decode_feature_bits(feat_mask).into_iter().map(|(bit, name)| format!("{}:{}", bit, name)).collect();
        if arch.is_gcn_subtable() { out.push_str(&format!("PlatformCaps     : 0x{:08X} ({})\n", feat_mask as u32, decoded_feats.join(", "))); } else { out.push_str(&format!("FeaturesToRun    : 0x{:016X} ({})\n", feat_mask, decoded_feats.join(", "))); }
        out.push_str(&format!("Decoded Vars     : {} entries\n", vars.len()));
        for (k, v) in vars {
            out.push_str(&format!("  {} = {}\n", k, v));
        }
        out
    }

}
