// sentiric-rtp-core/src/codecs/g711.rs

use super::{Encoder, Decoder, CodecType};

pub struct G711 {
    codec_type: CodecType,
}

impl G711 {
    pub fn new(codec_type: CodecType) -> Self {
        G711 { codec_type }
    }

    /// LINEAR (PCM) -> ALAW (G.711a)
    /// Standart ITU-T G.711 A-law sıkıştırma algoritması.
    /// Referans: SPANDSP / ITU-T G.711
    pub fn linear_to_alaw(pcm_val: i16) -> u8 {
        let mask = 0xD5; // A-law XOR maskesi
        
        // İşlem sırasında i16 sınırlarını aşmamak ve uyarı almamak için i32'ye cast ediyoruz.
        let mut pcm = pcm_val as i32;

        // 1. İşaret (Sign) bitini al ve mutlak değere çevir
        let sign = if pcm < 0 {
            pcm = -pcm - 1; // 1's complement'e yakınsama
            0x00 // Negatif için bit 0 (XOR sonrası ters dönecek)
        } else {
            0x80 // Pozitif için bit 1
        };

        // 2. Değeri sınırla (Clip) - Artık i32 olduğu için bu kontrol anlamlıdır.
        if pcm > 32767 { pcm = 32767; }

        // 3. Segmenti (Exponent) bul
        let exponent: u8;
        let mantissa: u8;

        if pcm < 256 {
            exponent = 0;
            mantissa = ((pcm >> 4) & 0x0F) as u8;
        } else if pcm < 512 {
            exponent = 1;
            mantissa = ((pcm >> 5) & 0x0F) as u8;
        } else if pcm < 1024 {
            exponent = 2;
            mantissa = ((pcm >> 6) & 0x0F) as u8;
        } else if pcm < 2048 {
            exponent = 3;
            mantissa = ((pcm >> 7) & 0x0F) as u8;
        } else if pcm < 4096 {
            exponent = 4;
            mantissa = ((pcm >> 8) & 0x0F) as u8;
        } else if pcm < 8192 {
            exponent = 5;
            mantissa = ((pcm >> 9) & 0x0F) as u8;
        } else if pcm < 16384 {
            exponent = 6;
            mantissa = ((pcm >> 10) & 0x0F) as u8;
        } else {
            exponent = 7;
            mantissa = ((pcm >> 11) & 0x0F) as u8;
        }

        // 4. Paketi birleştir: [Sign | Exponent | Mantissa]
        let val = sign | (exponent << 4) | mantissa;

        // 5. A-Law için bitleri ters çevir (XOR)
        val ^ mask
    }

    /// ALAW (G.711a) -> LINEAR (PCM)
    pub fn alaw_to_linear(alaw_val: u8) -> i16 {
        // 1. Maskeyi geri al
        let val = alaw_val ^ 0xD5;
        
        let sign = val & 0x80;
        let exponent = (val >> 4) & 0x07;
        let mantissa = val & 0x0F;

        // 2. Segmenti aç
        let mut t = if exponent == 0 {
            (mantissa as i32) << 4
        } else {
            ((mantissa as i32) << (exponent + 3)) | (0x100 << (exponent - 1))
        };
        
        // 3. Segment ortasına konumlandır (Quantization noise azaltma)
        t += 8;
        
        // 4. İşaret bitini uygula
        if sign == 0 { -t as i16 } else { t as i16 }
    }

    /// LINEAR (PCM) -> ULAW (G.711u)
    pub fn linear_to_ulaw(pcm_val: i16) -> u8 {
        let bias = 0x84;
        let mut pcm = pcm_val as i32; // i32 kullanımı burada da önemli
        let sign = if pcm < 0 { 
            pcm = -pcm;
            0x80 
        } else { 
            0x00 
        };

        if pcm > 32635 { pcm = 32635; }
        pcm += bias;

        let exponent: u8;
        // u-law exponent tablosu yerine mantıksal hesaplama
        if pcm < 0x100 { exponent = 0; }
        else if pcm < 0x200 { exponent = 1; }
        else if pcm < 0x400 { exponent = 2; }
        else if pcm < 0x800 { exponent = 3; }
        else if pcm < 0x1000 { exponent = 4; }
        else if pcm < 0x2000 { exponent = 5; }
        else if pcm < 0x4000 { exponent = 6; }
        else { exponent = 7; }

        let mantissa = ((pcm >> (exponent + 3)) & 0x0F) as u8;
        let val = sign | (exponent << 4) | mantissa;
        !val // u-law bitleri ters çevirir
    }

    /// ULAW (G.711u) -> LINEAR (PCM)
    pub fn ulaw_to_linear(ulaw_val: u8) -> i16 {
        let val = !ulaw_val;
        let sign = val & 0x80;
        let exponent = (val >> 4) & 0x07;
        let mantissa = val & 0x0F;

        let mut t = (((mantissa as i32) << 3) + 0x84) << exponent;
        t -= 0x84;

        if sign != 0 { -t as i16 } else { t as i16 }
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