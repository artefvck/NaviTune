#[derive(Debug, Clone)]
pub struct VbiosInfo {
    pub device_name: String,
    pub vendor_id: u16,
    pub device_id: u16,
    pub subvendor_id: u16,
    pub subdevice_id: u16,
    pub vendor_name: String,
    pub pci_vendor_name: String,
    pub bios_version: String,
    pub bios_ver_offset: Option<usize>,
    pub total_size: usize,
    pub legacy_size: usize,
    pub atom_header_offset: Option<usize>,
    pub master_data_offset: Option<usize>,
    pub ppt_offset: usize,
    pub ppt_size: usize,
    pub psp_dir_offset: Option<usize>,
    pub psp_entry_count: usize,
    pub psp_pubkey_offset: Option<usize>,
    pub psp_signed_size: Option<usize>,
    pub rsa_sig_offset: Option<usize>,
    pub rsa_sig_size: Option<usize>,
    pub has_legacy_opt_rom: bool,
    pub checksum_byte: u8,
    pub checksum_sum: u8,
}

fn read_u16(buf: &[u8], off: usize) -> Option<u16> {
    buf.get(off..off + 2).map(|s| u16::from_le_bytes([s[0], s[1]]))
}

fn read_u32(buf: &[u8], off: usize) -> Option<u32> {
    buf.get(off..off + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

/// Resolves PowerPlayInfo via AMD AtomBIOS pointer chain (0x48 -> AtomRomHeader -> MasterDataTable -> PowerPlayInfo),
/// matching Python `upp`.
pub fn resolve_master_powerplay_table(buf: &[u8]) -> Option<(usize, usize, usize, usize)> {
    let atom_hdr = read_u16(buf, 0x48)? as usize;
    if buf.get(atom_hdr + 4..atom_hdr + 8) != Some(b"ATOM") {
        return None;
    }
    let master_data = read_u16(buf, atom_hdr + 0x20)? as usize;
    let mdt_size = read_u16(buf, master_data)? as usize;
    let max_entries = mdt_size.saturating_sub(4) / 2;
    // atom_master_list_of_data_tables_v2_1.powerplayinfo is index 11 (or 19/20/21 on older revs)
    let mut order = vec![15usize, 11, 19, 20, 21];
    for i in 0..max_entries {
        if !order.contains(&i) {
            order.push(i);
        }
    }
    for idx in order {
        let ptr_off = master_data + 4 + idx * 2;
        if ptr_off + 2 > master_data + mdt_size || ptr_off + 2 > buf.len() {
            continue;
        }
        let ppt_off = read_u16(buf, ptr_off).unwrap_or(0) as usize;
        if ppt_off < 0x400 || ppt_off + 8 > buf.len() {
            continue;
        }
        let ppt_size = read_u16(buf, ppt_off).unwrap_or(0) as usize;
        let frev = buf[ppt_off + 2];
        let crev = buf[ppt_off + 3];
        let trev = buf[ppt_off + 4];
        if (1500..=6144).contains(&ppt_size)
            && ppt_off + ppt_size <= buf.len()
            && matches!(frev, 0x0F | 0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16)
            && crev <= 0x10
            && trev <= 0x06
        {
            return Some((atom_hdr, master_data, ppt_off, ppt_size));
        }
    }
    None
}

fn scan_powerplay_signature(buf: &[u8]) -> Option<(usize, usize)> {
    let known_frev = [0x07u8, 0x08, 0x0B, 0x0C, 0x0F, 0x12, 0x13, 0x14, 0x15, 0x16];
    for i in 0..buf.len().saturating_sub(8) {
        let sz = read_u16(buf, i)? as usize;
        let frev = buf[i + 2];
        let crev = buf[i + 3];
        let trev = buf[i + 4];
        if sz >= 256
            && sz <= 8192
            && i + sz <= buf.len()
            && known_frev.contains(&frev)
            && crev <= 0x10
            && trev <= 0x06
        {
            return Some((i, sz));
        }
    }
    None
}

pub fn extract_ppt_from_rom(rom: &[u8]) -> Result<(Vec<u8>, usize), String> {
    if rom.starts_with(&[0x55, 0xAA]) {
        if let Some((_ah, _md, ppt_off, ppt_size)) = resolve_master_powerplay_table(rom) {
            return Ok((rom[ppt_off..ppt_off + ppt_size].to_vec(), ppt_off));
        }
    }
    // 2 MB RDNA 3 (Navi 31/32/33) SPI flash images store the 5,424-byte (0x1530) PowerPlayInfo table inside the PSP image region
    for i in (0..rom.len().saturating_sub(5424)).step_by(4) {
        if rom[i] == 0x30
            && rom[i + 1] == 0x15
            && matches!(rom[i + 2], 0x14 | 0x15 | 0x16)
            && rom[i + 3] <= 0x10
            && rom[i + 4] == 0x03
        {
            return Ok((rom[i..i + 5424].to_vec(), i));
        }
    }
    if let Some((ppt_off, ppt_size)) = scan_powerplay_signature(rom) {
        return Ok((rom[ppt_off..ppt_off + ppt_size].to_vec(), ppt_off));
    }
    Err("Could not locate MasterDataTable -> PowerPlayInfo or valid RDNA 3 (5424 B) PPTable in ROM.".to_string())
}

pub fn patch_ppt_into_rom(orig_rom: &[u8], ppt_offset: usize, new_ppt: &[u8]) -> Result<Vec<u8>, String> {
    if ppt_offset + new_ppt.len() > orig_rom.len() {
        return Err(format!(
            "PPTable size ({} B) exceeds VBIOS bounds at offset 0x{:X}.",
            new_ppt.len(),
            ppt_offset
        ));
    }
    let mut out = orig_rom.to_vec();
    out[ppt_offset..ppt_offset + new_ppt.len()].copy_from_slice(new_ppt);
    recalc_vbios_checksum(&mut out);
    Ok(out)
}

pub fn recalc_vbios_checksum(buf: &mut [u8]) {
    let opt_base = if buf.starts_with(&[0x55, 0xAA]) {
        Some(0usize)
    } else {
        (0..buf.len().saturating_sub(0x60)).step_by(512).find(|&o| buf.get(o..o + 2) == Some(b"\x55\xAA"))
    };
    let Some(base) = opt_base else { return; };
    let slice = &mut buf[base..];
    let legacy_len = slice.get(2).map(|&b| (b as usize) * 512).filter(|&l| l > 0 && l <= slice.len()).unwrap_or(slice.len());
    if legacy_len <= 0x21 {
        return;
    }
    slice[0x21] = 0;
    let sum = slice[..legacy_len].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
    slice[0x21] = 0u8.wrapping_sub(sum);
}

fn gpu_device_name(did: u16) -> &'static str {
    match did {
        0x6860..=0x6864 | 0x6867 | 0x6868 | 0x686C | 0x687F => "AMD Radeon RX Vega 56 / 64 / Frontier / Pro WX (Vega 10)",
        0x66A0..=0x66A3 | 0x66A7 | 0x66AF => "AMD Radeon VII / Pro Vega II / Instinct MI50 (Vega 20)",
        0x7310 | 0x7312 | 0x731F => "AMD Radeon RX 5600 / 5700 XT (Navi 10)",
        0x7340 | 0x7341 | 0x7347 | 0x734F => "AMD Radeon RX 5300 / 5500 XT / Pro W5500 (Navi 14)",
        0x7360 | 0x7362 => "AMD Radeon Pro V520 / RX 5600M (Navi 12)",
        0x73BF => "Navi 21 (RX 6800/6900)",
        0x73AF => "Navi 21 XTXH (RX 6900 XT)",
        0x73DF => "Navi 22 (RX 6700/6750 XT)",
        0x73EF => "Navi 23 (RX 6650 XT)",
        0x73FF => "Navi 23 (RX 6600/6600 XT)",
        0x73E0 | 0x73E1 | 0x73E3 => "Navi 23 (RX 6600M/W6600)",
        0x743F => "Navi 24 (RX 6400/6500 XT)",
        0x7422 | 0x7423 | 0x7424 => "Navi 24 (Beige Goby)",
        0x744C | 0x7448 | 0x745E => "Navi 31 (RX 7900 XTX/XT/GRE)",
        0x747E | 0x7470 | 0x7479 => "Navi 32 (RX 7800/7700 XT)",
        0x7480 | 0x7483 | 0x7489 => "Navi 33 (RX 7600/7600 XT)",
        _ => "AMD Radeon GPU",
    }
}

fn subsystem_vendor_name(svid: u16) -> &'static str {
    match svid {
        0x1002 => "AMD Reference",
        0x1028 => "Dell",
        0x1043 => "ASUS",
        0x1458 => "Gigabyte",
        0x1462 => "MSI",
        0x148C => "PowerColor",
        0x1682 => "XFX",
        0x1DA2 => "Sapphire",
        0x1EAE => "XFX",
        0x103C => "HP",
        0x17AA => "Lenovo",
        _ => "AIB Partner",
    }
}

pub fn inspect_vbios_info(buf: &[u8], ppt_offset: usize) -> Option<VbiosInfo> {
    if buf.len() < 0x50 {
        return None;
    }
    let opt_base = if buf.starts_with(&[0x55, 0xAA]) {
        0usize
    } else {
        (0..buf.len().saturating_sub(0x60))
            .step_by(512)
            .find(|&o| buf.get(o..o + 2) == Some(b"\x55\xAA"))
            .unwrap_or(0)
    };
    let slice = &buf[opt_base..];
    let resolved = resolve_master_powerplay_table(slice);
    let atom_hdr_rel = resolved.map(|r| r.0).or_else(|| {
        let ah = read_u16(slice, 0x48)? as usize;
        (slice.get(ah + 4..ah + 8) == Some(b"ATOM")).then_some(ah)
    });
    let master_data_rel = resolved
        .map(|r| r.1)
        .or_else(|| atom_hdr_rel.and_then(|ah| read_u16(slice, ah + 0x20).map(|m| m as usize)).filter(|&m| m > 0 && m + 0x20 <= slice.len()));
    let atom_header_offset = atom_hdr_rel.map(|ah| opt_base + ah);
    let master_data_offset = master_data_rel.map(|md| opt_base + md);

    let pcir_off = read_u16(slice, 0x18).unwrap_or(0) as usize;
    let (vendor_id, device_id, legacy_size) = if slice.get(pcir_off..pcir_off + 4) == Some(b"PCIR") {
        let vid = read_u16(slice, pcir_off + 4).unwrap_or(0x1002);
        let did = read_u16(slice, pcir_off + 6).unwrap_or(0);
        let ilen = (read_u16(slice, pcir_off + 0x10).unwrap_or(0) as usize) * 512;
        (vid, did, if ilen > 0 && ilen <= buf.len() { ilen } else { buf.len() })
    } else {
        (0x1002, 0, buf.len())
    };

    let (subvendor_id, subdevice_id) = if let Some(ah) = atom_hdr_rel {
        let sv18 = read_u16(slice, ah + 0x18).unwrap_or(0);
        if sv18 != 0 && sv18 != 0xFFFF {
            (sv18, read_u16(slice, ah + 0x1A).unwrap_or(0))
        } else {
            (read_u16(slice, ah + 0x26).unwrap_or(0), read_u16(slice, ah + 0x28).unwrap_or(0))
        }
    } else {
        (0, 0)
    };

    let mut bios_version = String::from("Unknown");
    let mut bios_ver_offset = None;
    for i in 0..buf.len().saturating_sub(24) {
        if buf.get(i..i + 3) == Some(b"VER") && buf.get(i + 6) == Some(&b'.') {
            let end = buf[i..].iter().position(|&b| b == 0 || b == b'\r' || b == b'\n').unwrap_or(24).min(32);
            bios_version = String::from_utf8_lossy(&buf[i..i + end]).trim().to_string();
            bios_ver_offset = Some(i);
            break;
        }
    }

    let ppt_size = read_u16(buf, ppt_offset).unwrap_or(0) as usize;

    let mut psp_dir_offset = None;
    let mut psp_entry_count = 0usize;
    let mut psp_pubkey_offset = None;
    let mut psp_signed_size = None;
    let mut rsa_sig_offset = None;
    let mut rsa_sig_size = None;

    for off in (0..buf.len().saturating_sub(16)).step_by(4) {
        if buf.get(off..off + 4) == Some(b"$PSP") {
            psp_dir_offset = Some(off);
            let count = read_u32(buf, off + 8).unwrap_or(0) as usize;
            psp_entry_count = count.min(64);
            for idx in 0..psp_entry_count {
                let ent = off + 16 + idx * 16;
                if ent + 16 > buf.len() {
                    break;
                }
                let etype = buf[ent];
                let esize = read_u32(buf, ent + 4).unwrap_or(0) as usize;
                let eloc = read_u32(buf, ent + 8).unwrap_or(0) as usize;
                if etype == 0x05 && eloc < buf.len() {
                    psp_pubkey_offset = Some(eloc);
                } else if etype == 0x07 && eloc < buf.len() {
                    rsa_sig_offset = Some(eloc);
                    rsa_sig_size = Some(esize);
                } else if esize > 0 && eloc < buf.len() {
                    psp_signed_size = Some(psp_signed_size.unwrap_or(0).max(esize));
                }
            }
            break;
        }
    }

    let has_legacy_opt_rom = slice.starts_with(&[0x55, 0xAA]);
    let csum_len = legacy_size.min(slice.len());
    let checksum_byte = slice.get(0x21).copied().unwrap_or(0);
    let checksum_sum = slice[..csum_len].iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

    Some(VbiosInfo {
        device_name: gpu_device_name(device_id).to_string(),
        vendor_id,
        device_id,
        subvendor_id,
        subdevice_id,
        vendor_name: subsystem_vendor_name(subvendor_id).to_string(),
        pci_vendor_name: if vendor_id == 0x1002 { "AMD / ATI".to_string() } else { format!("0x{:04X}", vendor_id) },
        bios_version,
        bios_ver_offset,
        total_size: buf.len(),
        legacy_size,
        atom_header_offset,
        master_data_offset,
        ppt_offset,
        ppt_size,
        psp_dir_offset,
        psp_entry_count,
        psp_pubkey_offset,
        psp_signed_size,
        rsa_sig_offset,
        rsa_sig_size,
        has_legacy_opt_rom,
        checksum_byte,
        checksum_sum,
    })
}
