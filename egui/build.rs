//! tvty's words, the GPUI build's own (`../assets/locales/<lang>/*.ftl`),
//! built in as egui-pop catalogues: `CATALOGUES` in `$OUT_DIR/locales.rs`.

use std::fmt::Write as _;
use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/locales");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut entries = Vec::new();
    let mut langs: Vec<_> = std::fs::read_dir(&root).expect("assets/locales").flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    langs.sort();
    for lang in langs {
        let code = lang.file_name().unwrap().to_string_lossy().to_string();
        let mut files: Vec<_> = std::fs::read_dir(&lang).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "ftl")).collect();
        files.sort();
        for file in files {
            println!("cargo:rerun-if-changed={}", file.display());
            let name = file.file_name().unwrap().to_string_lossy().to_string();
            let path = file.canonicalize().unwrap();
            entries.push(format!("    Catalogue::new({code:?}, {name:?}, include_str!({:?})),", path.display().to_string()));
        }
    }
    let mut out = String::from("pub static CATALOGUES: &[Catalogue] = &[\n");
    for e in entries {
        writeln!(out, "{e}").unwrap();
    }
    out.push_str("];\n");
    std::fs::write(Path::new(&std::env::var("OUT_DIR").unwrap()).join("locales.rs"), out).unwrap();
}
