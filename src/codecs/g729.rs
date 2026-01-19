// sentiric-rtp-core/src/codecs/g729.rs

use super::{Encoder, Decoder, CodecType};
use std::ffi::c_void;

extern "C" {
    fn initBcg729EncoderChannel(enableVAD: u8) -> *mut c_void;
    fn closeBcg729EncoderChannel(context: *mut c_void);
    fn bcg729Encoder(context: *mut c_void, inputFrame: *const i16, bitStream: *mut u8, bitStreamLength: *mut u8);

    // Decoder
    fn initBcg729DecoderChannel() -> *mut c_void;
    fn closeBcg729DecoderChannel(context: *mut c_void);
    fn bcg729Decoder(context: *mut c_void, bitStream: *const u8, bitStreamLength: u8, frameErasure: u8, sidFrame: u8, rfc3389Payload: u8, outputFrame: *mut i16);
}

// --- ENCODER ---
pub struct G729Encoder {
    context: *mut c_void,
}

impl G729Encoder {
    pub fn new() -> Self {
        unsafe {
            let ctx = initBcg729EncoderChannel(0);
            if ctx.is_null() { panic!("G.729 Encoder Init Failed"); }
            G729Encoder { context: ctx }
        }
    }
}

impl Drop for G729Encoder {
    fn drop(&mut self) {
        unsafe {
            if !self.context.is_null() { closeBcg729EncoderChannel(self.context); }
        }
    }
}

unsafe impl Send for G729Encoder {}

impl Encoder for G729Encoder {
    fn get_type(&self) -> CodecType { CodecType::G729 }

    fn encode(&mut self, pcm_samples: &[i16]) -> Vec<u8> {
        let mut output = Vec::with_capacity(pcm_samples.len() / 8);
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

// --- DECODER ---
pub struct G729Decoder {
    context: *mut c_void,
}

impl G729Decoder {
    pub fn new() -> Self {
        unsafe {
            let ctx = initBcg729DecoderChannel();
            if ctx.is_null() { panic!("G.729 Decoder Init Failed"); }
            G729Decoder { context: ctx }
        }
    }
}

impl Drop for G729Decoder {
    fn drop(&mut self) {
        unsafe {
            if !self.context.is_null() { closeBcg729DecoderChannel(self.context); }
        }
    }
}

unsafe impl Send for G729Decoder {}

impl Decoder for G729Decoder {
    fn get_type(&self) -> CodecType { CodecType::G729 }

    fn decode(&mut self, payload: &[u8]) -> Vec<i16> {
        let mut output = Vec::with_capacity(payload.len() * 8);
        for chunk in payload.chunks(10) {
            if chunk.len() < 2 { continue; }
            let mut pcm_buf = [0i16; 80];
            unsafe {
                bcg729Decoder(
                    self.context,
                    chunk.as_ptr(),
                    chunk.len() as u8,
                    0, 0, 0,
                    pcm_buf.as_mut_ptr()
                );
            }
            output.extend_from_slice(&pcm_buf);
        }
        output
    }
}