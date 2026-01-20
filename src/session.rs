// sentiric-rtp-core/src/session.rs

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};
use crate::net_utils::{is_private_ip, is_public_ip};

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

    /// Akıllı Latching Mantığı
    pub fn latch(&self, source_addr: SocketAddr) -> bool {
        let mut latched_guard = self.is_latched.lock().unwrap();
        let mut target_guard = self.target_addr.lock().unwrap();

        // 1. Durum: Zaten kilitliysek ve kaynak değişmediyse çık.
        if *latched_guard && *target_guard == Some(source_addr) {
            return false;
        }

        // 2. Durum: SMART FILTERING (Kritik Düzeltme)
        // Eğer başlangıç hedefimiz (SDP'den gelen) bir Public IP ise,
        // ve gelen paket bir Private IP'den (Docker Gateway, LAN vb.) geliyorsa,
        // bu pakete kilitlenmek yanlıştır. Muhtemelen NAT/Docker maskelemesidir.
        // Bu durumda SDP'ye sadık kalırız.
        if let Some(init) = self.initial_addr {
            if is_public_ip(init.ip()) && is_private_ip(source_addr.ip()) {
                // Log kirliliği yapmamak için sadece ilk seferde veya nadiren uyarabiliriz
                // Şimdilik sessizce görmezden geliyoruz ki doğru hedefe (Public) atmaya devam etsin.
                return false;
            }
        }

        // 3. Durum: Latching Uygula
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