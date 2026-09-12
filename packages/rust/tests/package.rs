use quanttide_work::{DOMAIN, VERSION};

/// 版本常量与清单同源。`VERSION` 由 `CARGO_PKG_VERSION` 注入，这里再对着清单核一遍，
/// 免得有人把它改回手写字面量（同一件事两处写就会错开）。
#[test]
fn version_matches_manifest() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("读得到 Cargo.toml");
    let declared = manifest
        .lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|rest| rest.split('"').next())
        .expect("Cargo.toml 里有 version");
    assert_eq!(VERSION, declared);
}

#[test]
fn domain_name() {
    assert_eq!(DOMAIN, "knowledge-work");
}
