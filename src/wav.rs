// sentiric-rtp-core/src/wav.rs

use std::fs::File;
use std::io::{self, Read};

pub struct WavAudio {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
}

impl WavAudio {
    pub fn read_file(path: &str) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if buffer.len() < 12 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Dosya çok küçük"));
        }

        // RIFF Header Kontrolü
        if &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WAVE" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Geçersiz WAV formatı"));
        }

        let mut pos = 12;
        let mut audio_data_start = 0;
        let mut audio_data_len = 0;

        while pos + 8 < buffer.len() {
            let chunk_id = &buffer[pos..pos+4];
            let chunk_size = u32::from_le_bytes([
                buffer[pos+4], buffer[pos+5], buffer[pos+6], buffer[pos+7]
            ]) as usize;

            // Güvenlik: Chunk size mantıksız büyükse dur
            if pos + 8 + chunk_size > buffer.len() {
                break;
            }

            if chunk_id == b"data" {
                audio_data_start = pos + 8;
                audio_data_len = chunk_size;
                break;
            }

            // RIFF Alignment Fix:
            // Eğer chunk size tek sayı ise, +1 padding byte atla.
            let padding = if chunk_size % 2 != 0 { 1 } else { 0 };
            pos += 8 + chunk_size + padding;
        }

        if audio_data_start == 0 {
            // Fallback: Eğer data chunk bulunamazsa (veya ffmpeg header koymadıysa)
            // Header 44 byte varsay ve gerisini oku.
            if buffer.len() > 44 {
                audio_data_start = 44;
                audio_data_len = buffer.len() - 44;
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Ses verisi bulunamadı"));
            }
        }

        let data_part = &buffer[audio_data_start..audio_data_start + audio_data_len];

        // i16 Dönüşümü
        let samples: Vec<i16> = data_part
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        Ok(WavAudio {
            samples,
            sample_rate: 8000,
        })
    }
}