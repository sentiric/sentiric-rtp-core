// sentiric-rtp-core/src/codecs/mod.rs

pub mod codec_data;
pub mod g729;
pub mod pcmu;
pub mod pcma;

pub use g729::{G729Encoder, G729Decoder};
pub use pcmu::{PcmuEncoder, PcmuDecoder};
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

    /// Belirtilen ptime (ms) için örnek (sample) sayısını döndürür.
    /// Örn: 8000Hz * 20ms = 160 sample.
    pub fn samples_per_frame(&self, ptime_ms: u8) -> usize {
        let rate = self.sample_rate();
        (rate as usize * ptime_ms as usize) / 1000
    }

    /// Belirtilen ptime (ms) için RTP payload boyutunu (byte) döndürür.
    /// G.729 gibi sıkıştırılmış kodekler için bu hesap farklıdır.
    pub fn payload_size_bytes(&self, ptime_ms: u8) -> usize {
        match self {
            // G.729: 10ms = 10 byte (8kbps). 20ms = 20 byte.
            // Parantez uyarısı giderildi.
            CodecType::G729 => ptime_ms as usize, 
            // G.711 (PCMU/PCMA): 1 sample = 1 byte. 20ms = 160 byte.
            CodecType::PCMU | CodecType::PCMA => self.samples_per_frame(ptime_ms),
            // DTMF değişkendir, event packet genellikle 4 byte
            CodecType::TelephoneEvent => 4, 
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