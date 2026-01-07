// src/codecs/g729.rs

use super::Encoder;
use std::ffi::c_void;
use libc::{c_int, int16_t, uint8_t};

// C Fonksiyonlarının Tanımları
extern "C" {
    // bcg729/include/bcg729/encoder.h
    fn initBcg729EncoderChannel(enableVAD: uint8_t) -> *mut c_void;
    fn closeBcg729EncoderChannel(context: *mut c_void);
    fn bcg729Encoder(context: *mut c_void, inputFrame: *const int16_t, bitStream: *mut uint8_t, bitStreamLength: *mut uint8_t);
}

pub struct G729 {
    context: *mut c_void,
}

impl G729 {
    pub fn new() -> Self {
        unsafe {
            let ctx = initBcg729EncoderChannel(0); // VAD kapalı (0)
            if ctx.is_null() {
                panic!("G.729 Encoder başlatılamadı! (Out of memory?)");
            }
            G729 { context: ctx }
        }
    }
}

impl Drop for G729 {
    fn drop(&mut self) {
        unsafe {
            if !self.context.is_null() {
                closeBcg729EncoderChannel(self.context);
            }
        }
    }
}

// Thread-safe olduğunu garanti etmeliyiz (Raw pointer taşıdığı için)
unsafe impl Send for G729 {}

impl Encoder for G729 {
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        // G.729 frame boyutu: 80 sample (10ms)
        // Çıktı: 10 byte
        let mut output = Vec::new();
        
        // Gelen veriyi 80'lik bloklara böl
        for chunk in pcm_samples.chunks(80) {
            if chunk.len() != 80 { continue; } // Tam frame değilse atla (padding yapılabilir)

            let mut bitstream = [0u8; 10]; // G.729 output buffer (10 bytes)
            let mut out_len: u8 = 0;

            unsafe {
                bcg729Encoder(
                    self.context,
                    chunk.as_ptr(),
                    bitstream.as_mut_ptr(),
                    &mut out_len
                );
            }
            output.extend_from_slice(&bitstream[..out_len as usize]);
        }
        output
    }
}