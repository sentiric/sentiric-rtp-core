// src/codecs/pcma.rs

use super::{Encoder, Decoder, CodecType};
use super::codec_data::ALAW_TO_LINEAR_LUT; // İsimlendirme doğru

pub struct PcmaEncoder;
pub struct PcmaDecoder;

impl PcmaEncoder {
    // ITU-T G.191 Referans Kodu (TİP HATALARI KESİN OLARAK GİDERİLDİ)
    pub fn linear_to_alaw(pcm_val: i16) -> u8 {
        let sign = (pcm_val >> 8) & 0x80;
        let mut mag = if pcm_val < 0 { -pcm_val } else { pcm_val };

        mag >>= 3;
        if mag > 0xFFF { mag = 0xFFF; } // 12-bit max

        let segment: i32; // Segment i32 olarak doğru
        if mag < 32 { segment = 0; }
        else if mag < 64 { segment = 1; }
        else if mag < 128 { segment = 2; }
        else if mag < 256 { segment = 3; }
        else if mag < 512 { segment = 4; }
        else if mag < 1024 { segment = 5; }
        else if mag < 2048 { segment = 6; }
        else { segment = 7; }

        // DÜZELTME: mantissa hesaplamasının sonucu açıkça i32'ye dönüştürüldü.
        // Bu, derleyicinin yanlış tip çıkarımını engeller.
        let mantissa: i32 = if segment < 2 {
            ((mag >> 1) & 0x0F) as i32 
        } else {
            ((mag >> segment) & 0x0F) as i32
        };
        
        let result = (sign as i32 | (segment << 4) | mantissa) as u8; // mantissa şimdi i32
        result ^ 0x55
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