#!/bin/bash
# -------------------------------------------------------------------------
# Sentiric RTP Core Codec Doğrulama Scripti (v3 - Fault Tolerant)
# BİR TEST BAŞARISIZ OLSA BİLE DİĞERLERİNİ ÇALIŞTIRIR.
# -------------------------------------------------------------------------

# Dizin tanımları
ROOT_DIR=$(dirname "$(dirname "$(readlink -f "$0")")")
ASSETS_DIR="$ROOT_DIR/assets"
OUTPUT_DIR="$ROOT_DIR/target/codec_output"
SRC_ASSET="$ASSETS_DIR/reference_src_24k.wav"
NB_ASSET="$ASSETS_DIR/reference_nb_8k.wav"
WB_ASSET="$ASSETS_DIR/reference_wb_16k.wav"

mkdir -p "$OUTPUT_DIR"
rm -f "$OUTPUT_DIR"/*
echo "📦 Çıktı Dizini Hazırlandı: $OUTPUT_DIR"

# Ses dosyasını hazırla
ffmpeg -i "$SRC_ASSET" -acodec pcm_s16le -ac 1 -ar 8000 "$NB_ASSET" -y > /dev/null 2>&1
echo "✅ Narrowband Asset Hazır: $NB_ASSET"

# =========================================================================
# FAZ 1: MATEMATİKSEL BÜTÜNLÜK TESTLERİ (UNIT TESTS)
# =========================================================================
echo -e "\n🔬 FAZ 1: Matematiksel Bütünlük Testleri Başlatılıyor..."
cargo test -- --nocapture
TEST_RESULT=$? # Test sonucunu sakla ama script'i durdurma

if [ $TEST_RESULT -ne 0 ]; then
    echo "⚠️ UYARI: Matematiksel testlerden bazıları BAŞARISIZ OLDU."
else
    echo "✅ Matematiksel testlerin TÜMÜ BAŞARILI."
fi

# =========================================================================
# FAZ 2: İŞİTSEL DOĞRULAMA (CODEC LAB - BAĞIMSIZ ÇALIŞTIRMA)
# Bu faz, yukarıdaki test başarısız olsa bile çalışır.
# =========================================================================
echo -e "\n🎙️ FAZ 2: İşitsel Doğrulama Testleri Başlatılıyor (Başarısız Olanlar Atlanabilir)"

CODECS=("pcma" "pcmu" "g729" "g722")

for CODEC in "${CODECS[@]}"; do
    echo "▶️ [$CODEC] Laboratuvar Testi Başlatıldı..."
    
    # Uygun ses dosyasını seç
    INPUT_ASSET="$NB_ASSET"
    if [ "$CODEC" == "g722" ]; then
        INPUT_ASSET="$WB_ASSET"
    fi

    # Codec Lab'ı çalıştır
    cargo run --example codec_lab -- "$INPUT_ASSET" "$CODEC" > "$OUTPUT_DIR/${CODEC}_log.txt" 2>&1
    
    # WAV dosyasının oluşup oluşmadığını kontrol et
    if [ -f "$ROOT_DIR/output_${CODEC}.wav" ]; then
        mv "$ROOT_DIR/output_${CODEC}.wav" "$OUTPUT_DIR/output_${CODEC}.wav"
        PSNR=$(grep -oP 'Kalite \(PSNR\)\s*:\s*\K[0-9]+\.[0-9]+' "$OUTPUT_DIR/${CODEC}_log.txt")
        echo "   ✅ BAŞARILI: [$CODEC] için WAV dosyası oluşturuldu. PSNR: $PSNR dB"
    else
        echo "   🚨 HATA: [$CODEC] için WAV dosyası OLUŞTURULAMADI. Muhtemelen kod içinde panik yaşandı."
        echo "      Detaylar için: $OUTPUT_DIR/${CODEC}_log.txt"
    fi
done

# =========================================================================
# RAPOR
# =========================================================================
echo -e "\n========================================================"
echo "✅ KODEK DOĞRULAMA TESTLERİ TAMAMLANDI"
echo "Çıktı ve log dosyaları: $OUTPUT_DIR"
echo "========================================================"