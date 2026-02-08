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
// !!! GELİŞTİRME NOTLARI VE ÜRETİM (PRODUCTION) DURUMU !!!
// 
// Aşağıdaki kodeklerin stabilite durumları test sonuçlarına göre belirlenmiştir:
// 
// [STABLE] G.729: 8000Hz örnekleme ile tam uyumlu, üretim ortamına hazır.
// [BETA]   PCMU: Temel fonksiyonlar çalışıyor, uç durumlar (edge cases) test ediliyor.
// [DEV]    PCMA & G.722: Henüz stabil değil; ses bozulmaları veya paketleme sorunları 
//          mevcut. Üretim ortamında KULLANILMAMALIDIR.
// ============================================================================
pub use codecs::{
    Encoder,
    Decoder,
    CodecType,
    CodecFactory, 
    PcmaEncoder,
    PcmaDecoder,
    PcmuEncoder,
    PcmuDecoder, 
    G729Encoder,
    G729Decoder,
    G722
};
pub use wav::WavAudio;
pub use pacer::Pacer;
pub use session::RtpEndpoint;
pub use jitter_buffer::JitterBuffer;