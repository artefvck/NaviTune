fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    println!("cargo:rerun-if-changed=assets/navitune.manifest");
    println!("cargo:rerun-if-changed=assets/navitune.rc");

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    if target_env == "gnu" {
        let res_obj = out_dir.join("navitune_manifest.o");
        let status = std::process::Command::new("windres")
            .args(["assets/navitune.rc", "-O", "coff", "-o"])
            .arg(&res_obj)
            .status();
        if let Ok(st) = status {
            if st.success() {
                println!("cargo:rustc-link-arg={}", res_obj.display());
            }
        }
    } else {
        let res_lib = out_dir.join("navitune_manifest.res");
        let status = std::process::Command::new("rc.exe")
            .args(["/nologo", "/fo"])
            .arg(&res_lib)
            .arg("assets/navitune.rc")
            .status();
        if let Ok(st) = status {
            if st.success() {
                println!("cargo:rustc-link-arg={}", res_lib.display());
            }
        }
    }
}
