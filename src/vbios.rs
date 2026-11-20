pub fn extract_ppt_from_rom(rom: &[u8]) -> Result<(Vec<u8>, usize), String> {
    if rom.len() < 0x100 || !rom.starts_with(&[0x55, 0xAA]) {
        return Err("Not a valid PCI Option ROM (missing 55 AA signature)".into());
    }
    let rom_header_off = u16::from_le_bytes([rom[0x48], rom[0x49]]) as usize;
    if rom_header_off + 0x24 <= rom.len() {
        for hdr_ptr_off in [rom_header_off + 0x20, rom_header_off + 0x22] {
            let master_data_off = u16::from_le_bytes([rom[hdr_ptr_off], rom[hdr_ptr_off + 1]]) as usize;
            if master_data_off + 4 <= rom.len() {
                let mdt_size = u16::from_le_bytes([rom[master_data_off], rom[master_data_off + 1]]) as usize;
                let num_entries = mdt_size.saturating_sub(4) / 2;
                for idx in 0..num_entries {
                    let tbl_idx_off = 4 + idx * 2;
                    if master_data_off + tbl_idx_off + 2 <= rom.len() {
                        let ppt_off = u16::from_le_bytes([rom[master_data_off + tbl_idx_off], rom[master_data_off + tbl_idx_off + 1]]) as usize;
                        if ppt_off > 0x100 && ppt_off + 6 <= rom.len() {
                            let ppt_size = u16::from_le_bytes([rom[ppt_off], rom[ppt_off + 1]]) as usize;
                            let frev = rom[ppt_off + 2];
                            if (1500..=4096).contains(&ppt_size) && matches!(frev, 0x12 | 0x0F | 0x14) && ppt_off + ppt_size <= rom.len() {
                                return Ok((rom[ppt_off..ppt_off + ppt_size].to_vec(), ppt_off));
                            }
                        }
                    }
                }
            }
        }
    }

    // Direct signature scan for RDNA 2 PowerPlay table (A6 09 12 xx 02)
    for off in 0x100..rom.len().saturating_sub(std::mem::size_of::<crate::pptable_rdna2::AtomSmcDpmInfoV4_9>()) {
        let sz = u16::from_le_bytes([rom[off], rom[off + 1]]) as usize;
        if sz >= 2400 && sz <= 2600 && rom[off + 2] == 0x12 && rom[off + 4] == 0x02 && off + sz <= rom.len() {
            return Ok((rom[off..off + sz].to_vec(), off));
        }
    }

    for off in 0x100..rom.len().saturating_sub(1500) {
        let sz = u16::from_le_bytes([rom[off], rom[off + 1]]) as usize;
        let frev = rom[off + 2];
        if (1500..=4096).contains(&sz) && matches!(frev, 0x0F | 0x14) && rom[off + 4] <= 0x04 && off + sz <= rom.len() {
            return Ok((rom[off..off + sz].to_vec(), off));
        }
    }

    Err("Could not locate a valid AMD PowerPlay table inside VBIOS ROM".into())
}

pub fn patch_ppt_into_rom(orig_rom: &[u8], ppt_off: usize, new_ppt: &[u8]) -> Result<Vec<u8>, String> {
    if orig_rom.len() < 0x100 || !orig_rom.starts_with(&[0x55, 0xAA]) {
        return Err("No valid VBIOS ROM loaded to patch".into());
    }
    if ppt_off + new_ppt.len() > orig_rom.len() {
        return Err("Patched PowerPlay table exceeds VBIOS ROM bounds".into());
    }
    let orig_size = u16::from_le_bytes([orig_rom[ppt_off], orig_rom[ppt_off + 1]]) as usize;
    if orig_size != new_ppt.len() {
        return Err(format!(
            "PPTable size mismatch (VBIOS expects {} bytes, buffer is {} bytes)",
            orig_size,
            new_ppt.len()
        ));
    }

    let mut rom = orig_rom.to_vec();
    rom[ppt_off..ppt_off + new_ppt.len()].copy_from_slice(new_ppt);

    // Recalculate Legacy PCI Option ROM 8-bit checksum (sum of first rom[2] * 512 bytes == 0 mod 256)
    let legacy_blocks = rom[2] as usize;
    let legacy_len = (legacy_blocks * 512).min(rom.len());
    if legacy_len > 0x21 {
        // Byte 0x21 is the standard ATOMBIOS ROM checksum byte
        rom[0x21] = 0;
        let sum: u8 = rom[..legacy_len].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        rom[0x21] = 0u8.wrapping_sub(sum);
    }

    Ok(rom)
}

pub struct VbiosInfo {
    pub total_size: usize,
    pub legacy_size: usize,
    pub checksum_byte: u8,
    pub checksum_sum: u8,
    pub ppt_offset: usize,
    pub ppt_size: usize,
    pub bios_version: String,
    pub bios_ver_offset: Option<usize>,
    pub vendor_id: u16,
    pub device_id: u16,
    pub subvendor_id: u16,
    pub subdevice_id: u16,
    pub pci_vendor_name: &'static str,
    pub device_name: &'static str,
    pub vendor_name: &'static str,
    pub psp_dir_offset: Option<usize>,
    pub psp_entry_count: usize,
    pub psp_pubkey_offset: Option<usize>,
    pub psp_signed_size: Option<usize>,
    pub rsa_sig_offset: Option<usize>,
    pub rsa_sig_size: Option<usize>,
}

fn lookup_pci_vendor(vid: u16) -> &'static str {
    match vid {
        0x1002 | 0x1022 => "AMD / ATI",
        0x10DE => "NVIDIA",
        0x8086 => "Intel",
        _ => "Unknown Vendor",
    }
}

fn lookup_amd_device_name(did: u16) -> &'static str {
    match did {
        0x73BF => "AMD Radeon RX 6800 / 6800 XT / 6900 XT (Navi 21)",
        0x73AF => "AMD Radeon RX 6900 XT / 6950 XT (Navi 21 XTXH)",
        0x73DF => "AMD Radeon RX 6700 XT / 6750 XT (Navi 22)",
        0x73E0 | 0x73E1 | 0x73E3 => "AMD Radeon RX 6700 / 6650M (Navi 22)",
        0x73FF => "AMD Radeon RX 6600 / 6600 XT / 6600M (Navi 23)",
        0x73EF => "AMD Radeon RX 6650 XT (Navi 23)",
        0x743F => "AMD Radeon RX 6500 XT / 6400 (Navi 24)",
        0x7422 => "AMD Radeon PRO W6400 (Navi 24)",
        0x731F => "AMD Radeon RX 5700 / 5700 XT (Navi 10)",
        0x7340 => "AMD Radeon RX 5500 / 5500 XT (Navi 14)",
        _ => "AMD Radeon GPU",
    }
}

fn lookup_aib_vendor(svid: u16) -> &'static str {
    match svid {
        0x1002 => "AMD Reference",
        0x1043 => "ASUS",
        0x1458 => "Gigabyte",
        0x1462 => "MSI",
        0x148C => "PowerColor",
        0x174B => "Sapphire",
        0x1DA2 => "Sapphire (OEM)",
        0x1849 => "ASRock",
        0x1EAE | 0x1682 => "XFX",
        0x1545 => "VisionTek",
        0x196D => "Club 3D",
        0x1092 => "Diamond Multimedia",
        0x1787 => "HIS",
        0x1B0A => "Pegatron OEM",
        0x1028 => "Dell / Alienware",
        0x103C => "HP OEM",
        0x17AA => "Lenovo OEM",
        0x1025 => "Acer OEM",
        0x106B => "Apple",
        0x144D => "Samsung",
        0x1179 => "Toshiba / Dynabook",
        0x1558 => "Clevo",
        0x1D05 => "Tongfang",
        0x19DA => "PC Partner",
        0x1B4C => "KFA2 / Galaxy",
        0x7377 => "Colorful / AIB OEM",
        _ => "AIB / OEM",
    }
}

pub fn inspect_vbios_info(rom: &[u8], ppt_off: usize) -> Option<VbiosInfo> {
    if rom.len() < 0x100 || !rom.starts_with(&[0x55, 0xAA]) {
        return None;
    }
    let legacy_len = ((rom[2] as usize) * 512).min(rom.len());
    let checksum_byte = rom[0x21];
    let checksum_sum = rom[..legacy_len].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    let ppt_size = if ppt_off + 2 <= rom.len() {
        u16::from_le_bytes([rom[ppt_off], rom[ppt_off + 1]]) as usize
    } else {
        0
    };

    let pcir_off = u16::from_le_bytes([rom[0x18], rom[0x19]]) as usize;
    let (vendor_id, device_id) = if pcir_off + 8 <= rom.len() && &rom[pcir_off..pcir_off + 4] == b"PCIR" {
        (
            u16::from_le_bytes([rom[pcir_off + 4], rom[pcir_off + 5]]),
            u16::from_le_bytes([rom[pcir_off + 6], rom[pcir_off + 7]]),
        )
    } else {
        (0x1002, 0x0000)
    };

    let rom_hdr = u16::from_le_bytes([rom[0x48], rom[0x49]]) as usize;
    let (bios_version, bios_ver_offset, subvendor_id, subdevice_id) = if rom_hdr + 0x20 <= rom.len() {
        let svid = u16::from_le_bytes([rom[rom_hdr + 0x18], rom[rom_hdr + 0x19]]);
        let ssid = u16::from_le_bytes([rom[rom_hdr + 0x1A], rom[rom_hdr + 0x1B]]);
        let needle = b"ATOMBIOSBK-AMD";
        let found_off = rom[..legacy_len].windows(needle.len()).position(|w| w == needle);
        let str_off = found_off.unwrap_or_else(|| u16::from_le_bytes([rom[rom_hdr + 0x10], rom[rom_hdr + 0x11]]) as usize);
        let ver = if str_off > 0 && str_off < rom.len() {
            let end = rom[str_off..].iter().position(|&b| b == 0 || b == b'\r' || b == b'\n').map(|p| str_off + p).unwrap_or((str_off + 48).min(rom.len()));
            let txt = String::from_utf8_lossy(&rom[str_off..end]).trim().to_string();
            if txt.is_empty() { "Unknown".into() } else { txt }
        } else {
            "Unknown".into()
        };
        (ver, if str_off > 0 && str_off < rom.len() { Some(str_off) } else { None }, svid, ssid)
    } else {
        ("Unknown".into(), None, 0x0000, 0x0000)
    };

    let mut psp_dir_offset = None;
    let mut psp_entry_count = 0usize;
    let mut psp_pubkey_offset = None;
    let mut psp_signed_size = None;
    let mut rsa_sig_offset = None;
    let mut rsa_sig_size = None;

    if rom_hdr + 0x28 <= rom.len() {
        let psp_off = u32::from_le_bytes([
            rom[rom_hdr + 0x24],
            rom[rom_hdr + 0x25],
            rom[rom_hdr + 0x26],
            rom[rom_hdr + 0x27],
        ]) as usize;
        if psp_off > 0 && psp_off + 16 <= rom.len() && &rom[psp_off..psp_off + 4] == b"$PSP" {
            psp_dir_offset = Some(psp_off);
            let cnt = u32::from_le_bytes([
                rom[psp_off + 8],
                rom[psp_off + 9],
                rom[psp_off + 10],
                rom[psp_off + 11],
            ]) as usize;
            psp_entry_count = cnt.min(64);
            for i in 0..psp_entry_count {
                let e = psp_off + 16 + i * 16;
                if e + 16 > rom.len() { break; }
                let etype = rom[e];
                let esize = u32::from_le_bytes([rom[e + 4], rom[e + 5], rom[e + 6], rom[e + 7]]) as usize;
                let eloc = u64::from_le_bytes([
                    rom[e + 8], rom[e + 9], rom[e + 10], rom[e + 11],
                    rom[e + 12], rom[e + 13], rom[e + 14], rom[e + 15],
                ]) as usize;
                match etype {
                    0x00 => psp_pubkey_offset = Some(eloc),
                    0x04 => psp_signed_size = Some(esize),
                    0x05 => {
                        rsa_sig_offset = Some(eloc);
                        rsa_sig_size = Some(esize);
                    }
                    _ => {}
                }
            }
        }
    }

    Some(VbiosInfo {
        total_size: rom.len(),
        legacy_size: legacy_len,
        checksum_byte,
        checksum_sum,
        ppt_offset: ppt_off,
        ppt_size,
        bios_version,
        bios_ver_offset,
        vendor_id,
        device_id,
        subvendor_id,
        subdevice_id,
        pci_vendor_name: lookup_pci_vendor(vendor_id),
        device_name: lookup_amd_device_name(device_id),
        vendor_name: lookup_aib_vendor(subvendor_id),
        psp_dir_offset,
        psp_entry_count,
        psp_pubkey_offset,
        psp_signed_size,
        rsa_sig_offset,
        rsa_sig_size,
    })
}
