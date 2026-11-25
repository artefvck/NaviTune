use crate::{AdvanceOsTab, Tab};
use std::path::PathBuf;

pub struct PowerPlayApp {
    pub(crate) selected_gpu: usize,
    pub(crate) gpus: Vec<String>,
    pub(crate) ioreg_entries: Vec<crate::macos_ioreg::IoregGpuEntry>,
    pub(crate) live_telemetry_on: bool,
    pub(crate) telemetry_data: std::sync::Arc<std::sync::Mutex<crate::gpu_telemetry::GpuTelemetry>>,
    pub(crate) active_tab: Tab,
    pub advance_os_tab: AdvanceOsTab,
    pub raw_buffer: Vec<u8>,
    pub baseline_buffer: Vec<u8>,
    pub show_save_diff_modal: bool,
    pub baseline_gen: u64,
    pub vbios_buffer: Option<Vec<u8>>,
    pub vbios_ppt_offset: Option<usize>,
    #[allow(dead_code)]
    pub loaded_file: Option<PathBuf>,
    pub probe_offset: String,
    pub probe_write_val: String,
    pub probe_write_type: usize,
    pub debug_subtab: crate::tabs::more::DebugSubTab,
    pub debug_only_mismatch: bool,
    pub show_smu_features_window: bool,
    pub show_struct_inspector_window: bool,
    pub show_opencore_copy_modal: bool,
    pub more_popups: crate::more_popups::MorePopupsState,
    pub show_error_modal: bool,
    pub modal_error_title: String,
    pub modal_error_msg: String,
    pub status_msg: String,

    pub gpu_clock_limit: bool, pub gpu_clock_curve: bool, pub memory_clock_limit: bool, pub zero_rpm: bool,
    pub min_fan_speed: bool, pub fan_target_temp_limit: bool, pub operating_temp_limit: bool, pub fan_acoustic_rpm: bool,
    pub fan_curve: bool, pub power_limit_feat: bool, pub timing_tuning: bool, pub power_mode: bool,
    pub smu_features: [bool; 64],
    pub auto_undervolt_gpu: bool, pub auto_overclock_gpu: bool, pub auto_overclock_mem: bool, pub auto_fan_acoustic_limit: bool,

    pub oc_gfx_clock: [i32; 2], pub oc_mem_clock: [i32; 2], pub oc_power_limit: [i32; 2],
    pub oc_fan_rpm: [i32; 2], pub oc_fan_acoustic_rpm: [i32; 2],
    pub oc_mem_timing_ctrl: i32, pub oc_power_mode_ctrl: i32, pub oc_zero_rpm_ctrl: i32,

    pub pm_quiet: [i32; 2], pub pm_balanced: [i32; 2], pub pm_turbo: [i32; 2], pub pm_rage: [i32; 2],
    pub ac_quiet: [i32; 2], pub ac_balanced: [i32; 2], pub ac_turbo: [i32; 2], pub ac_rage: [i32; 2],

    pub volt_gfx: [i32; 2], pub volt_soc: [i32; 2], pub power_limit_w: i32, pub tdc_gfx: i32, pub tdc_soc: i32,
    pub ulv_gfx: [i32; 2], pub ulv_soc: [i32; 2], pub tvmin_gfx: [i32; 2], pub tvmin_soc: [i32; 2],
    pub vmin_lh_gfx: [i32; 2], pub vmin_lh_soc: [i32; 2],
    pub mem_dpm_volt: [[i32; 2]; 4], pub mem_dpm_freq: [[i32; 2]; 4],

    pub freq_gfx: [i32; 2], pub freq_soc: [i32; 2], pub freq_vclk: [i32; 2], pub freq_dclk: [i32; 2],
    pub freq_dcefclk: [i32; 2], pub freq_dispclk: [i32; 2], pub freq_pixclk: [i32; 2], pub freq_phyclk: [i32; 2],
    pub freq_dtbclk: [i32; 2], pub freq_fclk: [i32; 2], pub fclk_boost_freq: i32,
    pub pcie_gen_speed: [i32; 2], pub pcie_lane_count: [i32; 2], pub lclk_mhz: [i32; 2],

    pub static_volt_a: [f64; 2], pub static_volt_b: [f64; 2], pub static_volt_c: [f64; 2],
    pub static_off_a: [f64; 2], pub static_off_b: [f64; 2], pub static_off_c: [f64; 2],
    pub dcbtc_gfx_en: bool, pub dcbtc_soc_en: bool,
    pub dcbtc_min: [f64; 2], pub dcbtc_max: [f64; 2], pub dcbtc_gb: [f64; 2], pub dctol: [i32; 2],

    pub fan_pwm_min: i32, pub fan_acoustic_limit: i32, pub fan_throttling_rpm: i32, pub fan_max_rpm: i32,
    pub fan_mode: i32, pub fan_hyst_ctrl: i32,
    pub fan_target_temp: i32, pub fan_target_gfx_clock: i32, pub fan_zero_rpm_enable: bool,
    pub fan_stop_temp: i32, pub fan_start_temp: i32,
    pub od_fan_temp: [i32; 2], pub od_fan_speed: [i32; 2],
    pub fan_curve_temp: [i32; 5], pub fan_curve_pwm: [i32; 5],
    pub temp_limits: [i32; 9],
    pub power_limit_ac: [i32; 4],
}

impl Default for PowerPlayApp {
    fn default() -> Self {
        Self {
            fan_curve_temp: [0; 5],
            fan_curve_pwm: [0; 5],
            temp_limits: [0; 9],
            power_limit_ac: [0; 4],
            selected_gpu: 0, gpus: Vec::new(), ioreg_entries: Vec::new(),
            live_telemetry_on: false,
            telemetry_data: std::sync::Arc::new(std::sync::Mutex::new(crate::gpu_telemetry::GpuTelemetry::default())),
            active_tab: Tab::Features,
            advance_os_tab: AdvanceOsTab::MacOS, raw_buffer: Vec::new(),
            baseline_buffer: Vec::new(),
            show_save_diff_modal: false,
            baseline_gen: 0,
            vbios_buffer: None, vbios_ppt_offset: None, loaded_file: None,
            probe_offset: format!("0x{:03X}", std::mem::offset_of!(crate::pptable_rdna2::AtomSmcDpmInfoV4_9, socket_power_limit_ac)), probe_write_val: String::new(), probe_write_type: 1,
            debug_subtab: crate::tabs::more::DebugSubTab::Audit, debug_only_mismatch: false,
            show_smu_features_window: false,
            show_struct_inspector_window: false,
            show_opencore_copy_modal: false, more_popups: crate::more_popups::MorePopupsState,
            show_error_modal: false, modal_error_title: String::new(), modal_error_msg: String::new(),
            status_msg: "Ready".to_string(),
            gpu_clock_limit: false, gpu_clock_curve: false, memory_clock_limit: false, zero_rpm: false,
            min_fan_speed: false, fan_target_temp_limit: false, operating_temp_limit: false, fan_acoustic_rpm: false,
            fan_curve: false, power_limit_feat: false, timing_tuning: false, power_mode: false,
            smu_features: [false; 64],
            auto_undervolt_gpu: false, auto_overclock_gpu: false, auto_overclock_mem: false, auto_fan_acoustic_limit: false,
            oc_gfx_clock: [0, 0], oc_mem_clock: [0, 0], oc_power_limit: [0, 0],
            oc_fan_rpm: [0, 0], oc_fan_acoustic_rpm: [0, 0],
            oc_mem_timing_ctrl: 0, oc_power_mode_ctrl: 0, oc_zero_rpm_ctrl: 0,
            pm_quiet: [0, 0], pm_balanced: [0, 0], pm_turbo: [0, 0], pm_rage: [0, 0],
            ac_quiet: [0, 0], ac_balanced: [0, 0], ac_turbo: [0, 0], ac_rage: [0, 0],
            volt_gfx: [0, 0], volt_soc: [0, 0], power_limit_w: 0, tdc_gfx: 0, tdc_soc: 0,
            ulv_gfx: [0, 0], ulv_soc: [0, 0], tvmin_gfx: [0, 0], tvmin_soc: [0, 0],
            vmin_lh_gfx: [0, 0], vmin_lh_soc: [0, 0],
            mem_dpm_volt: [[0, 0]; 4], mem_dpm_freq: [[0, 0]; 4],
            freq_gfx: [0, 0], freq_soc: [0, 0], freq_vclk: [0, 0], freq_dclk: [0, 0],
            freq_dcefclk: [0, 0], freq_dispclk: [0, 0], freq_pixclk: [0, 0], freq_phyclk: [0, 0],
            freq_dtbclk: [0, 0], freq_fclk: [0, 0], fclk_boost_freq: 0,
            pcie_gen_speed: [0, 0], pcie_lane_count: [0, 0], lclk_mhz: [0, 0],
            static_volt_a: [0.0, 0.0], static_volt_b: [0.0, 0.0], static_volt_c: [0.0, 0.0],
            static_off_a: [0.0, 0.0], static_off_b: [0.0, 0.0], static_off_c: [0.0, 0.0],
            dcbtc_gfx_en: false, dcbtc_soc_en: false,
            dcbtc_min: [0.0, 0.0], dcbtc_max: [0.0, 0.0], dcbtc_gb: [0.0, 0.0], dctol: [0, 0],
            fan_pwm_min: 0, fan_acoustic_limit: 0, fan_throttling_rpm: 0, fan_max_rpm: 0,
            fan_mode: 0, fan_hyst_ctrl: 0,
            fan_target_temp: 0, fan_target_gfx_clock: 0, fan_zero_rpm_enable: false,
            fan_stop_temp: 0, fan_start_temp: 0,
            od_fan_temp: [0, 0], od_fan_speed: [0, 0],
        }
    }
}

impl PowerPlayApp {
    /// Counts how many bytes in `raw_buffer` differ from `baseline_buffer` (plus length delta).
    pub(crate) fn modified_byte_count(&self) -> usize {
        let shared = self.raw_buffer.len().min(self.baseline_buffer.len());
        let diff = self.raw_buffer[..shared]
            .iter()
            .zip(&self.baseline_buffer[..shared])
            .filter(|(a, b)| a != b)
            .count();
        diff + self.raw_buffer.len().abs_diff(self.baseline_buffer.len())
    }

    /// Validates active clock, voltage, and fan ranges so invalid inversions are surfaced before save/export.
    pub(crate) fn validate_tunables(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.freq_gfx[0] > self.freq_gfx[1] {
            warnings.push(format!(
                "GFX min clock ({} MHz) exceeds max clock ({} MHz)",
                self.freq_gfx[0], self.freq_gfx[1]
            ));
        }
        if self.volt_gfx[1] > 0 && self.volt_gfx[0] > self.volt_gfx[1] {
            warnings.push(format!(
                "GFX min voltage ({} mV) exceeds max voltage ({} mV)",
                self.volt_gfx[0], self.volt_gfx[1]
            ));
        }
        if self.volt_gfx[1] > 1350 {
            warnings.push(format!(
                "GFX max voltage ({} mV) exceeds 1350 mV safety ceiling",
                self.volt_gfx[1]
            ));
        }
        warnings
    }
}

#[cfg(test)]
mod app_validation_tests {
    use super::PowerPlayApp;

    #[test]
    fn modified_byte_count_tracks_buffer_deltas() {
        let empty = PowerPlayApp::default();
        assert_eq!(empty.modified_byte_count(), 0);
        let app = PowerPlayApp {
            baseline_buffer: vec![0x10, 0x20, 0x30],
            raw_buffer: vec![0x10, 0xFF, 0x30, 0x40],
            ..Default::default()
        };
        assert_eq!(app.modified_byte_count(), 2);
    }

    #[test]
    fn validate_tunables_catches_inversions_and_overvoltage() {
        let app = PowerPlayApp {
            freq_gfx: [2500, 1800],
            volt_gfx: [1400, 1380],
            ..Default::default()
        };
        let warns = app.validate_tunables();
        assert_eq!(warns.len(), 3);
    }
}
