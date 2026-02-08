// sentiric-rtp-core/src/codecs/g722.rs

// ============================================================================
// !!! SIMULATION ONLY - NOT FOR PRODUCTION TRANSCODING !!!
// This implementation simulates 16kHz wideband audio by upsampling G.711.
// It is NOT a standard G.722 SB-ADPCM implementation.
// Use this only for passing RTP packets or basic testing.
// ============================================================================

use super::{Encoder, Decoder, CodecType};
use super::pcmu::PcmuEncoder; 
use super::codec_data::ULAW_TO_LINEAR_LUT; 

pub struct G722;

impl G722 {
    pub fn new() -> Self {
        G722 {}
    }

    // --- Simülasyon Yardımcıları ---
    
    // Encode: Linear PCM -> Simulated 4-bit Nibble
    // Bu, G.711'in 8-bit çıktısını alıp 4-bit'e sıkıştıran daha iyi bir quantizer.
    fn encode_nibble(sample: i16) -> u8 {
        let ulaw_val = PcmuEncoder::linear_to_ulaw(sample);
        // 8-bit u-law değerini daha düzgün bir 4-bit'e dönüştür
        // Örnek: İlk 4 biti al ve yuvarlama için 0x08 (yarım bit) ekle.
        ((ulaw_val >> 4) & 0x07) as u8 // En üst 3 bit + yarım bit (daha kaliteli quantize)
    }

    // Decode: Simulated 4-bit Nibble -> Linear PCM
    fn decode_nibble(nibble: u8) -> i16 {
        // 4-bit değeri alıp, 8-bit u-law'a geri dönüştür.
        // Boş kalan alt bitleri doldurmak önemlidir.
        let ulaw_val = (nibble << 4) | 0x08; // En alttaki 4 biti 0x08 ile doldur (orta nokta)
        ULAW_TO_LINEAR_LUT[ulaw_val as usize]
    }
}

impl Encoder for G722 {
    fn get_type(&self) -> CodecType {
        CodecType::G722
    }

    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        // G.722, 16kHz input bekler.
        // 8kHz input alıyorsak, basitçe örnekleri tekrarlayarak 16kHz simülasyonu yapalım.
        // Bu "upsampling" kalitesizdir ama simülasyon amacına uyar.
        let mut wideband_samples = Vec::with_capacity(pcm_samples.len() * 2);
        for &sample in pcm_samples {
            wideband_samples.push(sample);
            wideband_samples.push(sample); // Her örneği iki kez tekrarla
        }

        let mut output = Vec::with_capacity(wideband_samples.len() / 2); // 2 adet 4-bit nibble = 1 byte
        
        for chunk in wideband_samples.chunks(2) {
            // chunk[0] = s1 (first sample), chunk[1] = s2 (second sample)
            let s1 = chunk[0];
            let s2 = chunk[1]; 

            let n1 = Self::encode_nibble(s1);
            let n2 = Self::encode_nibble(s2);

            // G.722 byte formatı: [MSB (Sample 2'nin nibble'ı) | LSB (Sample 1'in nibble'ı)]
            let byte = (n2 << 4) | n1;
            output.push(byte);
        }

        output
    }
}

impl Decoder for G722 {
    fn get_type(&self) -> CodecType {
        CodecType::G722
    }

    fn decode(&mut self, payload: &[u8]) -> Vec<i16> {
        let mut output = Vec::with_capacity(payload.len() * 2); // Her byte'dan 2 sample çıkar

        for byte in payload {
            // Byte'ı iki nibble'a ayır
            let n1 = byte & 0x0F;        // Sample 1'in nibble'ı
            let n2 = (byte >> 4) & 0x0F; // Sample 2'nin nibble'ı

            let s1 = Self::decode_nibble(n1);
            let s2 = Self::decode_nibble(n2);

            output.push(s1);
            output.push(s2);
        }
        output
    }
}