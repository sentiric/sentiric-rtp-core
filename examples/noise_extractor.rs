// examples/noise_extractor.rs
use hound;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Kullanım: cargo run --example noise_extractor -- <orijinal.wav> <islenmis.wav> <cikis_gurultu.wav>");
        std::process::exit(1);
    }

    let orig_path = &args[1];
    let proc_path = &args[2];
    let out_path = &args[3];

    println!("🔬 Null Test (Faz Tersleme) Analizi Başlatılıyor...");

    let mut orig_reader = hound::WavReader::open(orig_path).expect("Orijinal dosya açılamadı");
    let mut proc_reader = hound::WavReader::open(proc_path).expect("İşlenmiş dosya açılamadı");

    let orig_spec = orig_reader.spec();

    // Sesleri belleğe al
    let orig_samples: Vec<i16> = orig_reader.samples::<i16>().map(|s| s.unwrap()).collect();
    let proc_samples: Vec<i16> = proc_reader.samples::<i16>().map(|s| s.unwrap()).collect();

    let min_len = std::cmp::min(orig_samples.len(), proc_samples.len());

    let mut noise_writer =
        hound::WavWriter::create(out_path, orig_spec).expect("Çıktı dosyası oluşturulamadı");

    let mut max_noise = 0;
    let mut total_noise: i64 = 0;

    // Her bir örneği birbirinden çıkar
    for i in 0..min_len {
        // Orijinal - İşlenmiş = Sadece Gürültü (Fark)
        let diff = orig_samples[i] as i32 - proc_samples[i] as i32;

        if diff.abs() > max_noise {
            max_noise = diff.abs();
        }
        total_noise += diff.abs() as i64;

        // Gürültüyü kulakla rahat duyabilmek için genliğini 10 kat (20dB) artırıyoruz.
        // Eğer clipping (taşma) olursa diye clamp uyguluyoruz.
        let amplified_noise = (diff * 10).clamp(-32768, 32767) as i16;

        noise_writer.write_sample(amplified_noise).unwrap();
    }

    let avg_noise = total_noise as f64 / min_len as f64;

    noise_writer.finalize().unwrap();

    println!("✅ Analiz Tamamlandı!");
    println!(
        "├─ Ortalama Gürültü Sapması: {:.2} (Maks 32767 üzerinden)",
        avg_noise
    );
    println!("├─ Maksimum Gürültü Zirvesi: {}", max_noise);
    println!(
        "└─ Sadece gürültüyü içeren dosya '{}' olarak kaydedildi.",
        out_path
    );
    println!("\n🎧 LÜTFEN ŞİMDİ BU DOSYAYI DİNLEYİN. Sadece cızırtıyı duyacaksınız.");
}
