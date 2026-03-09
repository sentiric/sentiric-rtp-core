// sentiric-rtp-core/src/dsp.rs

use std::sync::{Arc, Mutex};

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
    history: Vec<f32>,
}

impl AudioResampler {
    pub fn new(input_rate: usize, output_rate: usize, _chunk_size: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(ResamplerState {
                history: vec![0.0; 4], 
            })),
            input_rate,
            output_rate,
        }
    }

    /// PCM (i16) verisini işler. (ARTIK SENKRON - ASYNC OVERHEAD KALDIRILDI)
    pub fn process(&self, input: &[i16]) -> Vec<i16> {
        if self.input_rate == self.output_rate {
            return input.to_vec();
        }

        // Std Mutex anında kilitlenir, async beklemeye (context switch) gerek yoktur.
        let mut state = self.state.lock().unwrap();
        let ratio = self.output_rate as f32 / self.input_rate as f32;
        let output_len = (input.len() as f32 * ratio).ceil() as usize;
        let mut output = Vec::with_capacity(output_len);

        let mut stream: Vec<f32> = state.history.clone();
        stream.reserve(input.len());
        for &s in input {
            stream.push(s as f32);
        }

        let mut input_index: f32 = 0.0;
        let offset = state.history.len() as f32 - 2.0; 
        
        while input_index < (input.len() as f32) {
            let virtual_idx = input_index + offset;
            let i = virtual_idx.floor() as usize;
            let t = virtual_idx - i as f32;

            if i + 3 >= stream.len() {
                break;
            }

            let p0 = stream[i];
            let p1 = stream[i + 1];
            let p2 = stream[i + 2];
            let p3 = stream[i + 3];

            let interpolated = cubic_interp(p0, p1, p2, p3, t);
            let clamped = interpolated.clamp(-32768.0, 32767.0);
            output.push(clamped as i16);

            input_index += 1.0 / ratio;
        }

        if stream.len() >= 4 {
            state.history = stream[stream.len()-4..].to_vec();
        }

        output
    }
}

#[inline(always)]
fn cubic_interp(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let a = -0.5 * p0 + 1.5 * p1 - 1.5 * p2 + 0.5 * p3;
    let b = p0 - 2.5 * p1 + 2.0 * p2 - 0.5 * p3;
    let c = -0.5 * p0 + 0.5 * p2;
    let d = p1;

    a * t * t * t + b * t * t + c * t + d
}

pub fn simple_resample(input: &[i16], from_rate: usize, to_rate: usize) -> Vec<i16> {
    if input.is_empty() { return vec![]; }
    
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