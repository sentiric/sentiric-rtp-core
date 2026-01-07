// src/codecs/g711.rs

use super::{Encoder, CodecType};

pub struct G711 {
    #[allow(dead_code)]
    codec_type: CodecType,
}

impl G711 {
    pub fn new(codec_type: CodecType) -> Self {
        G711 { codec_type }
    }

    // Basit PCMA/PCMU sıkıştırma simülasyonu (Linear -> A-Law/u-Law)
    // Gerçek G.711 tablosu daha sonra eklenebilir.
    fn compress_sample(pcm: i16) -> u8 {
        // En basit yöntem: 16 bitlik verinin en anlamlı 8 bitini al.
        // Bu ses kalitesini düşürür ama geçerli bir RTP payload oluşturur.
        ((pcm >> 8) & 0xFF) as u8
    }
}

impl Encoder for G711 {
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        pcm_samples.iter()
            .map(|&sample| Self::compress_sample(sample))
            .collect()
    }
}