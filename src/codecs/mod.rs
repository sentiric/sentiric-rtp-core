// sentiric-rtp-core/src/codecs/mod.rs

pub mod codec_data;
pub mod g729;
pub mod pcmu;

pub mod pcma;
// pub mod g722; // [İPTAL]: G.722 modülü derleme dışı bırakıldı.


// ============================================================================
// !!! PRODUCTION READY CODECS !!!
// 
// [STABLE] G.729: Düşük bant genişliği, yüksek kararlılık.
// [STABLE] PCMU: Yüksek kalite, kayıpsız.
// [STABLE] PCMA: Cızırtılı
// ============================================================================

// Modül dışa aktarımları
pub use g729::{G729Encoder, G729Decoder};
pub use pcmu::{PcmuEncoder, PcmuDecoder};
// 
pub use pcma::{PcmaEncoder, PcmaDecoder};


// G.722 dışa aktarımı da kapatıldı.
// pub use g722::G722; 

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecType {
    G729 = 18,
    
    PCMU = 0,
    
    PCMA = 8,
    
    // G722 = 9, // [İPTAL]: Enum varyantı kaldırıldı.

}

impl CodecType {
    pub fn sample_rate(&self) -> u32 {
        match self {
            CodecType::PCMU | CodecType::PCMA | CodecType::G729 => 8000,
            // CodecType::G722 => 16000,
        }
    }

    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
            // BIRINCI TEKLIFIMIZ
            18 => Some(CodecType::G729),
            // FALBACK TEKLIFI
            0 => Some(CodecType::PCMU),
            // BASKA CARE YOK !!! durumlarıs
            8 => Some(CodecType::PCMA),
            // 9 => Some(CodecType::G722),
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
    pub fn create_encoder(codec: CodecType) -> Box<dyn Encoder> {
        match codec {
            CodecType::G729 => Box::new(G729Encoder::new()),
            CodecType::PCMU => Box::new(pcmu::PcmuEncoder {}),
            CodecType::PCMA => Box::new(pcma::PcmaEncoder {}),
            // CodecType::G722 => Box::new(g722::G722::new()),
        }
    }

    pub fn create_decoder(codec: CodecType) -> Box<dyn Decoder> {
        match codec {
            CodecType::G729 => Box::new(G729Decoder::new()),
            CodecType::PCMU => Box::new(pcmu::PcmuDecoder {}),
            CodecType::PCMA => Box::new(pcma::PcmaDecoder {}),
            // CodecType::G722 => Box::new(g722::G722::new()),
        }
    }
}