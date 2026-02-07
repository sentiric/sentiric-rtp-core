use crate::rtp::RtpPacket;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// JitterBuffer: Ağdaki gecikme değişimlerini (Jitter) ve sıra hatalarını (Out-of-Order) düzeltir.
pub struct JitterBuffer {
    buffer: BTreeMap<u16, RtpPacket>,
    expected_seq: u16,
    max_capacity: usize,
    buffering_delay: Duration,
    first_packet_time: Option<Instant>,
    initialized: bool,
}

impl JitterBuffer {
    /// Yeni bir Jitter Buffer oluşturur.
    /// capacity: Maksimum paket sayısı (örn: 50 paket = 1 saniye)
    /// delay_ms: Oynatmaya başlamadan önceki tamponlama süresi (örn: 40-60ms)
    pub fn new(capacity: usize, delay_ms: u64) -> Self {
        Self {
            buffer: BTreeMap::new(),
            expected_seq: 0,
            max_capacity: capacity,
            buffering_delay: Duration::from_millis(delay_ms),
            first_packet_time: None,
            initialized: false,
        }
    }

    /// Gelen paketi tampona ekler.
    pub fn push(&mut self, packet: RtpPacket) {
        let seq = packet.header.sequence_number;

        // İlk paket geldiyse sekans takibini başlat
        if !self.initialized {
            self.expected_seq = seq;
            self.first_packet_time = Some(Instant::now());
            self.initialized = true;
        }

        // Eski paketleri yoksay (Late arrival)
        if self.is_late(seq) {
            // Metrics: packet_discarded_late++ (İleride eklenebilir)
            return;
        }

        // Buffer dolduysa en eski paketi at (Overflow protection)
        if self.buffer.len() >= self.max_capacity {
            if let Some(&first_key) = self.buffer.keys().next() {
                self.buffer.remove(&first_key);
                // Eğer attığımız paket tam da beklediğimiz ise, beklentiyi mecburen ilerlet
                if first_key == self.expected_seq {
                    self.expected_seq = self.expected_seq.wrapping_add(1);
                }
            }
        }

        self.buffer.insert(seq, packet);
    }

    /// Oynatılması gereken paketi döndürür.
    /// Eğer paket henüz gelmediyse veya tamponlama süresi dolmadıysa None döner.
    pub fn pop(&mut self) -> Option<RtpPacket> {
        if !self.initialized {
            return None;
        }

        // Başlangıç tamponlama süresi (Buffering Delay) kontrolü
        if let Some(start) = self.first_packet_time {
            if start.elapsed() < self.buffering_delay {
                return None; // Henüz doluyor...
            }
        }

        // Beklenen paket var mı?
        if let Some(packet) = self.buffer.remove(&self.expected_seq) {
            self.expected_seq = self.expected_seq.wrapping_add(1);
            return Some(packet);
        }

        // Beklenen paket yok (Packet Loss veya Gecikme).
        // Eğer buffer'da beklenen paketten çok daha ileri paketler birikmişse (Gap),
        // beklenen paketi "kayıp" kabul edip atlamalıyız.
        if let Some(&next_available_seq) = self.buffer.keys().next() {
            let gap = next_available_seq.wrapping_sub(self.expected_seq);
            
            // Eğer 5 paketten fazla boşluk varsa veya buffer %50 doluysa atla
            // Bu "Catch-up" (Yaklama) mantığıdır.
            if gap > 5 || self.buffer.len() > (self.max_capacity / 2) {
                // Kayıp paketi atla
                // İdealde burada Packet Loss Concealment (PLC) devreye girer.
                self.expected_seq = next_available_seq; 
                let packet = self.buffer.remove(&next_available_seq);
                self.expected_seq = self.expected_seq.wrapping_add(1);
                return packet;
            }
        }

        None
    }

    /// Sequence number wrapping (65535 -> 0) durumunu hesaba katarak
    /// paketin geç kalıp kalmadığını kontrol eder.
    fn is_late(&self, seq: u16) -> bool {
        const THRESHOLD: u16 = 30000;
        
        if seq == self.expected_seq {
            return false;
        }

        if seq < self.expected_seq {
            // Eğer fark çok büyükse wrap-around olmuştur, yani paket yenidir.
            return self.expected_seq - seq < THRESHOLD;
        } else {
            // Eğer fark çok büyükse (örn: beklenen 0, gelen 65000), paket eskidir.
            return seq - self.expected_seq > THRESHOLD;
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.initialized = false;
        self.first_packet_time = None;
    }
}

// --- UNIT TESTS ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtp::RtpHeader;
    use std::thread;

    fn create_dummy_packet(seq: u16) -> RtpPacket {
        RtpPacket {
            header: RtpHeader::new(0, seq, 0, 1234),
            payload: vec![],
        }
    }

    #[test]
    fn test_jitter_buffer_reordering() {
        // 10 kapasite, 50ms gecikme
        let mut jb = JitterBuffer::new(10, 50);
        
        // Paketleri karışık gönderiyoruz: 1, 3, 2
        jb.push(create_dummy_packet(1));
        jb.push(create_dummy_packet(3)); // Erken geldi (Out of order)
        jb.push(create_dummy_packet(2)); // Geç geldi (Reordered)

        // Henüz buffer süresi dolmadı, pop() None dönmeli
        assert!(jb.pop().is_none());

        // Buffer süresini doldur
        thread::sleep(Duration::from_millis(60));

        // Sıralı çıkmalı: 1, 2, 3
        let p1 = jb.pop().expect("Packet 1 should be available");
        assert_eq!(p1.header.sequence_number, 1);

        let p2 = jb.pop().expect("Packet 2 should be available");
        assert_eq!(p2.header.sequence_number, 2);

        let p3 = jb.pop().expect("Packet 3 should be available");
        assert_eq!(p3.header.sequence_number, 3);
        
        // Buffer boş olmalı
        assert!(jb.pop().is_none());
    }

    #[test]
    fn test_loss_skip() {
        let mut jb = JitterBuffer::new(10, 10);
        
        // 1. Paketi gönderdik
        jb.push(create_dummy_packet(100));
        
        thread::sleep(Duration::from_millis(15));
        let _ = jb.pop(); // 100'ü aldık. Beklenen şimdi 101.

        // 101, 102, 103, 104, 105 KAYIP.
        // Birden 106 geldi.
        jb.push(create_dummy_packet(106));

        // Jitter Buffer, 101'i bekliyor ama 106 var.
        // Boşluk (Gap) henüz 5'ten büyük değil ve buffer dolu değil, o yüzden bekler.
        // Ancak biz burada buffer doluluk testini tetiklemek için daha fazla paket basalım.
        
        for i in 107..115 {
            jb.push(create_dummy_packet(i));
        }

        // Şimdi buffer dolmaya yaklaştı, atlama yapmalı.
        let p = jb.pop().expect("Should skip to next available");
        assert_eq!(p.header.sequence_number, 106); // 101-105 arasını atladı
    }
}