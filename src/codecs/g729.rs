// sentiric-rtp-core/src/codecs/g729.rs

use super::{Encoder, CodecType};
use std::ffi::c_void;

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
            let ctx = initBcg729EncoderChannel(0); 
            if ctx.is_null() { panic!("G.729 Init Failed"); }
            G729 { context: ctx }
        }
    }
}

impl Drop for G729 {
    fn drop(&mut self) {
        unsafe {
            if !self.context.is_null() { closeBcg729EncoderChannel(self.context); }
        }
    }
}

unsafe impl Send for G729 {}

impl Encoder for G729 {
    fn get_type(&self) -> CodecType {
        CodecType::G729
    }

    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        let mut output = Vec::new();
        // G.729 10ms frame (80 samples) bekler
        for chunk in pcm_samples.chunks(80) {
            if chunk.len() != 80 { continue; } 

            let mut bitstream = [0u8; 10]; 
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