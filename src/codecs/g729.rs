// src/codecs/g729.rs

use super::Encoder;
use std::ffi::c_void;

// C Fonksiyonlarının Tanımları (Rust Tipleriyle)
extern "C" {
    fn initBcg729EncoderChannel(enableVAD: u8) -> *mut c_void;
    fn closeBcg729EncoderChannel(context: *mut c_void);
    fn bcg729Encoder(context: *mut c_void, inputFrame: *const i16, bitStream: *mut u8, bitStreamLength: *mut u8);
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

// Thread-safe olduğunu garanti etmeliyiz
unsafe impl Send for G729 {}

impl Encoder for G729 {
    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        let mut output = Vec::new();
        
        // G.729 frame boyutu: 80 sample (10ms)
        for chunk in pcm_samples.chunks(80) {
            if chunk.len() != 80 { continue; }

            let mut bitstream = [0u8; 10]; // G.729 output buffer
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