// sentiric-rtp-core/src/dsp.rs

use std::sync::Arc;
use tokio::sync::Mutex;

/// Profesyonel Ses İşleme Motoru (Cubic Interpolation).
/// 
/// Telekom (8kHz) ve AI (16kHz) arasında, paket sınırlarında sürekliliği (continuity)
/// koruyarak ve sabit gecikme ile dönüşüm yapar.
pub struct AudioResampler {
    state: Arc<Mutex<ResamplerState>>,
    input_rate: usize,
    output_rate: usize,
}

struct ResamplerState {
    // Önceki chunk'tan kalan son örnekler (Süreklilik için)
    // Cubic için en az 3 geçmiş örneğe ihtiyaç var.
    history: Vec<f32>,
}

impl AudioResampler {
    pub fn new(input_rate: usize, output_rate: usize, _chunk_size: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(ResamplerState {
                // Başlangıçta sessizlik ile doldur (Cubic 4 nokta ister: p0, p1, p2, p3)
                // Biz p1 ve p2 arasını interpole ederiz. p0 ve p1 geçmişten gelir.
                history: vec![0.0; 4], 
            })),
            input_rate,
            output_rate,
        }
    }

    /// PCM (i16) verisini işler.
    pub async fn process(&self, input: &[i16]) -> Vec<i16> {
        if self.input_rate == self.output_rate {
            return input.to_vec();
        }

        let mut state = self.state.lock().await;
        let ratio = self.output_rate as f32 / self.input_rate as f32;
        let output_len = (input.len() as f32 * ratio).ceil() as usize;
        let mut output = Vec::with_capacity(output_len);

        // 1. Girdiyi f32'ye çevir ve History ile birleştir
        // History: [p-2, p-1, p0] -> Yeni: [s0, s1, s2...]
        // İnterpolasyon algoritması için sürekli bir akış oluşturuyoruz.
        let mut stream: Vec<f32> = state.history.clone();
        stream.reserve(input.len());
        for &s in input {
            stream.push(s as f32);
        }

        // 2. Cubic Interpolation
        // p0, p1, p2, p3 noktaları arasında p1-p2 arasını dolduracağız.
        // t: 0.0 ile 1.0 arası (p1 ile p2 arasındaki konum)
        
        let mut input_index: f32 = 0.0;
        // History buffer boyutu kadar offset (Algoritma history'nin sonundan başlar)
        let offset = state.history.len() as f32 - 2.0; 
        
        // Yeni üretilecek her örnek için
        while (input_index) < (input.len() as f32) {
            let virtual_idx = input_index + offset;
            let i = virtual_idx.floor() as usize;
            let t = virtual_idx - i as f32;

            // Güvenlik kontrolü (Stream sınırlarını aşma)
            if i + 3 >= stream.len() {
                break;
            }

            let p0 = stream[i];
            let p1 = stream[i + 1];
            let p2 = stream[i + 2];
            let p3 = stream[i + 3];

            let interpolated = cubic_interp(p0, p1, p2, p3, t);
            
            // Clipping Protection & i16 Conversion
            let clamped = interpolated.clamp(-32768.0, 32767.0);
            output.push(clamped as i16);

            // Bir sonraki adıma geç (Ratio kadar ilerle)
            // Örn: 8k->16k için step 0.5 (tersi 2.0 mı?)
            // Hayır, Output üretirken input üzerinde ne kadar ilerlediğimiz:
            // 8k -> 16k (Upsample): Her output için inputta 0.5 ilerleriz.
            input_index += 1.0 / ratio;
        }

        // 3. State Güncelleme (Gelecek chunk için son örnekleri sakla)
        // Son 4 örneği sakla
        if stream.len() >= 4 {
            state.history = stream[stream.len()-4..].to_vec();
        }

        output
    }
}

/// Catmull-Rom Spline Interpolation (Cubic)
/// p0, p1, p2, p3: Kontrol noktaları
/// t: p1 ile p2 arasındaki interpolasyon faktörü [0.0, 1.0]
#[inline(always)]
fn cubic_interp(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let a = -0.5 * p0 + 1.5 * p1 - 1.5 * p2 + 0.5 * p3;
    let b = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
    let c = -0.5 * p0 + 0.5 * p2;
    let d = p1;

    a * t * t * t + b * t * t + c * t + d
}

/// Basit kullanım için statik helper (Stateful değildir, her seferinde sıfırlanır)
pub fn simple_resample(input: &[i16], from_rate: usize, to_rate: usize) -> Vec<i16> {
    if input.is_empty() { return vec![]; }
    
    // Geçici bir resampler oluştur ve işle (async olmadığı için block_on gerekmez, mantık aynıdır)
    // Ancak async fonksiyonu senkron içinde çağıramayız.
    // Basit bir lineer implementasyon burası için yeterli olabilir veya
    // yukarıdaki cubic mantığının senkron kopyası.
    // Hız için burada Linear yapalım (Testler için yeterli).
    
    let ratio = to_rate as f32 / from_rate as f32;
    let new_len = (input.len() as f32 * ratio).ceil() as usize;
    let mut output = Vec::with_capacity(new_len);
    
    for i in 0..new_len {
        let src_idx_f = i as f32 / ratio;
        let src_idx = src_idx_f.floor() as usize;
        let t = src_idx_f - src_idx as f32;
        
        if src_idx + 1 < input.len() {
            let a = input[src_idx] as f32;
            let b = input[src_idx + 1] as f32;
            let val = a + (b - a) * t;
            output.push(val as i16);
        } else if src_idx < input.len() {
            output.push(input[src_idx]);
        }
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resampler_upscale() {
        // 8k -> 16k (2 katına çıkmalı)
        // 160 input -> 320 output
        let input = vec![1000; 160]; 
        let resampler = AudioResampler::new(8000, 16000, 160);
        
        let output = resampler.process(&input).await;
        
        println!("Upscale: In {} -> Out {}", input.len(), output.len());
        // Cubic algoritması padding yüzünden bazen +/- 1-2 sample oynayabilir ama
        // rubato gibi yarı yarıya düşmez.
        assert!(output.len() >= 315 && output.len() <= 325, "Output boyutu hatalı: {}", output.len());
    }

    #[tokio::test]
    async fn test_resampler_downscale() {
        // 16k -> 8k (Yarıya inmeli)
        // 320 input -> 160 output
        let input = vec![1000; 320]; 
        let resampler = AudioResampler::new(16000, 8000, 320);
        
        let output = resampler.process(&input).await;
        
        println!("Downscale: In {} -> Out {}", input.len(), output.len());
        assert!(output.len() >= 158 && output.len() <= 162, "Output boyutu hatalı: {}", output.len());
    }
    
    #[tokio::test]
    async fn test_continuity() {
        // Stateful test: İki chunk arka arkaya gelince kırılma olmamalı
        let input = vec![1000; 160];
        let resampler = AudioResampler::new(8000, 16000, 160);
        
        let out1 = resampler.process(&input).await;
        let out2 = resampler.process(&input).await;
        
        assert_eq!(out1.len(), out2.len(), "Chunk boyutları tutarlı olmalı");
    }
}