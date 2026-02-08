#!/bin/bash
# -------------------------------------------------------------------------
# Sentiric RTP Core Codec Doğrulama Scripti (v4 - Inspector Mode)
# -------------------------------------------------------------------------

ROOT_DIR=$(dirname "$(dirname "$(readlink -f "$0")")")
ASSETS_DIR="$ROOT_DIR/assets"
OUTPUT_DIR="$ROOT_DIR/target/codec_output"
SRC_ASSET="$ASSETS_DIR/reference_src_24k.wav"
NB_ASSET="$ASSETS_DIR/reference_nb_8k.wav"
WB_ASSET="$ASSETS_DIR/reference_wb_16k.wav"

mkdir -p "$OUTPUT_DIR"
rm -f "$OUTPUT_DIR"/*
echo "📦 Çıktı Dizini Hazırlandı."

# Ses dosyasını hazırla
ffmpeg -i "$SRC_ASSET" -acodec pcm_s16le -ac 1 -ar 8000 "$NB_ASSET" -y > /dev/null 2>&1
echo "✅ Asset Hazır: $NB_ASSET"

# ---------------------------------------------------------
# 1. RÖNTGEN MODU (INSPECTOR)
# ---------------------------------------------------------
echo -e "\n🔍 FAZ 0: KODEK RÖNTGENİ (Değer Analizi)"
echo "-------------------------------------------"
cargo run --example codec_inspector
echo "-------------------------------------------"

# ---------------------------------------------------------
# 2. UNIT TESTLER
# ---------------------------------------------------------
echo -e "\n🔬 FAZ 1: Matematiksel Bütünlük Testleri"
cargo test -- --nocapture
# Hata olsa bile devam et, wav dosyalarını görelim

# ---------------------------------------------------------
# 3. İŞİTSEL TESTLER
# ---------------------------------------------------------
echo -e "\n🎙️ FAZ 2: İşitsel Doğrulama (WAV Üretimi)"

CODECS=("pcma" "pcmu" "g729" "g722")

for CODEC in "${CODECS[@]}"; do
    INPUT_ASSET="$NB_ASSET"
    if [ "$CODEC" == "g722" ]; then
        INPUT_ASSET="$WB_ASSET"
    fi

    cargo run --example codec_lab -- "$INPUT_ASSET" "$CODEC" > "$OUTPUT_DIR/${CODEC}_log.txt" 2>&1
    
    if [ -f "$ROOT_DIR/output_${CODEC}.wav" ]; then
        mv "$ROOT_DIR/output_${CODEC}.wav" "$OUTPUT_DIR/output_${CODEC}.wav"
        PSNR=$(grep -oP 'Kalite \(PSNR\)\s*:\s*\K[0-9]+\.[0-9]+' "$OUTPUT_DIR/${CODEC}_log.txt")
        echo "   ✅ [$CODEC] WAV OK. PSNR: $PSNR dB"
    else
        echo "   🚨 [$CODEC] HATA. Log: $OUTPUT_DIR/${CODEC}_log.txt"
    fi
done

echo -e "\n✅ TAMAMLANDI: $OUTPUT_DIR"