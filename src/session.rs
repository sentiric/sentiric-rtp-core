// sentiric-rtp-core/src/session.rs

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tracing::{info, debug};

/// RtpEndpoint: Dinamik hedef kilitlenme mantığı (Symmetric RTP).
#[derive(Debug, Clone)]
pub struct RtpEndpoint {
    target_addr: Arc<Mutex<Option<SocketAddr>>>,
    is_latched: Arc<Mutex<bool>>,
}

impl RtpEndpoint {
    pub fn new(initial_target: Option<SocketAddr>) -> Self {
        Self {
            target_addr: Arc::new(Mutex::new(initial_target)),
            is_latched: Arc::new(Mutex::new(false)),
        }
    }

    /// Gelen paketin adresine kilitlenir. 
    /// Docker ve NAT senaryolarında Master otoritedir.
    pub fn latch(&self, source_addr: SocketAddr) -> bool {
        let mut latched = self.is_latched.lock().unwrap();
        let mut target = self.target_addr.lock().unwrap();

        // Zaten aynı adrese kilitliysek bir şey yapma
        if *latched && *target == Some(source_addr) {
            return false;
        }

        // Kilitlenme (Latching)
        if !*latched {
            info!("🔒 [LATCH] Medya hedefi kilitlendi: {}", source_addr);
        } else {
            debug!("🔄 [ROAMING] Medya hedefi güncellendi: {}", source_addr);
        }

        *target = Some(source_addr);
        *latched = true;
        true
    }

    pub fn get_target(&self) -> Option<SocketAddr> {
        *self.target_addr.lock().unwrap()
    }

    pub fn reset(&self) {
        let mut latched = self.is_latched.lock().unwrap();
        let mut target = self.target_addr.lock().unwrap();
        *target = None;
        *latched = false;
    }
}