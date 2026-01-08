// sentiric-rtp-core/src/codecs/mod.rs

pub mod g711;
pub mod g729;

// KRİTİK DÜZELTME: Alt modülleri dışarıya açıyoruz
pub use g711::G711;
pub use g729::G729;

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

/// Tüm codec'lerin uyması gereken Ortak Arayüz
pub trait Encoder: Send {
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8>;
    fn get_type(&self) -> CodecType;
}

// Codec Fabrikası
pub struct CodecFactory;

impl CodecFactory {
    pub fn create_encoder(codec: CodecType) -> Box<dyn Encoder> {
        match codec {
            CodecType::PCMA => Box::new(G711::new(CodecType::PCMA)),
            CodecType::PCMU => Box::new(G711::new(CodecType::PCMU)),
            CodecType::G729 => Box::new(G729::new()),
            CodecType::G722 => panic!("G.722 henüz implemente edilmedi!"),
        }
    }
}