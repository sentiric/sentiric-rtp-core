// sentiric-rtp-core/src/codecs/mod.rs

pub mod codec_data;
pub mod g729;
pub mod g722;
pub mod pcma;
pub mod pcmu;

pub use g729::{G729Encoder, G729Decoder};
pub use g722::G722;
pub use pcma::{PcmaEncoder, PcmaDecoder};
pub use pcmu::{PcmuEncoder, PcmuDecoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecType {
    PCMU = 0,
    PCMA = 8,
    G722 = 9,
    G729 = 18,
}

impl CodecType {
    pub fn sample_rate(&self) -> u32 {
        match self {
            CodecType::PCMU | CodecType::PCMA | CodecType::G729 => 8000,
            CodecType::G722 => 16000,
        }
    }

    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
            0 => Some(CodecType::PCMU),
            8 => Some(CodecType::PCMA),
            9 => Some(CodecType::G722),
            18 => Some(CodecType::G729),
            _ => None,
        }
    }
}

pub trait Encoder: Send {
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8>;
    fn get_type(&self) -> CodecType;
}

pub trait Decoder: Send {
    fn decode(&mut self, payload: &[u8]) -> Vec<i16>;
    fn get_type(&self) -> CodecType;
}

pub struct CodecFactory;

impl CodecFactory {
    /// [v1.3.4 MİMARİ DÜZELTME]: Eksik PCMA (Alaw) desteği geri eklendi.
    /// Tüm varyantlar kapsandığı için 'unreachable pattern' uyarısını önlemek adına 
    /// default (_) bloğu kaldırıldı.
    pub fn create_encoder(codec: CodecType) -> Box<dyn Encoder> {
        match codec {
            CodecType::G729 => Box::new(G729Encoder::new()),
            CodecType::PCMU => Box::new(PcmuEncoder {}),
            CodecType::PCMA => Box::new(PcmaEncoder {}), // Alaw desteği
            CodecType::G722 => Box::new(G722::new()),    // Wideband desteği
        }
    }

    pub fn create_decoder(codec: CodecType) -> Box<dyn Decoder> {
        match codec {
            CodecType::G729 => Box::new(G729Decoder::new()),
            CodecType::PCMU => Box::new(PcmuDecoder {}),
            CodecType::PCMA => Box::new(PcmaDecoder {}), // Alaw desteği
            CodecType::G722 => Box::new(G722::new()),    // Wideband desteği
        }
    }
}