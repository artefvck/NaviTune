pub fn encode_base64(data: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;
        out.push(CHARS[b0 >> 2] as char);
        out.push(CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[b2 & 0x3f] as char);
        } else {
            out.push('=');
        }
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

pub fn format_mpt_ppt(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 0x100 + data.len()];
    out[0] = 0x55;
    out[1] = 0xAA;
    out[0x18] = 0x3C;
    out[0x100..].copy_from_slice(data);
    out
}

pub fn format_opencore_plist(data: &[u8], pci_path: &str) -> String {
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
				<key>PP_PhmSoftPowerPlayTable</key>
				<data>
{}				</data>
			</dict>
		</dict>
	</dict>
</dict>
</plist>
"#,
        pci_path, wrapped
    )
}
