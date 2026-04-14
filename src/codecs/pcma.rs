// sentiric-rtp-core/src/codecs/pcma.rs

use super::codec_data::ALAW_TO_LINEAR_LUT;
use super::{CodecType, Decoder, Encoder};

pub struct PcmaEncoder;
pub struct PcmaDecoder;

impl Default for PcmaEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl PcmaEncoder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn linear_to_alaw(&mut self, pcm_val: i16) -> u8 {
        let mut pcm = pcm_val;
        let mask: u8;

        if pcm >= 0 {
            mask = 0xD5;
        } else {
            mask = 0x55;
            // Negatif sayıları A-law standardına (13-bit) çek
            pcm = -pcm - 1;
        }

        if pcm > 32635 {
            pcm = 32635;
        }

        let ival = pcm;
        let segment = if ival < 256 {
            0
        } else if ival < 512 {
            1
        } else if ival < 1024 {
            2
        } else if ival < 2048 {
            3
        } else if ival < 4096 {
            4
        } else if ival < 8192 {
            5
        } else if ival < 16384 {
            6
        } else {
            7
        };

        let mantissa = if segment < 1 {
            (ival >> 4) & 0x0F
        } else {
            (ival >> (segment + 3)) & 0x0F
        };

        let val = (segment << 4) | (mantissa as u8);
        val ^ mask
    }
}

unsafe impl Send for PcmaEncoder {}
impl Encoder for PcmaEncoder {
    fn get_type(&self) -> CodecType {
        CodecType::PCMA
    }
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        pcm_samples
            .iter()
            .map(|&s| self.linear_to_alaw(s))
            .collect()
    }
}

unsafe impl Send for PcmaDecoder {}
impl Decoder for PcmaDecoder {
    fn get_type(&self) -> CodecType {
        CodecType::PCMA
    }
    fn decode(&mut self, payload: &[u8]) -> Vec<i16> {
        payload
            .iter()
            .map(|&b| ALAW_TO_LINEAR_LUT[b as usize])
            .collect()
    }
}
