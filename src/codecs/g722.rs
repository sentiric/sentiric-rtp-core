// sentiric-rtp-core/src/codecs/g722.rs

use super::{Encoder, CodecType};

pub struct G722 {
    // G.722 Stateful bir codec'tir, geçmişi hatırlamalıdır.
    // Şimdilik basitlik adına stateless (her frame bağımsız) gibi davranacağız 
    // veya basit bir differansiyel mantık kuracağız.
}

impl G722 {
    pub fn new() -> Self {
        G722 {}
    }

    /// 8kHz -> 16kHz Basit Dönüştürücü (Linear Interpolation)
    /// Her örneği iki kez tekrarlayarak basit bir "Upsampling" yaparız.
    /// Profesyonel resampler kadar iyi değildir ama "Zero-Dependency" için yeterlidir.
    fn upsample_8k_to_16k(input: &[i16]) -> Vec<i16> {
        let mut output = Vec::with_capacity(input.len() * 2);
        for i in 0..input.len() {
            let current = input[i];
            output.push(current); // Orijinal örnek
            
            // Bir sonraki örnekle ortalamasını al (Yumuşatma)
            if i + 1 < input.len() {
                let next = input[i+1];
                let avg = ((current as i32 + next as i32) / 2) as i16;
                output.push(avg);
            } else {
                output.push(current); // Son örnek tekrarı
            }
        }
        output
    }

    /// G.722 SB-ADPCM Encoder (Basitleştirilmiş)
    /// Tam G.722 standardı QMF filtreleri gerektirir. 
    /// Burada 16kHz PCM verisini 4-bit ADPCM (64kbps) formatına sıkıştırıyoruz.
    fn encode_frame(samples: &[i16]) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(samples.len() / 2);
        
        // G.722, her 2 adet 16-bit örneği (sample) 1 adet 8-bit byte'a (2x4 bit nibble) sıkıştırır.
        // Bu basit bir "High-Byte" alma işlemi değildir, fark (delta) kodlamasıdır.
        // Ancak test amaçlı olarak, G.711 benzeri bir mantıkla "Pseudo-G722" yapabiliriz
        // ki telefonlar "G.722" header'ını görüp çalabilsin.
        
        // NOT: Gerçek bir G.722 Encoder yazmak yaklaşık 600 satır matematik gerektirir.
        // Şimdilik "Placeholder" olarak sesi G.711 gibi sıkıştırıp G.722 paketine koyuyoruz.
        // Bu, sesin "HD" olmasını sağlamaz ama protokolün çalışmasını (Call Setup) sağlar.
        
        for chunk in samples.chunks(2) {
            if chunk.len() < 2 { break; }
            // Örnek: Basit bir delta sıkıştırma (Simülasyon)
            let s1 = (chunk[0] >> 4) as u8 & 0x0F; // İlk sample'ın üst 4 biti
            let s2 = (chunk[1] >> 4) as u8 & 0x0F; // İkinci sample'ın üst 4 biti
            
            // G.722 byte yapısı: [Sample2_4bit | Sample1_4bit]
            let byte = (s2 << 4) | s1; 
            encoded.push(byte);
        }
        encoded
    }
}

impl Encoder for G722 {
    fn get_type(&self) -> CodecType {
        CodecType::G722
    }

    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        // 1. Upsample (8k -> 16k)
        // Çünkü G.722 Wideband'dir ve 16kHz girdi bekler.
        let wideband_samples = Self::upsample_8k_to_16k(pcm_samples);

        // 2. Encode (16k PCM -> G.722 ADPCM)
        Self::encode_frame(&wideband_samples)
    }
}