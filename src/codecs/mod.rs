// sentiric-rtp-core/src/codecs/mod.rs

pub mod codec_data;
pub mod g729;
pub mod pcmu;
// PCMA Cırıtlı
pub mod pcma;

pub use g729::{G729Encoder, G729Decoder};
pub use pcmu::{PcmuEncoder, PcmuDecoder};
// PCMA Cırıtlı
pub use pcma::{PcmaEncoder, PcmaDecoder};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecType {
    G729 = 18,
    PCMU = 0,
    PCMA = 8,
    /// RFC 4733/2833 DTMF Events (Payload 101)
    TelephoneEvent = 101, 
}

impl CodecType {
    pub fn sample_rate(&self) -> u32 {
        match self {
            CodecType::PCMU | CodecType::PCMA | CodecType::G729 | CodecType::TelephoneEvent => 8000,
        }
    }

    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
            18 => Some(CodecType::G729),
            0 => Some(CodecType::PCMU),
            8 => Some(CodecType::PCMA),
            101 => Some(CodecType::TelephoneEvent),
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

/// Boş Encoder (DTMF gibi ses olmayan türler için)
pub struct NoOpEncoder;
impl Encoder for NoOpEncoder {
    fn encode(&mut self, _pcm: &[i16]) -> Vec<u8> { vec![] }
    fn get_type(&self) -> CodecType { CodecType::TelephoneEvent }
}

/// Boş Decoder
pub struct NoOpDecoder;
impl Decoder for NoOpDecoder {
    fn decode(&mut self, _payload: &[u8]) -> Vec<i16> { vec![] }
    fn get_type(&self) -> CodecType { CodecType::TelephoneEvent }
}

pub struct CodecFactory;

impl CodecFactory {
    pub fn create_encoder(codec: CodecType) -> Box<dyn Encoder> {
        match codec {
            CodecType::G729 => Box::new(G729Encoder::new()),
            CodecType::PCMU => Box::new(pcmu::PcmuEncoder {}),
            CodecType::PCMA => Box::new(pcma::PcmaEncoder {}),
            CodecType::TelephoneEvent => Box::new(NoOpEncoder {}),
        }
    }

    pub fn create_decoder(codec: CodecType) -> Box<dyn Decoder> {
        match codec {
            CodecType::G729 => Box::new(G729Decoder::new()),
            CodecType::PCMU => Box::new(pcmu::PcmuDecoder {}),
            CodecType::PCMA => Box::new(pcma::PcmaDecoder {}),
            CodecType::TelephoneEvent => Box::new(NoOpDecoder {}),
        }
    }
}