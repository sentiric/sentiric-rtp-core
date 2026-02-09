// sentiric-rtp-core/src/lib.rs

pub mod rtp;
pub mod codecs;
pub mod wav;
pub mod pacer;
pub mod session;
pub mod net_utils;
pub mod jitter_buffer;

pub use rtp::{RtpHeader, RtpPacket, RtcpPacket};

// ============================================================================
// !!! PRODUCTION READY CODECS !!!
// 
// [STABLE] G.729: Düşük bant genişliği, yüksek kararlılık.
// [STABLE] PCMU: Yüksek kalite, kayıpsız.
// [STABLE] PCMA: Cızırtılı
// ============================================================================
pub use codecs::{
    Encoder,
    Decoder,
    CodecType,
    CodecFactory, 
    
    G729Encoder,
    G729Decoder,

    PcmuEncoder,
    PcmuDecoder, 

    PcmaEncoder,
    PcmaDecoder
    
};
pub use wav::WavAudio;
pub use pacer::Pacer;
pub use session::RtpEndpoint;
pub use jitter_buffer::JitterBuffer;