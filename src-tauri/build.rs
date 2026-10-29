//! Embeds `catalog/*/tool.toml` into the binary, so an installed shell knows
//! its tools without the repository. Cargo re-runs this when catalog/ changes.

use std::fmt::Write;
use std::path::Path;
use std::{env, fs};

fn main() {
    let catalog = Path::new(env!("CARGO_MANIFEST_DIR")).join("../catalog");
    println!("cargo:rerun-if-changed={}", catalog.display());

    let mut entries: Vec<_> = fs::read_dir(&catalog)
        .expect("catalog/ folder is missing")
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("tool.toml"))
        .filter(|p| p.is_file())
        .collect();
    entries.sort();

    let mut code = String::from("pub static EMBEDDED_CATALOG: &[(&str, &str)] = &[\n");
    for path in entries {
        let tool = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy();
        let absolute = fs::canonicalize(&path).unwrap();
        writeln!(
            code,
            "    ({:?}, include_str!({:?})),",
            format!("{tool}/tool.toml"),
            absolute
        )
        .unwrap();
    }
    code.push_str("];\n");

    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("embedded_catalog.rs");
    fs::write(out, code).unwrap();

    tauri_build::build()
}
