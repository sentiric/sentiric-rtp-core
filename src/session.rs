// sentiric-rtp-core/src/session.rs

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// RtpEndpoint, bir RTP oturumunun karşı tarafının (Remote Peer) adresini yönetir.
///
/// "Symmetric RTP" veya "Latching" olarak bilinen tekniği uygular:
/// Başlangıçta SDP'deki IP'ye güvenilir, ancak karşıdan paket geldiği anda
/// hedef adres, paketin geldiği gerçek IP:Port (NAT dış bacağı) ile güncellenir.
#[derive(Debug, Clone)]
pub struct RtpEndpoint {
    // Mutex kullanıyoruz çünkü hem okuma (TX) hem yazma (RX Latching) thread-safe olmalı.
    // Ancak performans için RwLock yerine Mutex tercih ettik (RTP döngüsünde conflict azdır).
    target_addr: Arc<Mutex<Option<SocketAddr>>>,
    initial_addr: Option<SocketAddr>,
    is_latched: Arc<Mutex<bool>>,
}

impl RtpEndpoint {
    /// Yeni bir Endpoint oluşturur.
    /// initial_target: SDP'den okunan IP adresi (Başlangıç hedefi).
    pub fn new(initial_target: Option<SocketAddr>) -> Self {
        RtpEndpoint {
            target_addr: Arc::new(Mutex::new(initial_target)),
            initial_addr: initial_target,
            is_latched: Arc::new(Mutex::new(false)),
        }
    }

    /// Gelen bir pakete göre hedefi günceller (Latching).
    /// return: Eğer hedef değiştilirse `true` döner (Loglama için yararlıdır).
    pub fn latch(&self, source_addr: SocketAddr) -> bool {
        let mut latched_guard = self.is_latched.lock().unwrap();
        let mut target_guard = self.target_addr.lock().unwrap();

        // Eğer henüz kilitlenmediysek VEYA hedef değiştiyse güncelle.
        // NOT: Bazı senaryolarda "Strict Latching" (sadece ilk pakete kilitlen) gerekebilir.
        // Ancak mobil ağlarda IP değişimi olabileceği için "Dynamic Latching" kullanıyoruz.
        if !*latched_guard || *target_guard != Some(source_addr) {
            
            // Eğer başlangıçta bir hedefimiz varsa ve bu ondan farklıysa logla.
            if let Some(init) = self.initial_addr {
                if init != source_addr && !*latched_guard {
                    info!("🔄 NAT LATCH: SDP ({}) != Socket ({}). Hedef güncellendi.", init, source_addr);
                } else if *latched_guard {
                     info!("🔄 MOBİL ROAMING: Hedef güncellendi -> {}", source_addr);
                }
            } else if !*latched_guard {
                 info!("✅ İLK HEDEF: Hedef kilitlendi -> {}", source_addr);
            }

            *target_guard = Some(source_addr);
            *latched_guard = true;
            return true;
        }
        false
    }

    /// Şu anki aktif hedef adresi döndürür.
    pub fn get_target(&self) -> Option<SocketAddr> {
        *self.target_addr.lock().unwrap()
    }
    
    /// Hedefi manuel olarak sıfırlar (Örn: Hold durumunda).
    pub fn reset(&self) {
        let mut target_guard = self.target_addr.lock().unwrap();
        *target_guard = self.initial_addr;
        *self.is_latched.lock().unwrap() = false;
        warn!("⚠️ RTP Hedefi sıfırlandı (Reset).");
    }
}