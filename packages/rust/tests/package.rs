use quanttide_work::{DOMAIN, VERSION};

#[test]
fn version_matches_manifest() {
    assert_eq!(VERSION, "0.1.0-alpha.5");
}

#[test]
fn domain_name() {
    assert_eq!(DOMAIN, "knowledge-work");
}
