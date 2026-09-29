//! On Windows, the updater's .exe carries Terminal Velocity's icon.

fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=../../assets/tvty.ico");
        println!("cargo:rerun-if-changed=packaging/tvty-updater.rc");
        // Drawn by `cargo run --example windows_icon`, which builds this first.
        if !std::path::Path::new("../../assets/tvty.ico").exists() {
            println!("cargo:warning=no assets/tvty.ico: tvty-updater.exe without its icon");
            return;
        }
        embed_resource::compile("packaging/tvty-updater.rc", embed_resource::NONE).manifest_required().expect("the Windows resources");
    }
}
