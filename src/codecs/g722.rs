// sentiric-rtp-core/src/codecs/g722.rs

use super::{Encoder, CodecType};

pub struct G722;

impl G722 {
    pub fn new() -> Self {
        G722 {}
    }

    /// G.711 mu-law tablosu (G.722 lower-subband simülasyonu için)
    /// Gerçek bir G.722 encoder yazmak yerine, PCM verisini
    /// 4-bit ADPCM benzeri bir yapıya indirgiyoruz.
    /// Bu "HD" ses vermez ama "Geçerli" ses verir (Cızırtı yerine anlaşılır konuşma).
    fn encode_simulated(sample: i16) -> u8 {
        // Basitçe üst 12 biti alıp 4 bite sıkıştırma denemesi (Çok kayıplı)
        // Daha iyi yöntem: G.711 u-law encode et, sonra üst 4 biti al.
        let ulaw = crate::codecs::g711::G711::linear_to_ulaw(sample);
        // G.722 lower subband, u-law'a benzer karakteristik gösterir.
        // u-law 8 bit, G.722 nibble 4 bit.
        // En basit geçerli dönüşüm: u-law'ın en anlamlı 4 bitini al.
        (ulaw >> 4) & 0x0F
    }
}

impl Encoder for G722 {
    fn get_type(&self) -> CodecType {
        CodecType::G722
    }

    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        // G.722, 16kHz input bekler. Bizim WAV dosyamız 8kHz.
        // Önce basit upsampling (Linear Interpolation)
        let mut wideband = Vec::with_capacity(pcm_samples.len() * 2);
        
        for i in 0..pcm_samples.len() {
            wideband.push(pcm_samples[i]);
            if i + 1 < pcm_samples.len() {
                // Ara değer enterpolasyonu
                let val = (pcm_samples[i] as i32 + pcm_samples[i+1] as i32) / 2;
                wideband.push(val as i16);
            } else {
                wideband.push(pcm_samples[i]);
            }
        }

        // Encode: 2 sample -> 1 byte (High Nibble | Low Nibble)
        let mut output = Vec::with_capacity(wideband.len() / 2);
        
        for chunk in wideband.chunks(2) {
            if chunk.len() < 2 { break; }
            let s1 = chunk[0];
            let s2 = chunk[1];

            // Simüle edilmiş ADPCM nibble'ları
            let n1 = Self::encode_simulated(s1);
            let n2 = Self::encode_simulated(s2);

            // G.722 Byte yapısı: [Sample 2 (4 bit) | Sample 1 (4 bit)]
            // Dikkat: Endianness ve nibble sırası önemlidir.
            // Genelde Low nibble ilk sample'dır.
            let byte = (n2 << 4) | n1;
            output.push(byte);
        }

        output
    }
}