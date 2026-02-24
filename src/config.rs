// sentiric-rtp-core/src/config.rs
use crate::codecs::CodecType;
use std::env;

/// Platform genelinde geçerli Medya Anayasası.
/// Hangi kodeklerin, hangi sırayla ve hangi ayarlarla kullanılacağını belirler.
#[derive(Clone, Debug)]
pub struct AudioProfile {
    /// SDP'de teklif edilecek kodeklerin sıralı listesi (Öncelik sırasına göre).
    pub codecs: Vec<CodecConfig>,
    /// Paketleme süresi (ms). Standart: 20ms.
    pub ptime: u8,
}

#[derive(Clone, Debug)]
pub struct CodecConfig {
    pub codec: CodecType,
    pub payload_type: u8,
    pub name: &'static str,
    pub rate: u32,
    pub fmtp: Option<&'static str>,
}

impl Default for AudioProfile {
    fn default() -> Self {
        // [YENİ]: Çevresel değişkenden tercih edilen kodeki al (Varsayılan G729)
        let preferred = env::var("PREFERRED_AUDIO_CODEC")
            .unwrap_or_else(|_| "G729".to_string())
            .to_uppercase();

        let mut profile = Self {
            ptime: 20, // Telekom standardı: 20ms paketler
            codecs: vec![
                
                // --- SES KODEKLERİ ---

                // 1. G.729 (Bant genişliği dostu, lisans gerektirmez - bcg729)
                CodecConfig {
                    codec: CodecType::G729,
                    payload_type: 18,
                    name: "G729",
                    rate: 8000,
                    fmtp: Some("annexb=no"),
                },

                // 2. PCMU (En güvenli, en uyumlu, düşük işlemci yükü)
                CodecConfig {
                    codec: CodecType::PCMU,
                    payload_type: 0,
                    name: "PCMU",
                    rate: 8000,
                    fmtp: None,
                },   

                // 3. PCMA (Avrupa standardı)
                CodecConfig {
                    codec: CodecType::PCMA,
                    payload_type: 8,
                    name: "PCMA",
                    rate: 8000,
                    fmtp: None,
                },
                
                // --- SİNYAL KODEKLERİ ---
                // 4. DTMF (Tuşlama)
                CodecConfig {
                    codec: CodecType::TelephoneEvent,
                    payload_type: 101,
                    name: "telephone-event",
                    rate: 8000,
                    fmtp: Some("0-16"),
                },
            ],
        };

        // Eğer tercih edilen kodek listede varsa, onu bulup EN BAŞA (Priority 1) yerleştirir.
        if let Some(pos) = profile.codecs.iter().position(|c| c.name == preferred) {
            let pref_codec = profile.codecs.remove(pos);
            profile.codecs.insert(0, pref_codec);
        }

        profile
    }
}

impl AudioProfile {
    /// Tercih edilen birincil **SES** kodeğini döndürür (DTMF hariç).
    pub fn preferred_audio_codec(&self) -> CodecType {
        self.codecs.iter()
            .find(|c| c.codec != CodecType::TelephoneEvent) // DTMF'i atla
            .map(|c| c.codec)
            .unwrap_or(CodecType::PCMU) // Hiçbiri yoksa Fallback
    }

    pub fn get_by_payload(&self, pt: u8) -> Option<CodecConfig> {
        self.codecs.iter().find(|c| c.payload_type == pt).cloned()
    }
}