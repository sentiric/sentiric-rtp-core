// examples/codec_inspector.rs
// Bu araç, kodeklerin matematiksel davranışını satır satır inceler.

use sentiric_rtp_core::codecs::{CodecFactory, CodecType};

fn main() {
    let test_values: Vec<i16> = vec![
        0, 1, 10, 100, 1000, 10000, 20000, 32000, // Pozitifler
        -1, -10, -100, -1000, -10000, -20000, -32000 // Negatifler
    ];

    inspect_codec(CodecType::PCMA, "PCMA (A-law)", &test_values);
    inspect_codec(CodecType::PCMU, "PCMU (u-law)", &test_values);
    // G729 lossy olduğu için birebir eşleşme beklenmez, onu atlıyoruz.
}

fn inspect_codec(codec_type: CodecType, name: &str, values: &[i16]) {
    println!("\n🔎 KODEK İNCELEMESİ: {}", name);
    println!("{:<10} | {:<10} | {:<10} | {:<10}", "Girdi", "Hex Kod", "Çıktı", "Fark");
    println!("{}", "-".repeat(50));

    let mut encoder = CodecFactory::create_encoder(codec_type);
    let mut decoder = CodecFactory::create_decoder(codec_type);

    for &val in values {
        let input_slice = [val];
        let encoded = encoder.encode(&input_slice);
        
        if encoded.is_empty() {
            println!("{:<10} | {:<10} | HATA", val, "BOŞ");
            continue;
        }

        let code_byte = encoded[0];
        let decoded = decoder.decode(&encoded);
        let output_val = decoded[0];
        let diff = output_val as i32 - val as i32;

        println!("{:<10} | 0x{:<02X}       | {:<10} | {:<10}", 
            val, code_byte, output_val, diff);
    }
}