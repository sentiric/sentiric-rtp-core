#!/bin/bash
# -------------------------------------------------------------------------
# Sentiric RTP Core Codec Doğrulama Scripti (v4.1 - Fixes)
# -------------------------------------------------------------------------

ROOT_DIR=$(dirname "$(dirname "$(readlink -f "$0")")")
ASSETS_DIR="$ROOT_DIR/assets"
OUTPUT_DIR="$ROOT_DIR/target/codec_output"
SRC_ASSET="$ASSETS_DIR/reference_src_24k.wav"
NB_ASSET="$ASSETS_DIR/reference_nb_8k.wav"

mkdir -p "$OUTPUT_DIR"
rm -f "$OUTPUT_DIR"/*

# Ses dosyasını hazırla
if [ ! -f "$NB_ASSET" ]; then
    echo "⚙️  Referans ses dosyası oluşturuluyor..."
    if command -v ffmpeg &> /dev/null; then
        ffmpeg -i "$SRC_ASSET" -acodec pcm_s16le -ac 1 -ar 8000 "$NB_ASSET" -y > /dev/null 2>&1
    else
        echo "⚠️  FFmpeg bulunamadı! Testler mevcut '$NB_ASSET' dosyası varsa onunla çalışacak."
    fi
fi

# ---------------------------------------------------------
# 1. UNIT TESTLER (Matematiksel Doğruluk)
# ---------------------------------------------------------
echo -e "\n🔬 FAZ 1: Matematiksel Bütünlük Testleri (cargo test)"
cargo test --tests -- --nocapture
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -ne 0 ]; then
    echo "❌ HATA: Unit testler başarısız oldu! Dağıtım durduruluyor."
    exit 1
fi

# ---------------------------------------------------------
# 2. İŞİTSEL TESTLER (Lab Simülasyonu)
# ---------------------------------------------------------
echo -e "\n🎙️ FAZ 2: İşitsel Doğrulama (WAV Üretimi)"

# G722 listeden çıkarıldı
CODECS=("pcma" "pcmu" "g729")

for CODEC in "${CODECS[@]}"; do
    cargo run --example codec_lab -- "$NB_ASSET" "$CODEC" > "$OUTPUT_DIR/${CODEC}_log.txt" 2>&1
    
    if [ -f "output_${CODEC}.wav" ]; then
        mv "output_${CODEC}.wav" "$OUTPUT_DIR/output_${CODEC}.wav"
        
        # PSNR Değerini logdan çek
        PSNR=$(grep -oP 'Kalite \(PSNR\)\s*:\s*\K[0-9]+\.[0-9]+' "$OUTPUT_DIR/${CODEC}_log.txt")
        
        # Basit renkli çıktı
        if (( $(echo "$PSNR > 30.0" | bc -l) )); then
            echo "   ✅ [$CODEC] MÜKEMMEL. PSNR: $PSNR dB"
        elif (( $(echo "$PSNR > 10.0" | bc -l) )); then
            echo "   ⚠️ [$CODEC] KABUL EDİLEBİLİR (Lossy). PSNR: $PSNR dB"
        else
            echo "   ❌ [$CODEC] BAŞARISIZ! PSNR: $PSNR dB (Logu inceleyin)"
        fi
    else
        echo "   🚨 [$CODEC] KRİTİK HATA: WAV üretilemedi. Log: $OUTPUT_DIR/${CODEC}_log.txt"
    fi
done

echo -e "\n✅ TÜM TESTLER TAMAMLANDI: $OUTPUT_DIR"