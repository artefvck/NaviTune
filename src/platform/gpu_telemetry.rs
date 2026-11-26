use std::process::Command;

#[derive(Clone, Debug, Default)]
pub struct GpuTelemetry {
    pub valid: bool,
    pub gfx_mhz: i64,
    pub mem_mhz: i64,
    pub temp_c: i64,
    pub hotspot_c: i64,
    pub load_pct: i64,
    pub power_w: i64,
    pub fan_rpm: i64,
    pub fan_pct: i64,
    pub vram_used_mb: i64,
    pub vram_total_mb: i64,
}

#[cfg(target_os = "macos")]
fn extract_ioreg_num(txt: &str, key: &str) -> Option<i64> {
    let quoted = format!("\"{}\"", key);
    let mut search_from = 0usize;
    while let Some(rel) = txt[search_from..].find(&quoted) {
        let after_key = search_from + rel + quoted.len();
        let trimmed = txt[after_key..].trim_start();
        if let Some(after_eq) = trimmed.strip_prefix('=') {
            let rest = after_eq.trim_start();
            let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(rest.len());
            if end > 0 {
                if let Ok(v) = rest[..end].parse::<i64>() {
                    return Some(v);
                }
            }
        }
        search_from = after_key;
    }
    None
}

#[cfg(target_os = "macos")]
fn normalize_clock_mhz(raw: i64) -> i64 {
    if raw > 100_000 { raw / 1_000_000 } else if raw > 10_000 { raw / 100 } else { raw }
}

#[cfg(target_os = "macos")]
fn parse_macos_accelerator_dump(txt: &str) -> GpuTelemetry {
    let mut best_amd: Option<GpuTelemetry> = None;
    let mut best_any: Option<GpuTelemetry> = None;

    for block in txt.split("+-o ").skip(1) {
        let header = block.lines().next().unwrap_or("");
        let is_amd = header.contains("AMD") || header.contains("ATY") || block.contains("AMDRadeon");
        let slice = if let Some(p) = block.find("\"PerformanceStatistics\"") { &block[p..] } else { block };
        let gfx = normalize_clock_mhz(extract_ioreg_num(slice, "Core Clock(MHz)").unwrap_or(0));
        let mem = normalize_clock_mhz(extract_ioreg_num(slice, "Memory Clock(MHz)").unwrap_or(0));
        let temp = extract_ioreg_num(slice, "Temperature(C)").unwrap_or(0);
        let hot = extract_ioreg_num(slice, "Hotspot Temperature(C)").unwrap_or(temp);
        let load = extract_ioreg_num(slice, "GPU Activity(%)")
            .or_else(|| extract_ioreg_num(slice, "Device Utilization %"))
            .unwrap_or(0);
        let pwr = extract_ioreg_num(slice, "Total Power(W)").unwrap_or(0);
        let rpm = extract_ioreg_num(slice, "Fan Speed(RPM)").unwrap_or(0);
        let pct = extract_ioreg_num(slice, "Fan Speed(%)").unwrap_or(0);
        let vram_used = extract_ioreg_num(slice, "inUseVidMemoryBytes")
            .or_else(|| extract_ioreg_num(slice, "vramUsedBytes"))
            .unwrap_or(0) / (1024 * 1024);
        let vram_free = extract_ioreg_num(slice, "vramFreeBytes").unwrap_or(0) / (1024 * 1024);
        let vram_total = extract_ioreg_num(block, "VRAM,totalMB")
            .or_else(|| extract_ioreg_num(txt, "VRAM,totalMB"))
            .unwrap_or_else(|| if vram_free > 0 { vram_used + vram_free } else { 0 });
        let cand = GpuTelemetry {
            valid: gfx > 0 || mem > 0 || temp > 0 || load > 0 || vram_total > 0,
            gfx_mhz: gfx, mem_mhz: mem, temp_c: temp, hotspot_c: hot,
            load_pct: load, power_w: pwr, fan_rpm: rpm, fan_pct: pct,
            vram_used_mb: vram_used, vram_total_mb: vram_total,
        };
        if !cand.valid { continue; }
        if is_amd && best_amd.as_ref().is_none_or(|b| cand.gfx_mhz > b.gfx_mhz || cand.temp_c > b.temp_c) {
            best_amd = Some(cand.clone());
        }
        if best_any.as_ref().is_none_or(|b| cand.gfx_mhz > b.gfx_mhz || cand.temp_c > b.temp_c) {
            best_any = Some(cand);
        }
    }
    best_amd.or(best_any).unwrap_or_default()
}

#[cfg(target_os = "macos")]
pub fn poll_gpu_telemetry() -> GpuTelemetry {
    let Ok(out) = crate::platform::run_cmd_bounded(Command::new("ioreg").args(["-r", "-c", "IOAccelerator", "-l"]), 1500) else {
        return GpuTelemetry::default();
    };
    let txt = String::from_utf8_lossy(&out.stdout);
    parse_macos_accelerator_dump(&txt)
}

#[cfg(target_os = "linux")]
fn parse_active_dpm_mhz(p: &std::path::Path) -> i64 {
    let Ok(txt) = std::fs::read_to_string(p) else { return 0; };
    for line in txt.lines() {
        if line.contains('*') {
            if let Some(mhz_part) = line.split_whitespace().find(|w| w.ends_with("Mhz") || w.ends_with("MHz")) {
                let digits: String = mhz_part.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(v) = digits.parse::<i64>() { return v; }
            }
        }
    }
    0
}

#[cfg(target_os = "linux")]
pub fn poll_gpu_telemetry() -> GpuTelemetry {
    let read_i64 = |p: &std::path::Path| -> i64 {
        std::fs::read_to_string(p).ok().and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(0)
    };
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else { return GpuTelemetry::default(); };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("card") && !name.contains('-') {
            let dev = entry.path().join("device");
            if std::fs::read_to_string(dev.join("vendor")).unwrap_or_default().trim().eq_ignore_ascii_case("0x1002") {
                let load = read_i64(&dev.join("gpu_busy_percent"));
                let vram_used = read_i64(&dev.join("mem_info_vram_used")) / (1024 * 1024);
                let vram_total = read_i64(&dev.join("mem_info_vram_total")) / (1024 * 1024);
                let mut gfx = parse_active_dpm_mhz(&dev.join("pp_dpm_sclk"));
                let mut mem = parse_active_dpm_mhz(&dev.join("pp_dpm_mclk"));
                if let Ok(hwmons) = std::fs::read_dir(dev.join("hwmon")) {
                    for hw in hwmons.flatten() {
                        let hp = hw.path();
                        if gfx == 0 { gfx = read_i64(&hp.join("freq1_input")) / 1_000_000; }
                        if mem == 0 { mem = read_i64(&hp.join("freq2_input")) / 1_000_000; }
                        let temp = read_i64(&hp.join("temp1_input")) / 1000;
                        let hot = (read_i64(&hp.join("temp2_input")) / 1000).max(temp);
                        let pwr = read_i64(&hp.join("power1_average")).max(read_i64(&hp.join("power1_input"))) / 1_000_000;
                        let rpm = read_i64(&hp.join("fan1_input"));
                        let pwm = read_i64(&hp.join("pwm1"));
                        let pct = if pwm > 0 { (pwm * 100) / 255 } else { 0 };
                        return GpuTelemetry {
                            valid: true, gfx_mhz: gfx, mem_mhz: mem, temp_c: temp, hotspot_c: hot,
                            load_pct: load, power_w: pwr, fan_rpm: rpm, fan_pct: pct,
                            vram_used_mb: vram_used, vram_total_mb: vram_total,
                        };
                    }
                }
            }
        }
    }
    GpuTelemetry::default()
}

#[cfg(target_os = "windows")]
pub fn poll_gpu_telemetry() -> GpuTelemetry {
    use std::ffi::c_void;

    #[repr(C)]
    union PdhValue {
        long_val: i32,
        double_val: f64,
        large_val: i64,
        ansi_str: *mut i8,
        wide_str: *mut u16,
    }

    #[repr(C)]
    struct PdhFmtCounterValue {
        c_status: u32,
        val: PdhValue,
    }

    #[link(name = "pdh")]
    extern "system" {
        fn PdhOpenQueryW(szDataSource: *const u16, dwUserData: usize, phQuery: *mut *mut c_void) -> u32;
        fn PdhAddEnglishCounterW(hQuery: *mut c_void, szFullCounterPath: *const u16, dwUserData: usize, phCounter: *mut *mut c_void) -> u32;
        fn PdhCollectQueryData(hQuery: *mut c_void) -> u32;
        fn PdhGetFormattedCounterArrayW(
            hCounter: *mut c_void,
            dwFormat: u32,
            lpdwBufferSize: *mut u32,
            lpdwItemCount: *mut u32,
            itemBuffer: *mut u8,
        ) -> u32;
        fn PdhCloseQuery(hQuery: *mut c_void) -> u32;
    }

    const PDH_FMT_DOUBLE: u32 = 0x0000_0200;
    const PDH_FMT_LARGE: u32 = 0x0000_0400;
    const PDH_MORE_DATA: u32 = 0x8000_07D2;

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe fn read_counter_sum_or_max(counter: *mut c_void, fmt: u32, is_sum: bool) -> i64 {
        let mut buf_sz = 0u32;
        let mut count = 0u32;
        let st = PdhGetFormattedCounterArrayW(counter, fmt, &mut buf_sz, &mut count, std::ptr::null_mut());
        if st != PDH_MORE_DATA || buf_sz == 0 || count == 0 {
            return 0;
        }
        let mut buf = vec![0u8; buf_sz as usize];
        if PdhGetFormattedCounterArrayW(counter, fmt, &mut buf_sz, &mut count, buf.as_mut_ptr()) != 0 {
            return 0;
        }
        let stride = (buf_sz as usize) / (count as usize);
        let mut acc = 0.0f64;
        let mut max_v = 0i64;
        for i in 0..(count as usize) {
            let item_ptr = buf.as_ptr().add(i * stride + std::mem::size_of::<*const u16>()) as *const PdhFmtCounterValue;
            let cv = &*item_ptr;
            if cv.c_status == 0 {
                if fmt == PDH_FMT_DOUBLE {
                    acc += cv.val.double_val;
                } else if cv.val.large_val > max_v {
                    max_v = cv.val.large_val;
                }
            }
        }
        if fmt == PDH_FMT_DOUBLE { acc.round() as i64 } else if is_sum { acc.round() as i64 } else { max_v }
    }

    let mut load_pct = 0i64;
    let mut vram_used_mb = 0i64;
    let mut vram_total_mb = 0i64;
    let mut pdh_ok = false;

    unsafe {
        let mut q: *mut c_void = std::ptr::null_mut();
        if PdhOpenQueryW(std::ptr::null(), 0, &mut q) == 0 && !q.is_null() {
            let p_load = to_wide(r"\GPU Engine(*engtype_3D)\Utilization Percentage");
            let p_used = to_wide(r"\GPU Adapter Memory(*)\Dedicated Usage");
            let p_lim = to_wide(r"\GPU Adapter Memory(*)\Dedicated Limit");
            let mut c_load: *mut c_void = std::ptr::null_mut();
            let mut c_used: *mut c_void = std::ptr::null_mut();
            let mut c_lim: *mut c_void = std::ptr::null_mut();
            let _ = PdhAddEnglishCounterW(q, p_load.as_ptr(), 0, &mut c_load);
            let _ = PdhAddEnglishCounterW(q, p_used.as_ptr(), 0, &mut c_used);
            let _ = PdhAddEnglishCounterW(q, p_lim.as_ptr(), 0, &mut c_lim);
            if PdhCollectQueryData(q) == 0 {
                std::thread::sleep(std::time::Duration::from_millis(80));
                if PdhCollectQueryData(q) == 0 {
                    pdh_ok = true;
                    if !c_load.is_null() {
                        load_pct = read_counter_sum_or_max(c_load, PDH_FMT_DOUBLE, true).clamp(0, 100);
                    }
                    if !c_used.is_null() {
                        vram_used_mb = read_counter_sum_or_max(c_used, PDH_FMT_LARGE, false) / (1024 * 1024);
                    }
                    if !c_lim.is_null() {
                        vram_total_mb = read_counter_sum_or_max(c_lim, PDH_FMT_LARGE, false) / (1024 * 1024);
                    }
                }
            }
            let _ = PdhCloseQuery(q);
        }
    }

    GpuTelemetry {
        valid: pdh_ok || vram_used_mb > 0 || vram_total_mb > 0 || load_pct > 0,
        load_pct,
        vram_used_mb,
        vram_total_mb,
        ..Default::default()
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn poll_gpu_telemetry() -> GpuTelemetry { GpuTelemetry::default() }

#[cfg(test)]
mod gpu_telemetry_resilience_tests {
    #[test]
    fn telemetry_poll_completes_without_panic_on_any_host() {
        let snap = super::poll_gpu_telemetry();
        let _ = format!("{:?}", snap);
    }
}
