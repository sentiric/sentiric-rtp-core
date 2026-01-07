// build.rs
fn main() {
    let src_path = "deps/bcg729/src";
    let include_path = "deps/bcg729/include";

    // C dosyalarını bul
    let c_files = glob::glob(&format!("{}/*.c", src_path))
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok);

    // cc crate'i ile derle
    let mut build = cc::Build::new();
    
    for file in c_files {
        // Test dosyalarını hariç tut (bazen kaynak kodda testler de olur)
        if !file.to_string_lossy().contains("test") {
            build.file(file);
        }
    }

    build
        .include(include_path)
        .define("BCG729_STATIC", None) // Statik derleme için gerekli flag
        .warnings(false) // C uyarılarını görmezden gel
        .compile("g729"); // libg729.a üretir ve linkler

    println!("cargo:rerun-if-changed={}", src_path);
}