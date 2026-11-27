pub fn encode_base64(data: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        out.push(CHARS[b0 >> 2] as char);
        out.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        out.push(if chunk.len() > 1 { CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)] as char } else { '=' });
        out.push(if chunk.len() > 2 { CHARS[b2 & 0x3f] as char } else { '=' });
    }
    out
}

pub fn format_reg_ppt(data: &[u8]) -> String {
    let mut out = String::from(
        r#"Windows Registry Editor Version 5.00

[HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000]
"PP_PhmSoftPowerPlayTable"=hex:"#
    );
    let mut col = 27usize;
    for (i, &b) in data.iter().enumerate() {
        let byte_str = format!("{:02X}", b);
        if i + 1 < data.len() {
            if col + 3 > 78 {
                out.push_str(&format!("{},\\\r\n  ", byte_str));
                col = 2;
            } else {
                out.push_str(&format!("{},", byte_str));
                col += 3;
            }
        } else {
            out.push_str(&byte_str);
        }
    }
    out.push_str("\r\n");
    out
}

fn infer_default_did_for_ppt(data: &[u8]) -> u16 {
    let frev = data.get(2).copied().unwrap_or(0);
    match frev {
        0x07 => 0x67DF, // Polaris (RX 580)
        0x08 => 0x687F, // Vega 10 (RX Vega 64)
        0x0B => 0x66AF, // Vega 20 (Radeon VII)
        0x0C | 0x0D => 0x731F, // RDNA 1 (RX 5700 XT)
        0x0F => 0x73BF, // RDNA 2 Navi 21 (RX 6800/6900 XT)
        0x10 | 0x11 => 0x73DF, // RDNA 2 Navi 22 (RX 6700 XT)
        0x12 => 0x73FF, // RDNA 2 Navi 23 (RX 6600/6600 XT)
        0x13 => 0x743F, // RDNA 2 Navi 24 (RX 6400/6500 XT)
        0x14 => 0x744C, // RDNA 3 Navi 31 (RX 7900 XTX)
        0x15 => 0x747E, // RDNA 3 Navi 32 (RX 7800 XT)
        0x16 => 0x7480, // RDNA 3 Navi 33 (RX 7600)
        _ => 0x73FF,
    }
}

pub fn format_mpt_ppt_with_did(data: &[u8], device_id: Option<u16>) -> Vec<u8> {
    let did = device_id.filter(|&d| d != 0).unwrap_or_else(|| infer_default_did_for_ppt(data));
    let mut out = vec![0u8; 0x100 + data.len()];
    out[0] = 0x55;
    out[1] = 0xAA;
    out[0x18] = 0x3C;
    out[0x42..0x44].copy_from_slice(&did.to_le_bytes());
    out[0x48] = 0x50;
    out[0x70] = 0x80;
    out[0xA3] = 0x01;
    out[0x100..].copy_from_slice(data);
    out
}

pub fn format_mpt_ppt(data: &[u8]) -> Vec<u8> {
    format_mpt_ppt_with_did(data, None)
}

pub fn format_opencore_plist(data: &[u8], pci_path: &str) -> String {
    format_opencore_plist_with_key(data, pci_path, "PP,PhmSoftPowerPlayTable")
}

pub fn format_opencore_plist_with_key(data: &[u8], pci_path: &str, key_name: &str) -> String {
    let b64 = encode_base64(data);
    let wrapped: String = b64
        .as_bytes()
        .chunks(64)
        .map(|c| format!("\t\t\t\t{}\n", std::str::from_utf8(c).unwrap_or("")))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://apple.com">
<plist version="1.0">
<dict>
	<key>DeviceProperties</key>
	<dict>
		<key>Add</key>
		<dict>
			<key>{}</key>
			<dict>
				<key>{}</key>
				<data>
{}				</data>
			</dict>
		</dict>
	</dict>
</dict>
</plist>
"#,
        pci_path, key_name, wrapped
    )
}

pub fn serialize_export_bytes(
    ext: &str,
    raw_buffer: &[u8],
    vbios_buffer: Option<&[u8]>,
    vbios_ppt_offset: Option<usize>,
    pci_path: &str,
) -> Result<Vec<u8>, String> {
    match ext {
        "rom" => match (vbios_buffer, vbios_ppt_offset) {
            (Some(orig_rom), Some(off)) => crate::vbios::patch_ppt_into_rom(orig_rom, off, raw_buffer),
            _ => Err("You must load an original VBIOS (.rom / .bin) before saving a patched .rom image.".into()),
        },
        "reg" => Ok(format_reg_ppt(raw_buffer).into_bytes()),
        "mpt" => {
            let did = vbios_buffer
                .and_then(|rom| crate::vbios::inspect_vbios_info(rom, vbios_ppt_offset.unwrap_or(0)))
                .map(|vi| vi.device_id);
            Ok(format_mpt_ppt_with_did(raw_buffer, did))
        }
        "plist" => Ok(format_opencore_plist(raw_buffer, pci_path).into_bytes()),
        _ => Ok(raw_buffer.to_vec()),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_mpt_reg_plist_all_table_sizes() {
        for (sz, rev) in [(833usize, 0x07u8), (1534, 0x0B), (1674, 0x0C), (2470, 0x0F), (5424, 0x15)] {
            let mut sample = vec![0u8; sz];
            sample[0..2].copy_from_slice(&(sz as u16).to_le_bytes());
            sample[2] = rev;
            for (i, b) in sample.iter_mut().enumerate().skip(4) {
                *b = (i.wrapping_mul(31) & 0xFF) as u8;
            }

            let mpt = format_mpt_ppt(&sample);
            assert_eq!(crate::import_formats::parse_mpt_ppt(&mpt).unwrap_or_default(), sample);

            let reg = format_reg_ppt(&sample).into_bytes();
            assert_eq!(crate::import_formats::parse_reg_ppt(&reg).unwrap_or_default(), sample);

            let plist = format_opencore_plist(&sample, "PciRoot(0x0)/Pci(0x1,0x0)/Pci(0x0,0x0)").into_bytes();
            assert_eq!(crate::import_formats::parse_plist_ppt(&plist).unwrap_or_default(), sample);
        }
    }
}
