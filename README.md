# Sentiric RTP Core

Bu kütüphane, Sentiric VoIP servisleri için RTP paketleme ve ses kodlama işlemlerini yürütür.

## Özellikler
*   **G.729:** `bcg729` kütüphanesi statik olarak derlenir ve gömülür. Harici kurulum gerektirmez.
*   **RTP:** RFC 3550 uyumlu paket oluşturucu.
*   **Sıfır Bağımlılık:** Sadece `cc` (build-time) ve `libc` kullanır.

## Desteklenen Codec'ler
1.  **G.729 (Payload: 18):** 8kbps, yüksek sıkıştırma.
2.  **PCMA (Payload: 8):** G.711 A-law.
3.  **PCMU (Payload: 0):** G.711 u-law.