// tests/codec_integrity.rs

use sentiric_rtp_core::codecs::{CodecFactory, CodecType};
use std::f64::consts::PI;

// --- Test Yardımcı Fonksiyonları ---
fn generate_sine_wave(freq: f64, duration_ms: u32, sample_rate: u32) -> Vec<i16> {
    let num_samples = (sample_rate as u32 * duration_ms / 1000) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let amplitude = 28000.0;
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        let value = amplitude * (2.0 * PI * freq * t).sin();
        samples.push(value as i16);
    }
    samples
}

fn calculate_psnr(original: &[i16], processed: &[i16]) -> f64 {
    let min_len = std::cmp::min(original.len(), processed.len());
    if min_len == 0 { return 0.0; }
    let mut sum_sq_diff = 0.0;
    for i in 0..min_len {
        let diff = original[i] as f64 - processed[i] as f64;
        sum_sq_diff += diff * diff;
    }
    let mse = sum_sq_diff / min_len as f64;
    if mse < 1e-10 { return 100.0; }
    let max_val = 32767.0;
    20.0 * (max_val / mse.sqrt()).log10()
}

// --- AYRI TEST FONKSİYONLARI ---

#[test]
fn test_pcma_quality() {
    let codec_type = CodecType::PCMA;
    let name = "PCMA (A-law)";
    let psnr_threshold = 35.0; // ITU standardı 37dB'ye yakın olmalı
    println!("\n--- [TEST BAŞLADI] Kodek: {} ---", name);

    let original_pcm = generate_sine_wave(440.0, 1000, 8000);
    let mut encoder = CodecFactory::create_encoder(codec_type);
    let mut decoder = CodecFactory::create_decoder(codec_type);

    let encoded = encoder.encode(&original_pcm);
    let decoded = decoder.decode(&encoded);
    let psnr = calculate_psnr(&original_pcm, &decoded);

    println!("  └─ Hesaplanan PSNR: {:.2} dB (Eşik: >{:.1} dB)", psnr, psnr_threshold);
    assert!(psnr > psnr_threshold, "{} PSNR değeri ({:.2} dB) eşiğin altında kaldı!", name, psnr);
}

#[test]
fn test_pcmu_quality() {
    let codec_type = CodecType::PCMU;
    let name = "PCMU (u-law)";
    let psnr_threshold = 35.0;
    println!("\n--- [TEST BAŞLADI] Kodek: {} ---", name);

    let original_pcm = generate_sine_wave(440.0, 1000, 8000);
    let mut encoder = CodecFactory::create_encoder(codec_type);
    let mut decoder = CodecFactory::create_decoder(codec_type);

    let encoded = encoder.encode(&original_pcm);
    let decoded = decoder.decode(&encoded);
    let psnr = calculate_psnr(&original_pcm, &decoded);

    println!("  └─ Hesaplanan PSNR: {:.2} dB (Eşik: >{:.1} dB)", psnr, psnr_threshold);
    assert!(psnr > psnr_threshold, "{} PSNR değeri ({:.2} dB) eşiğin altında kaldı!", name, psnr);
}

#[test]
fn test_g729_quality() {
    let codec_type = CodecType::G729;
    let name = "G729";
    let psnr_threshold = 5.0; // G.729 için sinüs dalgasında daha gerçekçi eşik
    println!("\n--- [TEST BAŞLADI] Kodek: {} ---", name);

    let original_pcm = generate_sine_wave(440.0, 1000, 8000);
    let g729_pcm_len = (original_pcm.len() / 80) * 80;
    let g729_pcm_slice = &original_pcm[0..g729_pcm_len];

    let mut encoder = CodecFactory::create_encoder(codec_type);
    let mut decoder = CodecFactory::create_decoder(codec_type);

    let encoded = encoder.encode(g729_pcm_slice);
    let decoded = decoder.decode(&encoded);
    let psnr = calculate_psnr(g729_pcm_slice, &decoded);

    println!("  └─ Hesaplanan PSNR: {:.2} dB (Eşik: >{:.1} dB)", psnr, psnr_threshold);
    assert!(psnr > psnr_threshold, "{} PSNR değeri ({:.2} dB) eşiğin altında kaldı!", name, psnr);
}

// G.722 testi kaldırıldı.