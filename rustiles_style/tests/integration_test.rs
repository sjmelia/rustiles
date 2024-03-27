use std::fs;
use std::path::PathBuf;
use rustiles_style::Style;

#[test]
fn it_deserializes_cwm() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources");
    d.push("test");
    d.push("maps.clockworkmicro.com.json");
    println!("{}", d.display());
    let data = fs::read_to_string(d).expect("Unable to read file");
    let style: Style = serde_json::from_str(&data).expect("Unable to read JSON");
    assert_eq!(style.name, "Bright New");
}