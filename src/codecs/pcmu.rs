// src/codecs/pcmu.rs

use super::{Encoder, Decoder, CodecType};
use super::codec_data::ULAW_TO_LINEAR_LUT; // YENİ İSİM: g711_data yerine codec_data

pub struct PcmuEncoder;
pub struct PcmuDecoder;

impl PcmuEncoder {
    // 43dB ile testi geçen, kanıtlanmış kod. (Şimdi pub statik)
    pub fn linear_to_ulaw(pcm_val: i16) -> u8 {
        let bias = 0x84; // 132
        let mut pcm = pcm_val as i32;
        let sign = if pcm < 0 {
            if pcm == i16::MIN as i32 { 
                pcm = i16::MAX as i32; 
            } else {
                pcm = -pcm;
            }
            0x80 
        } else {
            0x00 
        };

        if pcm > 32635 { pcm = 32635; }
        
        pcm += bias;

        let exponent: u8;
        if pcm >= 16384 { exponent = 7; }
        else if pcm >= 8192 { exponent = 6; }
        else if pcm >= 4096 { exponent = 5; }
        else if pcm >= 2048 { exponent = 4; }
        else if pcm >= 1024 { exponent = 3; }
        else if pcm >= 512 { exponent = 2; }
        else if pcm >= 256 { exponent = 1; }
        else { exponent = 0; }
        
        let mantissa = ((pcm >> (exponent + 3)) & 0x0F) as u8;
        
        let val = (sign as u8) | (exponent << 4) | mantissa; 
        
        !val
    }
}

unsafe impl Send for PcmuEncoder {}
impl Encoder for PcmuEncoder {
    fn get_type(&self) -> CodecType { CodecType::PCMU }
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        pcm_samples.iter().map(|&s| Self::linear_to_ulaw(s)).collect()
    }
}

unsafe impl Send for PcmuDecoder {}
impl Decoder for PcmuDecoder {
    fn get_type(&self) -> CodecType { CodecType::PCMU }
    fn decode(&mut self, payload: &[u8]) -> Vec<i16> {
        payload.iter().map(|&b| ULAW_TO_LINEAR_LUT[b as usize]).collect()
    }
}