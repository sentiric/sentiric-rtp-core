# 🧬 RTP Core - Medya Anayasası ve DSP Motoru

**Rol:** Tek Doğruluk Kaynağı (Single Source of Truth) ve Sinyal İşleme Motoru.

## 1. Temel Sorumluluklar

1.  **Medya Anayasası (`config.rs`):**
    *   Platformun desteklediği tüm Kodekleri (G.729, PCMU, PCMA, DTMF) tanımlar.
    *   Kodeklerin öncelik sırasını belirler (Şu an: PCMU > G.729 > PCMA).
    *   RTP paketleme süresini (`ptime`) belirler (Standart: 20ms).
    *   B2BUA ve Media Service, konfigürasyon için **sadece** burayı referans alır.

2.  **Sinyal İşleme (DSP):**
    *   Ham ses verisini (PCM) işler (Resampling, Mixing).
    *   **Upsampling:** 8kHz Telekom sesini -> 16kHz AI sesine çevirir.
    *   **Downsampling:** 16kHz AI sesini -> 8kHz Telekom sesine çevirir (Ortalama alma yöntemiyle).

3.  **Paketleme ve Tamponlama:**
    *   `JitterBuffer`: Ağ gecikmelerini yönetir.
    *   `Pacer`: Paketlerin 20ms aralıklarla, donanım hızında gönderilmesini sağlar.

## 2. Yasaklar (Anti-Patterns)

*   ❌ **Karar Vermez:** "Hangi kodeği kullanayım?" diye sormaz, sadece tanımlı olanı sunar.
*   ❌ **Ağ I/O Yapmaz:** Soket açmaz, sadece byte dizileri (payload) üretir.