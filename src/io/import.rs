fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut table = [255u8; 256];
    for (i, &c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
        table[c as usize] = i as u8;
    }
    let clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if clean.is_empty() {
        return Err("Empty Base64 payload in .mpt file".into());
    }
    let mut out = Vec::with_capacity(clean.len() * 3 / 4);
    for chunk in clean.chunks(4) {
        if chunk.len() < 4 {
            break;
        }
        let a = table[chunk[0] as usize];
        let b = table[chunk[1] as usize];
        if a == 255 || b == 255 {
            return Err("Invalid Base64 character in .mpt PowerPlayTable".into());
        }
        out.push((a << 2) | (b >> 4));
        if chunk[2] != b'=' {
            let c = table[chunk[2] as usize];
            if c == 255 { return Err("Invalid Base64 character".into()); }
            out.push(((b & 0x0f) << 4) | (c >> 2));
            if chunk[3] != b'=' {
                let d = table[chunk[3] as usize];
                if d == 255 { return Err("Invalid Base64 character".into()); }
                out.push(((c & 0x03) << 6) | d);
            }
        }
    }
    Ok(out)
}

fn decode_text_auto(raw: &[u8]) -> String {
    if raw.starts_with(&[0xFF, 0xFE]) {
        let u16s: Vec<u16> = raw[2..]
            .as_chunks::<2>().0.iter()
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&u16s)
    } else if raw.starts_with(&[0xFE, 0xFF]) {
        let u16s: Vec<u16> = raw[2..]
            .as_chunks::<2>().0.iter()
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&u16s)
    } else if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        String::from_utf8_lossy(&raw[3..]).into_owned()
    } else {
        String::from_utf8_lossy(raw).into_owned()
    }
}

pub fn parse_reg_ppt(raw: &[u8]) -> Result<Vec<u8>, String> {
    let text = decode_text_auto(raw);
    let key_pos = text
        .find("PP_PhmSoftPowerPlayTable")
        .ok_or_else(|| "PP_PhmSoftPowerPlayTable key not found in .reg file".to_string())?;
    let after_key = &text[key_pos..];
    let hex_pos = after_key
        .find("hex:")
        .ok_or_else(|| "Missing hex: value after PP_PhmSoftPowerPlayTable".to_string())?;
    let mut bytes = Vec::new();
    for line in after_key[hex_pos + 4..].lines() {
        let trimmed = line.trim();
        let cont = trimmed.ends_with('\\');
        let clean = trimmed.trim_end_matches('\\');
        for tok in clean.split(',') {
            let h = tok.trim();
            if h.len() == 2 {
                if let Ok(b) = u8::from_str_radix(h, 16) {
                    bytes.push(b);
                }
            }
        }
        if !cont {
            break;
        }
    }
    if bytes.len() < 64 {
        return Err(format!("Parsed PowerPlay table too short ({} bytes)", bytes.len()));
    }
    Ok(bytes)
}

pub fn parse_mpt_ppt(raw: &[u8]) -> Result<Vec<u8>, String> {
    // Igor's Lab binary .mpt format: 256-byte (0x100) 55 AA header + raw PowerPlay table
    if raw.len() >= 0x104 && raw.starts_with(&[0x55, 0xAA]) {
        let sz = u16::from_le_bytes([raw[0x100], raw[0x101]]) as usize;
        if sz >= 64 && 0x100 + sz <= raw.len() {
            return Ok(raw[0x100..0x100 + sz].to_vec());
        }
    }

    let text = decode_text_auto(raw);
    let tags = [
        ("PowerPlayTable>", "</PowerPlayTable>"),
        ("SoftPowerPlayTable>", "</SoftPowerPlayTable>"),
        ("PP_PhmSoftPowerPlayTable>", "</PP_PhmSoftPowerPlayTable>"),
    ];
    for (open_tag, close_tag) in tags {
        if let Some(s_idx) = text.find(open_tag) {
            let val_start = s_idx + open_tag.len();
            if let Some(e_rel) = text[val_start..].find(close_tag) {
                let inner = text[val_start..val_start + e_rel].trim();
                if inner.is_empty() {
                    continue;
                }
                // Try hex pairs first if string only contains hex digits, commas, or spaces
                if inner.contains(',') || inner.contains('-') {
                    let bytes: Vec<u8> = inner
                        .split(|c: char| c == ',' || c == '-' || c.is_whitespace())
                        .filter(|t| t.len() == 2)
                        .filter_map(|t| u8::from_str_radix(t, 16).ok())
                        .collect();
                    if bytes.len() >= 64 {
                        return Ok(bytes);
                    }
                }
                let decoded = decode_base64(inner)?;
                if decoded.len() >= 64 {
                    return Ok(decoded);
                }
            }
        }
    }
    Err("Could not find <PowerPlayTable> or <SoftPowerPlayTable> payload in .mpt file".into())
}

fn decode_base64_clean(input: &str) -> Result<Vec<u8>, String> {
    let val = |c: u8| -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };
    let clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if clean.is_empty() || !clean.len().is_multiple_of(4) {
        return Err("Invalid Base64 length inside <data> tag".into());
    }
    let mut out = Vec::with_capacity((clean.len() / 4) * 3);
    for chunk in clean.as_chunks::<4>().0 {
        let a = val(chunk[0]).ok_or("Invalid Base64 character")?;
        let b = val(chunk[1]).ok_or("Invalid Base64 character")?;
        out.push((a << 2) | (b >> 4));
        if chunk[2] != b'=' {
            let c = val(chunk[2]).ok_or("Invalid Base64 character")?;
            out.push((b << 4) | (c >> 2));
            if chunk[3] != b'=' {
                let d = val(chunk[3]).ok_or("Invalid Base64 character")?;
                out.push((c << 6) | d);
            }
        }
    }
    Ok(out)
}

pub fn parse_plist_ppt(raw: &[u8]) -> Result<Vec<u8>, String> {
    let txt = String::from_utf8_lossy(raw);
    let key_pos = txt
        .find("PP_PhmSoftPowerPlayTable")
        .ok_or("Could not find PP_PhmSoftPowerPlayTable key in .plist")?;
    let after_key = &txt[key_pos..];
    let data_start = after_key.find("<data>").ok_or("Missing <data> tag after PP_PhmSoftPowerPlayTable")? + 6;
    let data_end = after_key[data_start..].find("</data>").ok_or("Missing </data> closing tag")? + data_start;
    decode_base64_clean(&after_key[data_start..data_end])
}
