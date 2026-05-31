fn main() {
    #[cfg(feature = "desktop")]
    {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let dist = manifest_dir.join("dist");
        let index = dist.join("index.html");

        println!("cargo:rerun-if-changed=tauri.conf.json");
        println!("cargo:rerun-if-changed=permissions");
        println!("cargo:rerun-if-changed=capabilities");
        println!("cargo:rerun-if-changed=icons");

        if index.is_file() {
            println!("cargo:rerun-if-changed={}", index.display());
            if let Ok(entries) = std::fs::read_dir(dist.join("assets")) {
                for entry in entries.flatten() {
                    println!("cargo:rerun-if-changed={}", entry.path().display());
                }
            }
        } else {
            eprintln!(
                "cargo:warning=dist/index.html 不存在，请先运行: cd frontend && npm run build"
            );
        }

        tauri_build::build()
    }
}
