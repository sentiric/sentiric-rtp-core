// src/codecs/pcma.rs

use super::{Encoder, Decoder, CodecType};
use super::codec_data::ALAW_TO_LINEAR_LUT;

pub struct PcmaEncoder;
pub struct PcmaDecoder;

impl PcmaEncoder {
    // A-LAW ENCODER (Bitwise Shift Yöntemi - KESİN ÇÖZÜM)
    // 16-bit PCM'i 12-bit A-law aralığına oturtmanın en temiz yolu.
    pub fn linear_to_alaw(pcm_val: i16) -> u8 {
        let mut pcm = pcm_val as i32;
        let mask;

        // 1. İşaret Bitini Yönet (A-law: Pozitif=0xD5, Negatif=0x55)
        if pcm >= 0 {
            mask = 0xD5;
        } else {
            mask = 0x55;
            pcm = -pcm - 8; // Negatif offset
        }

        // 2. Sınırla ve Ölçekle (16-bit -> 12-bit)
        // 16-bit veriyi sağa 4 bit kaydırarak 12-bit (0..4095) aralığına indiriyoruz.
        // Bu işlem, önceki kodlardaki karmaşık matematik hatalarını ortadan kaldırır.
        let val = pcm >> 4;
        let mut effective = if val < 0 { 0 } else { val };
        if effective > 0xFFF { effective = 0xFFF; } // 12-bit max

        // 3. Segment Bulma (G.711 Standart Tablosu)
        let segment: i32;
        let mantissa: i32;

        if effective < 32 {
            segment = 0;
            mantissa = (effective >> 1) & 0x0F;
        } else if effective < 64 {
            segment = 1;
            mantissa = (effective >> 2) & 0x0F;
        } else if effective < 128 {
            segment = 2;
            mantissa = (effective >> 3) & 0x0F;
        } else if effective < 256 {
            segment = 3;
            mantissa = (effective >> 4) & 0x0F;
        } else if effective < 512 {
            segment = 4;
            mantissa = (effective >> 5) & 0x0F;
        } else if effective < 1024 {
            segment = 5;
            mantissa = (effective >> 6) & 0x0F;
        } else if effective < 2048 {
            segment = 6;
            mantissa = (effective >> 7) & 0x0F;
        } else {
            segment = 7;
            mantissa = (effective >> 8) & 0x0F;
        }

        // 4. Birleştir ve Maskele
        let result = ((segment << 4) | mantissa) as u8;
        result ^ mask
    }
}

unsafe impl Send for PcmaEncoder {}
impl Encoder for PcmaEncoder {
    fn get_type(&self) -> CodecType { CodecType::PCMA }
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        pcm_samples.iter().map(|&s| Self::linear_to_alaw(s)).collect()
    }
}

unsafe impl Send for PcmaDecoder {}
impl Decoder for PcmaDecoder {
    fn get_type(&self) -> CodecType { CodecType::PCMA }
    fn decode(&mut self, payload: &[u8]) -> Vec<i16> {
        payload.iter().map(|&b| ALAW_TO_LINEAR_LUT[b as usize]).collect()
    }
}