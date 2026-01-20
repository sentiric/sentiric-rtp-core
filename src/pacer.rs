// sentiric-rtp-core/src/pacer.rs

use std::thread;
use std::time::{Duration, Instant};

/// HybridPacer, RTP paketlerinin gönderim zamanlamasını hassas bir şekilde yönetir.
///
/// Standart işletim sistemi `sleep` fonksiyonları 10-15ms hata payına sahip olabilir.
/// Bu yapı, sürenin büyük kısmını `sleep` ile (CPU'yu yormadan),
/// son 2ms'lik kısmını `spin_loop` ile (CPU'yu aktif kullanarak) bekler.
///
/// Sonuç: <1ms Jitter, Kristal netliğinde ses.
pub struct Pacer {
    interval: Duration,
    next_tick: Instant,
}

impl Pacer {
    /// Yeni bir Pacer oluşturur.
    /// frame_duration: Genellikle 20ms (0.02s)
    pub fn new(frame_duration: Duration) -> Self {
        Pacer {
            interval: frame_duration,
            next_tick: Instant::now(),
        }
    }

    /// Bir sonraki kare zamanına kadar bekler.
    /// Bu fonksiyon BLOKLAYICIDIR (Blocking).
    /// Gerçek zamanlı ses thread'inde çalıştırılmalıdır.
    #[inline(always)]
    pub fn wait(&mut self) {
        let now = Instant::now();
        
        // Eğer zaman geride kaldıysa (gecikme varsa), bekleme yapma, hemen dön.
        // Ancak next_tick'i güncelle ki bir sonraki paketi yakalamaya çalışsın.
        if now >= self.next_tick {
            self.next_tick += self.interval;
            return;
        }

        let remaining = self.next_tick - now;

        // 1. Aşama: Kaba Bekleme (Coarse Wait) - CPU Dostu
        // Eğer 2ms'den fazla varsa işletim sistemine bırak.
        if remaining > Duration::from_millis(2) {
            thread::sleep(remaining - Duration::from_millis(2));
        }

        // 2. Aşama: Hassas Bekleme (Busy Wait) - Düşük Latency
        // Kalan süreyi döngüde yakarak tam zamanında çıkış yap.
        while Instant::now() < self.next_tick {
            std::hint::spin_loop();
        }

        // Bir sonraki hedefi belirle
        self.next_tick += self.interval;
    }

    /// Pacer'ı sıfırlar (Örn: Yayın durup tekrar başladığında).
    pub fn reset(&mut self) {
        self.next_tick = Instant::now();
    }
}