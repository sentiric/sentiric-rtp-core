// src/codecs/mod.rs

pub mod g711;
pub mod g729;

pub use g711::G711;
pub use g729::G729;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodecType {
    PCMU = 0,
    PCMA = 8,
    G729 = 18,
}

/// Tüm ses kodlayıcıların uygulaması gereken ortak arayüz
pub trait Encoder {
    /// 16-bit PCM (Mono, 8000Hz) verisini alır, sıkıştırılmış byte dizisi döner.
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8>;
}