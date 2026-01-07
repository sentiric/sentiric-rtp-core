use std::env;
use std::path::PathBuf;

fn main() {
    // Projenin kök dizinini al
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = PathBuf::from(manifest_dir);

    // bcg729 kaynak kodlarının yolu
    let bcg729_src = root.join("deps/bcg729/src");
    let bcg729_include = root.join("deps/bcg729/include");

    // Kaynak kod klasörünün varlığını kontrol et (Build hatası vermeden önce uyar)
    if !bcg729_src.exists() {
        panic!("bcg729 kaynak kodları bulunamadı! 'deps/bcg729' klasörünün dolu olduğundan emin olun. (git submodule update --init --recursive)");
    }

    // C dosyalarını bul
    let c_files = glob::glob(&format!("{}/*.c", bcg729_src.display()))
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok);

    let mut build = cc::Build::new();
    
    // Dosyaları ekle
    let mut file_count = 0;
    for file in c_files {
        // Test dosyalarını hariç tut
        if !file.to_string_lossy().contains("test") {
            build.file(file);
            file_count += 1;
        }
    }

    if file_count == 0 {
        panic!("Hiçbir C dosyası bulunamadı! Yol: {}", bcg729_src.display());
    }

    // Derleme
    build
        .include(bcg729_include)
        .define("BCG729_STATIC", None)
        .warnings(false)
        .compile("g729"); // libg729.a üretir

    // Değişiklik izleme
    println!("cargo:rerun-if-changed={}", bcg729_src.display());
}