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
    let needle = format!("\"{}\"=", key);
    let pos = txt.find(&needle)? + needle.len();
    let rest = txt[pos..].trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(rest.len());
    rest[..end].parse::<i64>().ok()
}

#[cfg(target_os = "macos")]
pub fn poll_gpu_telemetry() -> GpuTelemetry {
    let Ok(out) = crate::platform::run_cmd_bounded(Command::new("ioreg").args(["-r", "-c", "IOAccelerator", "-l"]), 1500) else {
        return GpuTelemetry::default();
    };
    let txt = String::from_utf8_lossy(&out.stdout);
    let mut best = GpuTelemetry::default();

    for line in txt.lines() {
        let Some(perf_pos) = line.find("\"PerformanceStatistics\"") else { continue; };
        let slice = &line[perf_pos..];
        let gfx = extract_ioreg_num(slice, "Core Clock(MHz)").unwrap_or(0);
        let mem = extract_ioreg_num(slice, "Memory Clock(MHz)").unwrap_or(0);
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
        let vram_total = extract_ioreg_num(&txt, "VRAM,totalMB")
            .unwrap_or_else(|| if vram_free > 0 { vram_used + vram_free } else { 0 });
        let cand = GpuTelemetry {
            valid: gfx > 0 || mem > 0 || temp > 0 || load > 0 || vram_total > 0,
            gfx_mhz: gfx, mem_mhz: mem, temp_c: temp, hotspot_c: hot,
            load_pct: load, power_w: pwr, fan_rpm: rpm, fan_pct: pct,
            vram_used_mb: vram_used, vram_total_mb: vram_total,
        };
        if cand.valid && (cand.gfx_mhz > best.gfx_mhz || cand.temp_c > best.temp_c || !best.valid) {
            best = cand;
        }
    }
    best
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
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let ps = "$l=(Get-Counter '\\GPU Engine(*engtype_3D)\\Utilization Percentage' -EA SilentlyContinue).CounterSamples.CookedValue|Measure-Object -Sum;$v=(Get-Counter '\\GPU Adapter Memory(*)\\Dedicated Usage' -EA SilentlyContinue).CounterSamples.CookedValue|Measure-Object -Max;\"$([math]::Round($l.Sum))|$([math]::Round($v.Maximum/1MB))\"";
    let Ok(out) = crate::platform::run_cmd_bounded(Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args(["-NoProfile", "-NonInteractive", "-Command", ps])
        , 1500) else { return GpuTelemetry::default(); };
    let txt = String::from_utf8_lossy(&out.stdout);
    let mut parts = txt.trim().split('|');
    let load = parts.next().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
    let vram_used = parts.next().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
    GpuTelemetry { valid: load > 0 || vram_used > 0, load_pct: load, vram_used_mb: vram_used, ..Default::default() }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn poll_gpu_telemetry() -> GpuTelemetry { GpuTelemetry::default() }
