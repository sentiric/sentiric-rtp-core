// src/wav.rs

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

        // Basit WAV Header Kontrolü (RIFF....WAVEfmt )
        if buffer.len() < 44 || &buffer[0..4] != b"RIFF" || &buffer[8..12] != b"WAVE" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Geçersiz WAV formatı"));
        }

        // 16-bit Mono PCM olduğunu varsayıyoruz (Sentiric standardı)
        // Header'ı atla (ilk 44 byte standart header)
        let data_part = &buffer[44..];
        
        let samples: Vec<i16> = data_part
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        Ok(WavAudio {
            samples,
            sample_rate: 8000, // Varsayılan
        })
    }
}