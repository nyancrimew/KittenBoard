use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

// embed the emoji data into the binary as a pre-populated vec
fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("emoji_data.rs");

    let data_json =
        fs::read_to_string("assets/emoji-en-US.json").expect("couldn't read emoji data json");
    let data: HashMap<String, Vec<String>> =
        serde_json::from_str(data_json.as_str()).expect("failed to deserialize emoji data");

    let entries: Vec<String> = data
        .iter()
        .map(|f| {
            let values: Vec<String> = f.1.iter().map(|v| format!("\"{}\"", v)).collect();
            format!("(\"{}\", vec![{}])", f.0, values.join(", "))
        })
        .collect();

    fs::write(
        &dest_path,
        format!(
            "use lazy_static::lazy_static;

lazy_static! {{
    // emoji data (grabbed from https://github.com/muan/emojilib)
    pub static ref EMOJI_DATA: [(&'static str, Vec<&'static str>); {}] = [{}];
}}",
            entries.len(),
            entries.join(",\n        ")
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/emoji-en-US.json");
}
