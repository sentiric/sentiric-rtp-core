// sentiric-rtp-core/src/codecs/g722.rs

use super::{Encoder, Decoder, CodecType};

pub struct G722;

impl G722 {
    pub fn new() -> Self {
        G722 {}
    }

    // --- Simülasyon Yardımcıları ---
    
    // Encode: Linear -> Simulated ADPCM Nibble (4-bit)
    fn encode_nibble(sample: i16) -> u8 {
        let ulaw = crate::codecs::g711::G711::linear_to_ulaw(sample);
        (ulaw >> 4) & 0x0F
    }

    // Decode: Simulated ADPCM Nibble (4-bit) -> Linear
    fn decode_nibble(nibble: u8) -> i16 {
        // 4 bit veriyi alıp, u-law'ın en anlamlı 4 bitiymiş gibi yukarı kaydırıyoruz.
        // Geri kalan alt bitleri 0 yerine 1000... (orta değer) ile doldurmak sesi biraz yumuşatır.
        let ulaw = (nibble << 4) | 0x08; 
        crate::codecs::g711::G711::ulaw_to_linear(ulaw)
    }
}

impl Encoder for G722 {
    fn get_type(&self) -> CodecType {
        CodecType::G722
    }

    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        // G.722, 16kHz input bekler. Basit upsampling (Linear Interpolation)
        let mut wideband = Vec::with_capacity(pcm_samples.len() * 2);
        
        for i in 0..pcm_samples.len() {
            wideband.push(pcm_samples[i]);
            if i + 1 < pcm_samples.len() {
                let val = (pcm_samples[i] as i32 + pcm_samples[i+1] as i32) / 2;
                wideband.push(val as i16);
            } else {
                wideband.push(pcm_samples[i]);
            }
        }

        let mut output = Vec::with_capacity(wideband.len() / 2);
        
        for chunk in wideband.chunks(2) {
            if chunk.len() < 2 { break; }
            let s1 = chunk[0];
            let s2 = chunk[1];

            let n1 = Self::encode_nibble(s1);
            let n2 = Self::encode_nibble(s2);

            // G.722 Byte yapısı: [Sample 2 (4 bit) | Sample 1 (4 bit)]
            let byte = (n2 << 4) | n1;
            output.push(byte);
        }

        output
    }
}

// YENİ: Decoder Implementasyonu
impl Decoder for G722 {
    fn get_type(&self) -> CodecType {
        CodecType::G722
    }

    fn decode(&mut self, payload: &[u8]) -> Vec<i16> {
        // G.722 çıkışı 16kHz'dir.
        let mut output = Vec::with_capacity(payload.len() * 2);

        for byte in payload {
            // Byte: [High Nibble (Sample 2) | Low Nibble (Sample 1)]
            let n1 = byte & 0x0F;
            let n2 = (byte >> 4) & 0x0F;

            let s1 = Self::decode_nibble(n1);
            let s2 = Self::decode_nibble(n2);

            output.push(s1);
            output.push(s2);
        }
        
        // Downsampling (16kHz -> 8kHz) yapılmalı mı?
        // CodecType::G722 sample_rate() 16000 döndüğü için, 
        // çağıran taraf (media service) bunu bekler. Olduğu gibi 16k döndürüyoruz.
        output
    }
}