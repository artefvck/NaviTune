use crate::{AdvanceOsTab, Tab};
use std::path::PathBuf;

pub struct PowerPlayApp {
    pub(crate) selected_gpu: usize,
    pub(crate) gpus: Vec<String>,
    pub(crate) ioreg_entries: Vec<crate::macos_ioreg::IoregGpuEntry>,
    pub(crate) live_telemetry_on: bool,
    pub(crate) telemetry_stop: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
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
            telemetry_stop: None,
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

    /// Menyalin hanya nilai tuning dari profil donor tanpa menimpa struktur header biner asli.
    pub fn apply_tuning_values_only(&mut self, donor_bytes: &[u8]) -> Result<String, String> {
        if self.raw_buffer.is_empty() {
            return Err("Load your GPU's native VBIOS (.rom) or select GPU Path first.".into());
        }
        let target_arch = crate::ppt_arch::PptArch::detect(&self.raw_buffer);
        if target_arch == crate::ppt_arch::PptArch::Unknown {
            return Err("Active buffer is not a valid AMD PowerPlayTable.".into());
        }
        let mut donor = Self::default();
        donor.load_bytes_for_path(std::path::Path::new("donor.bin"), donor_bytes.to_vec(), "donor")?;
        let donor_arch = crate::ppt_arch::PptArch::detect(&donor.raw_buffer);
        if donor.raw_buffer.is_empty() || donor_arch == crate::ppt_arch::PptArch::Unknown {
            return Err("Donor profile does not contain a valid AMD PowerPlayTable.".into());
        }
        if donor_arch != target_arch {
            return Err(format!("Architecture mismatch: active {:?} vs donor {:?}.", target_arch, donor_arch));
        }

        self.gpu_clock_limit = donor.gpu_clock_limit;
        self.gpu_clock_curve = donor.gpu_clock_curve;
        self.memory_clock_limit = donor.memory_clock_limit;
        self.zero_rpm = donor.zero_rpm;
        self.min_fan_speed = donor.min_fan_speed;
        self.fan_target_temp_limit = donor.fan_target_temp_limit;
        self.operating_temp_limit = donor.operating_temp_limit;
        self.fan_acoustic_rpm = donor.fan_acoustic_rpm;
        self.fan_curve = donor.fan_curve;
        self.power_limit_feat = donor.power_limit_feat;
        self.timing_tuning = donor.timing_tuning;
        self.power_mode = donor.power_mode;
        self.smu_features = donor.smu_features;
        self.auto_undervolt_gpu = donor.auto_undervolt_gpu;
        self.auto_overclock_gpu = donor.auto_overclock_gpu;
        self.auto_overclock_mem = donor.auto_overclock_mem;
        self.auto_fan_acoustic_limit = donor.auto_fan_acoustic_limit;

        self.oc_gfx_clock = donor.oc_gfx_clock;
        self.oc_mem_clock = donor.oc_mem_clock;
        self.oc_power_limit = donor.oc_power_limit;
        self.oc_fan_rpm = donor.oc_fan_rpm;
        self.oc_fan_acoustic_rpm = donor.oc_fan_acoustic_rpm;
        self.oc_mem_timing_ctrl = donor.oc_mem_timing_ctrl;
        self.oc_power_mode_ctrl = donor.oc_power_mode_ctrl;
        self.oc_zero_rpm_ctrl = donor.oc_zero_rpm_ctrl;

        self.pm_quiet = donor.pm_quiet;
        self.pm_balanced = donor.pm_balanced;
        self.pm_turbo = donor.pm_turbo;
        self.pm_rage = donor.pm_rage;
        self.ac_quiet = donor.ac_quiet;
        self.ac_balanced = donor.ac_balanced;
        self.ac_turbo = donor.ac_turbo;
        self.ac_rage = donor.ac_rage;

        self.volt_gfx = donor.volt_gfx;
        self.volt_soc = donor.volt_soc;
        self.power_limit_w = donor.power_limit_w;
        self.tdc_gfx = donor.tdc_gfx;
        self.tdc_soc = donor.tdc_soc;
        self.ulv_gfx = donor.ulv_gfx;
        self.ulv_soc = donor.ulv_soc;
        self.tvmin_gfx = donor.tvmin_gfx;
        self.tvmin_soc = donor.tvmin_soc;
        self.vmin_lh_gfx = donor.vmin_lh_gfx;
        self.vmin_lh_soc = donor.vmin_lh_soc;
        self.mem_dpm_volt = donor.mem_dpm_volt;
        self.mem_dpm_freq = donor.mem_dpm_freq;

        self.freq_gfx = donor.freq_gfx;
        self.freq_soc = donor.freq_soc;
        self.freq_vclk = donor.freq_vclk;
        self.freq_dclk = donor.freq_dclk;
        self.freq_dcefclk = donor.freq_dcefclk;
        self.freq_dispclk = donor.freq_dispclk;
        self.freq_pixclk = donor.freq_pixclk;
        self.freq_phyclk = donor.freq_phyclk;
        self.freq_dtbclk = donor.freq_dtbclk;
        self.freq_fclk = donor.freq_fclk;
        self.fclk_boost_freq = donor.fclk_boost_freq;
        self.pcie_gen_speed = donor.pcie_gen_speed;
        self.pcie_lane_count = donor.pcie_lane_count;
        self.lclk_mhz = donor.lclk_mhz;

        self.static_volt_a = donor.static_volt_a;
        self.static_volt_b = donor.static_volt_b;
        self.static_volt_c = donor.static_volt_c;
        self.static_off_a = donor.static_off_a;
        self.static_off_b = donor.static_off_b;
        self.static_off_c = donor.static_off_c;
        self.dcbtc_gfx_en = donor.dcbtc_gfx_en;
        self.dcbtc_soc_en = donor.dcbtc_soc_en;
        self.dcbtc_min = donor.dcbtc_min;
        self.dcbtc_max = donor.dcbtc_max;
        self.dcbtc_gb = donor.dcbtc_gb;
        self.dctol = donor.dctol;

        self.fan_pwm_min = donor.fan_pwm_min;
        self.fan_acoustic_limit = donor.fan_acoustic_limit;
        self.fan_throttling_rpm = donor.fan_throttling_rpm;
        self.fan_max_rpm = donor.fan_max_rpm;
        self.fan_mode = donor.fan_mode;
        self.fan_hyst_ctrl = donor.fan_hyst_ctrl;
        self.fan_target_temp = donor.fan_target_temp;
        self.fan_target_gfx_clock = donor.fan_target_gfx_clock;
        self.fan_zero_rpm_enable = donor.fan_zero_rpm_enable;
        self.fan_stop_temp = donor.fan_stop_temp;
        self.fan_start_temp = donor.fan_start_temp;
        self.od_fan_temp = donor.od_fan_temp;
        self.od_fan_speed = donor.od_fan_speed;
        self.fan_curve_temp = donor.fan_curve_temp;
        self.fan_curve_pwm = donor.fan_curve_pwm;
        self.temp_limits = donor.temp_limits;
        self.power_limit_ac = donor.power_limit_ac;

        self.sync_fields_to_buffer();
        Ok(format!(
            "Applied tuning parameters onto native {:?} table ({} B) — binary structure preserved",
            target_arch,
            self.raw_buffer.len()
        ))
    }

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
    pub(crate) fn is_gfx_voltage_valid(&self) -> bool {
        let arch = crate::arch::detect::PptArch::detect(&self.raw_buffer);
        let (vg0, vg1) = (self.volt_gfx[0], self.volt_gfx[1]);
        if arch.is_rdna3() {
            vg0 > 0 && (vg1 == 0 || (vg0 <= vg1 && vg1 <= 1250))
        } else {
            vg0 > 0 && vg0 <= vg1 && vg1 <= 1250
        }
    }

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
        if self.volt_soc[1] > 0 && self.volt_soc[0] > self.volt_soc[1] {
            warnings.push(format!(
                "Soc min voltage ({} mV) exceeds max voltage ({} mV)",
                self.volt_soc[0], self.volt_soc[1]
            ));
        }
        if self.oc_gfx_clock[0] > self.oc_gfx_clock[1] || self.oc_mem_clock[0] > self.oc_mem_clock[1] || self.oc_power_limit[0] > self.oc_power_limit[1] {
            warnings.push("OverDrive min limit exceeds max limit".to_string());
        }
        if self.temp_limits.iter().any(|&t| !(0..=130).contains(&t)) {
            warnings.push("Thermal limit out of 0..130 °C range".to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_gfx_voltage_valid_avfs_rdna3_vs_legacy() {
        let mut app_state = PowerPlayApp::default();

        // RDNA 3 (V4_12) AVFS: Vmin > 0 and Vmax == 0 must be valid
        let mut r3 = vec![0u8; 5424];
        r3[0..2].copy_from_slice(&5424u16.to_le_bytes());
        r3[2] = 0x15;
        app_state.raw_buffer = r3;
        app_state.volt_gfx = [725, 0];
        assert!(app_state.is_gfx_voltage_valid(), "RDNA 3 AVFS (Vmax=0) must be valid");

        app_state.volt_gfx = [0, 0];
        assert!(!app_state.is_gfx_voltage_valid(), "RDNA 3 with Vmin=0 must be invalid");

        // RDNA 2 (V4_9): Vmax == 0 must be invalid; 750..1150 mV must be valid
        let mut r2 = vec![0u8; 2470];
        r2[0..2].copy_from_slice(&2470u16.to_le_bytes());
        r2[2] = 0x0F;
        app_state.raw_buffer = r2;
        app_state.volt_gfx = [750, 0];
        assert!(!app_state.is_gfx_voltage_valid(), "RDNA 2 with Vmax=0 must be invalid");

        app_state.volt_gfx = [750, 1150];
        assert!(app_state.is_gfx_voltage_valid(), "RDNA 2 750..1150 mV must be valid");

        app_state.volt_gfx = [750, 1300];
        assert!(!app_state.is_gfx_voltage_valid(), "Voltage > 1250 mV must fail check");
    }
}
