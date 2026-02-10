// sentiric-rtp-core/src/dsp.rs

/// Digital Signal Processing (DSP) Utilities
/// Gerçek zamanlı ses işleme için optimize edilmiş algoritmalar.

pub struct Resampler;

impl Resampler {
    /// 8kHz'den 16kHz'e Lineer İnterpolasyon ile yükseltme (Upsampling).
    /// AI servisleri (STT/LLM) genellikle 16kHz ister, Telekom (G.711) 8kHz verir.
    /// Bu işlem, eksik örnekleri "tahmin ederek" (yumuşatarak) araya doldurur.
    pub fn upsample_linear_8k_to_16k(input: &[i16]) -> Vec<i16> {
        let mut output = Vec::with_capacity(input.len() * 2);
        for i in 0..input.len() {
            let current = input[i];
            output.push(current);
            
            // Bir sonraki örnek varsa ortalamasını al, yoksa aynısını tekrarla
            if i + 1 < input.len() {
                let next = input[i + 1];
                // (A + B) / 2
                let avg = ((current as i32 + next as i32) / 2) as i16;
                output.push(avg);
            } else {
                output.push(current);
            }
        }
        output
    }

    /// 16kHz'den 8kHz'e Ortalamalı İndirgeme (Downsampling).
    /// AI'dan gelen 16kHz sesi, Telekom (G.711) için 8kHz'e düşürür.
    /// Basitçe her 2. örneği atmak yerine (Decimation), iki örneğin ortalamasını alır.
    /// Bu, "Aliasing" (örtüşme) bozulmalarını azaltır ve sesi daha doğal kılar.
    pub fn downsample_average_16k_to_8k(input: &[i16]) -> Vec<i16> {
        let mut output = Vec::with_capacity(input.len() / 2);
        for chunk in input.chunks(2) {
            if chunk.len() == 2 {
                // (Sample1 + Sample2) / 2
                let avg = ((chunk[0] as i32 + chunk[1] as i32) / 2) as i16;
                output.push(avg);
            } else if chunk.len() == 1 {
                output.push(chunk[0]);
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsample() {
        let input = vec![100, 200];
        let output = Resampler::upsample_linear_8k_to_16k(&input);
        // Beklenen: 100, 150 (ort), 200, 200 (son)
        assert_eq!(output, vec![100, 150, 200, 200]);
        assert_eq!(output.len(), 4);
    }

    #[test]
    fn test_downsample() {
        let input = vec![100, 150, 200, 220];
        let output = Resampler::downsample_average_16k_to_8k(&input);
        // Beklenen: (100+150)/2 = 125, (200+220)/2 = 210
        assert_eq!(output, vec![125, 210]);
        assert_eq!(output.len(), 2);
    }
}