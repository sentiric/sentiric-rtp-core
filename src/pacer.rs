// sentiric-rtp-core/src/pacer.rs

use std::thread;
use std::time::{Duration, Instant};

/// HybridPacer: Nano-saniye hassasiyetinde RTP zamanlaması sağlar.
/// 'Sleep' ile CPU'yu korur, 'Spin' ile zamanlamayı yakalar.
pub struct Pacer {
    interval: Duration,
    next_tick: Instant,
}

impl Pacer {
    pub fn new(frame_duration_ms: u64) -> Self {
        Self {
            interval: Duration::from_millis(frame_duration_ms),
            next_tick: Instant::now(),
        }
    }

    #[inline(always)]
    pub fn wait(&mut self) {
        let now = Instant::now();
        
        if now >= self.next_tick {
            // Eğer gecikme varsa, bir sonraki hedefi şimdiye göre ayarla
            self.next_tick = now + self.interval;
            return;
        }

        let remaining = self.next_tick - now;

        // 1. Kaba Bekleme: Eğer 2ms'den fazla varsa OS'e bırak
        if remaining > Duration::from_millis(2) {
            thread::sleep(remaining - Duration::from_millis(2));
        }

        // 2. Hassas Bekleme: Son mikrosaniyeleri yakmak için busy-wait
        while Instant::now() < self.next_tick {
            std::hint::spin_loop();
        }

        self.next_tick += self.interval;
    }

    pub fn reset(&mut self) {
        self.next_tick = Instant::now();
    }
}