// sentiric-rtp-core/src/session.rs

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
// use crate::net_utils::{is_private_ip, is_public_ip}; // ARTIK GEREK YOK

#[derive(Debug, Clone)]
pub struct RtpEndpoint {
    target_addr: Arc<Mutex<Option<SocketAddr>>>,
    initial_addr: Option<SocketAddr>,
    is_latched: Arc<Mutex<bool>>,
}

impl RtpEndpoint {
    pub fn new(initial_target: Option<SocketAddr>) -> Self {
        RtpEndpoint {
            target_addr: Arc::new(Mutex::new(initial_target)),
            initial_addr: initial_target,
            is_latched: Arc::new(Mutex::new(false)),
        }
    }

    /// Latching Mantığı (DÜZELTİLDİ: Docker Dostu)
    pub fn latch(&self, source_addr: SocketAddr) -> bool {
        let mut latched_guard = self.is_latched.lock().unwrap();
        let mut target_guard = self.target_addr.lock().unwrap();

        // 1. Zaten aynı adrese kilitliysek çık.
        if *latched_guard && *target_guard == Some(source_addr) {
            return false;
        }

        // --- İPTAL EDİLEN FİLTRE ---
        // Docker Bridge ağında dış paketler Gateway IP'si (10.x veya 172.x) ile görünür.
        // Bu yüzden Private IP'leri engellemek, Docker'da Latching'i bozar.
        // Artık gelen her paketi geçerli kabul ediyoruz.
        // ---------------------------

        // 2. Durum: Latching Uygula ve Logla
        if let Some(init) = self.initial_addr {
            if init != source_addr {
                if !*latched_guard {
                    info!("🔄 NAT LATCH: SDP ({}) != Socket ({}). Hedef güncellendi.", init, source_addr);
                } else {
                     info!("🔄 MOBİL ROAMING: Hedef güncellendi -> {}", source_addr);
                }
            } else if !*latched_guard {
                 info!("✅ İLK HEDEF: Hedef kilitlendi -> {}", source_addr);
            }
        } else if !*latched_guard {
             info!("✅ İLK HEDEF (SDP Yok): Hedef kilitlendi -> {}", source_addr);
        }

        *target_guard = Some(source_addr);
        *latched_guard = true;
        return true;
    }

    pub fn get_target(&self) -> Option<SocketAddr> {
        *self.target_addr.lock().unwrap()
    }
    
    pub fn reset(&self) {
        let mut target_guard = self.target_addr.lock().unwrap();
        *target_guard = self.initial_addr;
        *self.is_latched.lock().unwrap() = false;
        warn!("⚠️ RTP Hedefi sıfırlandı (Reset).");
    }
}