// sentiric-rtp-core/src/codecs/g711.rs

use super::{Encoder, CodecType};

pub struct G711 {
    codec_type: CodecType,
}

impl G711 {
    pub fn new(codec_type: CodecType) -> Self {
        G711 { codec_type }
    }

    /// SPANDSP A-Law Encoder (Final Clean Version)
    fn linear_to_alaw(pcm_val: i16) -> u8 {
        let mask = 0x55;
        // i32 ile güvenli hesaplama
        let mut pcm = pcm_val as i32;

        let sign = if pcm >= 0 {
            0xD5
        } else {
            pcm = -pcm - 8;
            0x55
        };

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

        // DÜZELTME: Gereksiz parantezler kaldırıldı
        let alaw_byte = sign | ((exponent << 4) as u8) | (mantissa as u8);
        alaw_byte ^ mask
    }

    /// SPANDSP u-Law Encoder
    fn linear_to_ulaw(pcm_val: i16) -> u8 {
        let bias = 0x84;
        let clip = 32635;
        let sign = if pcm_val < 0 { 0x80 } else { 0x00 };
        
        let mut pcm = pcm_val as i32;
        if pcm < 0 { pcm = -pcm; }
        if pcm > clip { pcm = clip; }
        
        pcm += bias;

        let exponent = if pcm < 0x100 { 0 }
        else if pcm < 0x200 { 1 }
        else if pcm < 0x400 { 2 }
        else if pcm < 0x800 { 3 }
        else if pcm < 0x1000 { 4 }
        else if pcm < 0x2000 { 5 }
        else if pcm < 0x4000 { 6 }
        else { 7 };

        let mantissa = (pcm >> (exponent + 3)) & 0x0F;
        // u-Law mantığında ! (NOT) operatörü tüm ifadeyi kapsar, parantez gereklidir.
        let ulaw_byte = !(sign | ((exponent as u8) << 4) | (mantissa as u8));
        
        ulaw_byte
    }
}

impl Encoder for G711 {
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        match self.codec_type {
            CodecType::PCMA => {
                pcm_samples.iter()
                    .map(|&sample| {
                        // VOLÜM KONTROLÜ: %50
                        // Bu ayar sesin patlamasını engeller.
                        let soft_sample = (sample as f32 * 0.50) as i16;
                        Self::linear_to_alaw(soft_sample)
                    })
                    .collect()
            },
            CodecType::PCMU => {
                pcm_samples.iter()
                    .map(|&sample| {
                        let soft_sample = (sample as f32 * 0.50) as i16;
                        Self::linear_to_ulaw(soft_sample)
                    })
                    .collect()
            },
            _ => vec![], 
        }
    }
}