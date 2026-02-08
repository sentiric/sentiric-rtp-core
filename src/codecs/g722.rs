// sentiric-rtp-core/src/codecs/g722.rs

// ============================================================================
// !!! SIMULATION ONLY - NOT FOR PRODUCTION TRANSCODING !!!
// This implementation simulates 16kHz wideband audio by upsampling G.711.
// It is NOT a standard G.722 SB-ADPCM implementation.
// Use this only for passing RTP packets or basic testing.
// ============================================================================

use super::{Encoder, Decoder, CodecType};
use super::pcmu::PcmuEncoder; 
use super::codec_data::ULAW_TO_LINEAR_LUT; // YENİ İSİM: g711_data yerine codec_data

pub struct G722;

impl G722 {
    pub fn new() -> Self {
        G722 {}
    }

    // --- Simülasyon Yardımcıları ---
    
    // Encode: Linear -> Simulated ADPCM Nibble (4-bit)
    fn encode_nibble(sample: i16) -> u8 {
        PcmuEncoder::linear_to_ulaw(sample)
    }

    // Decode: Simulated ADPCM Nibble (4-bit) -> Linear
    fn decode_nibble(nibble: u8) -> i16 {
        let ulaw = (nibble << 4) | 0x08; 
        ULAW_TO_LINEAR_LUT[ulaw as usize]
    }
}

impl Encoder for G722 {
    fn get_type(&self) -> CodecType {
        CodecType::G722
    }

    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        let mut wideband = Vec::with_capacity(pcm_samples.len() * 2);
        
        for i in 0..pcm_samples.len() {
            wideband.push(pcm_samples[i]);
            if i + 1 < pcm_samples.len() {
                let val = (pcm_samples[i] as i32 + pcm_samples[i+1] as i32) / 2;
                wideband.push(val as i16);
            } else {
                wideband.push(pcm_samples[i]);
            }
        }

        let mut output = Vec::with_capacity(wideband.len() / 2);
        
        for chunk in wideband.chunks(2) {
            if chunk.len() < 2 { break; }
            let s1 = chunk[0];
            let s2 = chunk[1];

            let n1 = Self::encode_nibble(s1);
            let n2 = Self::encode_nibble(s2);

            let byte = (n2 << 4) | n1;
            output.push(byte);
        }

        output
    }
}

impl Decoder for G722 {
    fn get_type(&self) -> CodecType {
        CodecType::G722
    }

    fn decode(&mut self, payload: &[u8]) -> Vec<i16> {
        let mut output = Vec::with_capacity(payload.len() * 2);

        for byte in payload {
            let n1 = byte & 0x0F;
            let n2 = (byte >> 4) & 0x0F;

            let s1 = Self::decode_nibble(n1);
            let s2 = Self::decode_nibble(n2);

            output.push(s1);
            output.push(s2);
        }
        output
    }
}