// sentiric-rtp-core/src/codecs/g711.rs

use super::{Encoder, Decoder, CodecType};

pub struct G711 {
    codec_type: CodecType,
}

impl G711 {
    pub fn new(codec_type: CodecType) -> Self {
        G711 { codec_type }
    }

    // Encoder fonksiyonları (linear_to_alaw/ulaw) burada mevcut kalacak...
    // ... (Önceki kodun aynısı linear_to_alaw vb.) ...
    // ANTI-LAZY: Tam kod veriliyor.
    
    pub fn linear_to_alaw(pcm_val: i16) -> u8 {
        let mask = 0x55;
        let mut pcm: i32 = pcm_val as i32;
        let sign = if pcm >= 0 { 0xD5 } else { pcm = -pcm - 8; 0x55 };
        if pcm > 32767 { pcm = 32767; }
        if pcm < 0 { pcm = 0; }
        let exponent: i32;
        let mantissa: i32;
        if pcm < 256 {
            exponent = 0;
            mantissa = (pcm >> 4) & 0x0F;
        } else {
            let mut seg = 1;
            let mut check = pcm >> 8;
            if check >= 16 { check >>= 4; seg += 4; }
            if check >= 4  { check >>= 2; seg += 2; }
            if check >= 2  { seg += 1; }
            exponent = seg;
            mantissa = (pcm >> (exponent + 4)) & 0x0F;
        }
        let alaw_byte = sign | ((exponent << 4) as u8) | (mantissa as u8);
        alaw_byte ^ mask
    }

    pub fn linear_to_ulaw(pcm_val: i16) -> u8 {
        let bias = 0x84;
        let clip = 32635;
        let sign = if pcm_val < 0 { 0x80 } else { 0x00 };
        let mut pcm: i32 = pcm_val as i32;
        if pcm < 0 { pcm = -pcm; }
        if pcm > clip { pcm = clip; }
        pcm += bias;
        let exponent = if pcm < 0x100 { 0 } else if pcm < 0x200 { 1 } else if pcm < 0x400 { 2 } else if pcm < 0x800 { 3 } else if pcm < 0x1000 { 4 } else if pcm < 0x2000 { 5 } else if pcm < 0x4000 { 6 } else { 7 };
        let mantissa = (pcm >> (exponent + 3)) & 0x0F;
        !(sign | ((exponent as u8) << 4) | (mantissa as u8))
    }

    // --- DECODERS ---
    pub fn alaw_to_linear(a_val: u8) -> i16 {
        let t = a_val ^ 0x55;
        let mut t16 = ((t & 0xf) as i16) << 4;
        t16 += 8;
        if (t & 0x70) != 0 {
            t16 += 0x100;
            let shift = ((t & 0x70) >> 4) - 1;
            t16 <<= shift;
        }
        if (t & 0x80) == 0 { -t16 } else { t16 }
    }

    pub fn ulaw_to_linear(u_val: u8) -> i16 {
        let t = !u_val;
        let mut t16 = ((t & 0xf) as i16) << 3;
        t16 += 0x84;
        t16 <<= (t & 0x70) >> 4;
        t16 -= 0x84;
        if (t & 0x80) == 0 { -t16 } else { t16 }
    }
}

impl Encoder for G711 {
    fn get_type(&self) -> CodecType { self.codec_type }
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        match self.codec_type {
            CodecType::PCMA => pcm_samples.iter().map(|&s| Self::linear_to_alaw(s)).collect(),
            CodecType::PCMU => pcm_samples.iter().map(|&s| Self::linear_to_ulaw(s)).collect(),
            _ => vec![],
        }
    }
}

// YENİ: Decoder Implementasyonu
impl Decoder for G711 {
    fn get_type(&self) -> CodecType { self.codec_type }
    fn decode(&mut self, payload: &[u8]) -> Vec<i16> {
        match self.codec_type {
            CodecType::PCMA => payload.iter().map(|&b| Self::alaw_to_linear(b)).collect(),
            CodecType::PCMU => payload.iter().map(|&b| Self::ulaw_to_linear(b)).collect(),
            _ => vec![],
        }
    }
}