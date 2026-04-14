// sentiric-rtp-core/src/dsp.rs

/// Profesyonel Ses İşleme Motoru.
/// Özellikler: Cubic Interpolation, Stateful Anti-Aliasing, 300Hz High-Pass Filter, Noise Gate.
/// [ARCH-COMPLIANCE] Zero-Cost Abstraction: Mutex ve Arc kaldırılarak Lock Contention engellendi.
pub struct AudioResampler {
    history: Vec<f32>,
    fractional_phase: f32,
    fir_history: [i16; 4],
    // High-Pass Filter Hafızası (300Hz DC Blocker)
    hpf_x1: f32,
    hpf_x2: f32,
    hpf_y1: f32,
    hpf_y2: f32,
    input_rate: usize,
    output_rate: usize,
}

impl AudioResampler {
    pub fn new(input_rate: usize, output_rate: usize, _chunk_size: usize) -> Self {
        Self {
            history: vec![0.0; 4],
            fractional_phase: 0.0,
            fir_history: [0; 4],
            hpf_x1: 0.0,
            hpf_x2: 0.0,
            hpf_y1: 0.0,
            hpf_y2: 0.0,
            input_rate,
            output_rate,
        }
    }

    /// 300Hz High-Pass Filter (Butterworth IIR)
    /// G.711'in cızırtı yapmasına neden olan bas frekansları ve DC kaymasını temizler.
    fn apply_hpf(&mut self, sample: f32) -> f32 {
        let b0 = 0.8413;
        let b1 = -1.6826;
        let b2 = 0.8413;
        let a1 = -1.6380;
        let a2 = 0.7272;
        let out =
            b0 * sample + b1 * self.hpf_x1 + b2 * self.hpf_x2 - a1 * self.hpf_y1 - a2 * self.hpf_y2;
        self.hpf_x2 = self.hpf_x1;
        self.hpf_x1 = sample;
        self.hpf_y2 = self.hpf_y1;
        self.hpf_y1 = out;
        out
    }

    /// Stateful Anti-Aliasing FIR Filtresi
    fn apply_anti_aliasing(&mut self, input: &[i16]) -> Vec<f32> {
        let mut filtered = Vec::with_capacity(input.len());
        let coeffs = [-0.05, 0.25, 0.60, 0.25, -0.05];
        for i in 0..input.len() {
            let mut sum = 0.0;
            for j in 0..5 {
                let val = if i >= j {
                    input[i - j]
                } else {
                    self.fir_history[4 - (j - i)]
                };
                sum += val as f32 * coeffs[j];
            }
            filtered.push(sum);
        }
        if input.len() >= 4 {
            self.fir_history.copy_from_slice(&input[input.len() - 4..]);
        }
        filtered
    }

    // [ARCH-COMPLIANCE] &self yerine &mut self alarak state mutasyonunu güvenli ve lock-free hale getirdik.
    pub fn process(&mut self, input: &[i16]) -> Vec<i16> {
        if self.input_rate == self.output_rate {
            return input.to_vec();
        }

        let ratio = self.output_rate as f32 / self.input_rate as f32;

        let processed_input = if self.output_rate < self.input_rate {
            self.apply_anti_aliasing(input)
        } else {
            input.iter().map(|&s| s as f32).collect()
        };

        let mut stream = self.history.clone();
        stream.extend(processed_input);
        if stream.len() >= 4 {
            self.history = stream[stream.len() - 4..].to_vec();
        }
        if let Some(&last) = stream.last() {
            stream.push(last);
            stream.push(last);
        }

        let mut output = Vec::with_capacity((input.len() as f32 * ratio).ceil() as usize);
        let mut input_index = self.fractional_phase;

        while input_index < (input.len() as f32) {
            let virtual_idx = input_index + 2.0;
            let i = virtual_idx.floor() as usize;
            let t = virtual_idx - i as f32;
            if i + 3 >= stream.len() {
                break;
            }
            let interpolated =
                cubic_interp(stream[i], stream[i + 1], stream[i + 2], stream[i + 3], t);

            // HPF Uygula
            let filtered = self.apply_hpf(interpolated);

            // Noise Gate (Kelime arası sessizleştirme)
            let final_sample = if filtered.abs() < 30.0 { 0.0 } else { filtered };
            output.push(final_sample.clamp(-32768.0, 32767.0) as i16);
            input_index += 1.0 / ratio;
        }
        self.fractional_phase = input_index - input.len() as f32;
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
    if input.is_empty() {
        return vec![];
    }
    let ratio = to_rate as f32 / from_rate as f32;
    let mut output = Vec::new();
    for i in 0..(input.len() as f32 * ratio).ceil() as usize {
        let idx_f = i as f32 / ratio;
        let idx = idx_f.floor() as usize;
        let t = idx_f - idx as f32;
        if idx + 1 < input.len() {
            let val = input[idx] as f32 + (input[idx + 1] as f32 - input[idx] as f32) * t;
            output.push(val as i16);
        }
    }
    output
}
