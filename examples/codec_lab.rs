// examples/codec_lab.rs
use sentiric_rtp_core::codecs::{CodecFactory, CodecType};
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn calculate_psnr(original: &[i16], processed: &[i16]) -> f64 {
    let min_len = std::cmp::min(original.len(), processed.len());
    if min_len == 0 {
        return 0.0;
    }
    let mut sum_sq_diff = 0.0;

    for i in 0..min_len {
        let diff = original[i] as f64 - processed[i] as f64;
        sum_sq_diff += diff * diff;
    }

    let mse = sum_sq_diff / min_len as f64;
    if mse == 0.0 {
        return 100.0; // Mükemmel eşleşme
    }

    let max_val = 32767.0; // 16-bit PCM için tepe değer
    20.0 * (max_val / mse.sqrt()).log10()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Kullanım: cargo run --example codec_lab -- <input.wav> <codec>");
        eprintln!("Desteklenen codec'ler: pcma, pcmu, g729"); // G722 kaldırıldı
        std::process::exit(1);
    }

    let input_path = &args[1];
    let codec_str = &args[2].to_lowercase();

    let codec_type = match codec_str.as_str() {
        "pcma" => CodecType::PCMA,
        "pcmu" => CodecType::PCMU,
        "g729" => CodecType::G729,
        // "g722" => CodecType::G722, // Kaldırıldı
        _ => {
            eprintln!("Hata: Geçersiz veya desteklenmeyen codec '{}'", codec_str);
            std::process::exit(1);
        }
    };

    println!("🔬 Codec Laboratuvarı Başlatıldı");
    println!("├─ Dosya: {}", input_path);
    println!("└─ Kodek: {:?}", codec_type);

    let reader = hound::WavReader::open(input_path).expect("WAV dosyası okunamadı.");
    let spec = reader.spec();

    if spec.channels != 1
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
    {
        eprintln!("Hata: WAV dosyası 16-bit, mono, PCM formatında olmalıdır.");
        std::process::exit(1);
    }

    // G.729, PCMA, PCMU hepsi 8000Hz ister
    if spec.sample_rate != 8000 {
        eprintln!(
            "Uyarı: Bu kodek için 8000 Hz örnekleme hızı bekleniyor. Girdi: {} Hz",
            spec.sample_rate
        );
    }

    let original_samples: Vec<i16> = reader
        .into_samples::<i16>()
        .collect::<Result<_, _>>()
        .unwrap();

    let mut encoder = CodecFactory::create_encoder(codec_type);
    let mut decoder = CodecFactory::create_decoder(codec_type);

    let encoded_payload = encoder.encode(&original_samples);
    let decoded_samples = decoder.decode(&encoded_payload);

    let psnr = calculate_psnr(&original_samples, &decoded_samples);
    let original_size = original_samples.len() * 2; // 16 bit = 2 byte
    let encoded_size = encoded_payload.len();
    let compression_ratio = original_size as f64 / encoded_size as f64;

    println!("\n📊 Analiz Sonuçları:");
    println!("├─ Orijinal Boyut   : {} bytes", original_size);
    println!("├─ Sıkıştırılmış Boyut: {} bytes", encoded_size);
    println!("├─ Sıkıştırma Oranı  : {:.2}:1", compression_ratio);
    println!("└─ Kalite (PSNR)    : {:.2} dB", psnr);

    let output_filename = format!("output_{}.wav", codec_str);
    let out_spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8000, // Artık sadece 8k kodekler var
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let path = Path::new(&output_filename);
    let file = File::create(path).expect("Çıktı dosyası oluşturulamadı.");
    let writer = BufWriter::new(file);
    let mut wav_writer = hound::WavWriter::new(writer, out_spec).unwrap();

    for sample in decoded_samples {
        wav_writer.write_sample(sample).unwrap();
    }
    wav_writer.finalize().unwrap();

    println!(
        "\n✅ Başarılı: İşlenmiş ses '{}' dosyasına kaydedildi.",
        output_filename
    );
}
