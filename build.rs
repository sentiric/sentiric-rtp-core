use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = PathBuf::from(manifest_dir);
    let bcg729_path = root.join("deps/bcg729");
    let bcg729_src = bcg729_path.join("src");
    let bcg729_include = bcg729_path.join("include");

    // 1. Submodule Kontrolü (Eğer boşsa init et)
    if !bcg729_src.exists() {
        // Build sırasında uyarı ver ve indirmeyi dene
        println!("cargo:warning=bcg729 submodule içeriği boş. İndiriliyor...");
        let status = Command::new("git")
            .args(&["submodule", "update", "--init", "--recursive"])
            .current_dir(&root)
            .status();

        if status.is_err() || !status.unwrap().success() {
            panic!("KRİTİK HATA: 'deps/bcg729' submodule indirilemedi. Lütfen 'git submodule update --init --recursive' komutunu çalıştırın.");
        }
    }

    // 2. C Dosyalarını Bul
    let c_files = glob::glob(&format!("{}/*.c", bcg729_src.display()))
        .expect("Glob pattern hatası")
        .filter_map(Result::ok)
        .filter(|path| !path.to_string_lossy().contains("test")); // Test dosyalarını hariç tut

    let mut build = cc::Build::new();
    let mut file_count = 0;

    for file in c_files {
        build.file(file);
        file_count += 1;
    }

    if file_count == 0 {
        panic!("HATA: bcg729 kaynak kodları bulunamadı! Yol: {}", bcg729_src.display());
    }

    // 3. Derleme (Static Linking)
    build
        .include(bcg729_include)
        .define("BCG729_STATIC", None)
        .warnings(false) // C kütüphanesinin uyarılarını konsola basma
        .compile("g729");

    // Linkleme
    println!("cargo:rustc-link-lib=static=g729");
    println!("cargo:rerun-if-changed=deps/bcg729");
}