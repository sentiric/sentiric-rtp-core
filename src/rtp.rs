// sentiric-rtp-core/src/rtp.rs

#[derive(Debug, Clone)]
pub struct RtpHeader {
    pub version: u8,
    pub padding: bool,
    pub extension: bool,
    pub csrc_count: u8,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
}

impl RtpHeader {
    pub fn new(payload_type: u8, seq: u16, ts: u32, ssrc: u32) -> Self {
        Self {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type,
            sequence_number: seq,
            timestamp: ts,
            ssrc,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12);
        let b0 = (self.version << 6) | ((self.padding as u8) << 5) | ((self.extension as u8) << 4) | (self.csrc_count & 0x0F);
        let b1 = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);
        bytes.push(b0);
        bytes.push(b1);
        bytes.extend_from_slice(&self.sequence_number.to_be_bytes());
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(&self.ssrc.to_be_bytes());
        bytes
    }
}

pub struct RtpPacket {
    pub header: RtpHeader,
    pub payload: Vec<u8>,
}

impl RtpPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = self.header.to_bytes();
        out.extend_from_slice(&self.payload);
        out
    }
}

/// RTCP Sender Report (Minimal)
pub struct RtcpPacket {
    pub payload: Vec<u8>,
}

impl RtcpPacket {
    pub fn new_sender_report(ssrc: u32) -> Self {
        let mut bytes = Vec::with_capacity(8);
        bytes.push(0x80); // Version 2, Count 0
        bytes.push(200);  // Type: Sender Report
        bytes.extend_from_slice(&1u16.to_be_bytes()); // Length
        bytes.extend_from_slice(&ssrc.to_be_bytes());
        Self { payload: bytes }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        self.payload.clone()
    }
}