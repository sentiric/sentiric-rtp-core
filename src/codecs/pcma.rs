// sentiric-rtp-core/src/codecs/pcma.rs
// // PCMA Cırıtlı aha zonra uzerınde calısılıacak
use super::{Encoder, Decoder, CodecType};
use super::codec_data::ALAW_TO_LINEAR_LUT;

pub struct PcmaEncoder;
pub struct PcmaDecoder;

impl PcmaEncoder {
    // A-LAW ENCODER (ITU-T Standardı)
    // 13-bit signed linear PCM -> 8-bit A-law
    pub fn linear_to_alaw(pcm_val: i16) -> u8 {
        let mut pcm = pcm_val;
        let mask;

        // 1. İşaret Bitini Yönet (A-law: Pozitif=0xD5 ile XORlanır)
        if pcm >= 0 {
            mask = 0xD5;
        } else {
            mask = 0x55;
            pcm = -pcm - 8;
        }

        // 2. A-law sıkıştırma aralığına çek (MAX 32635)
        if pcm > 32635 { pcm = 32635; }

        // 3. Segment ve Mantissa Hesabı
        // A-law algoritması 13-bitlik veriyi (işaret hariç) kullanır.
        let ival = pcm;
        let segment: u8;
        
        if ival < 256 {
            segment = 0;
        } else if ival < 512 {
            segment = 1;
        } else if ival < 1024 {
            segment = 2;
        } else if ival < 2048 {
            segment = 3;
        } else if ival < 4096 {
            segment = 4;
        } else if ival < 8192 {
            segment = 5;
        } else if ival < 16384 {
            segment = 6;
        } else {
            segment = 7;
        }

        let mantissa = if segment < 1 {
            (ival >> 4) & 0x0F
        } else {
            (ival >> (segment + 3)) & 0x0F
        };

        let val = (segment << 4) | (mantissa as u8);
        
        // 4. Standart A-law XOR Maskelemesi
        val ^ mask
    }
}

unsafe impl Send for PcmaEncoder {}
impl Encoder for PcmaEncoder {
    fn get_type(&self) -> CodecType { CodecType::PCMA }
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        pcm_samples.iter().map(|&s| Self::linear_to_alaw(s)).collect()
    }
}

unsafe impl Send for PcmaDecoder {}
impl Decoder for PcmaDecoder {
    fn get_type(&self) -> CodecType { CodecType::PCMA }
    fn decode(&mut self, payload: &[u8]) -> Vec<i16> {
        payload.iter().map(|&b| ALAW_TO_LINEAR_LUT[b as usize]).collect()
    }
}