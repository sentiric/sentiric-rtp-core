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
        RtpHeader {
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
        
        let b0 = (self.version << 6) 
               | ((self.padding as u8) << 5) 
               | ((self.extension as u8) << 4) 
               | (self.csrc_count & 0x0F);
        bytes.push(b0);

        let b1 = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);
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

// --- YENİ EKLENEN KISIM: RTCP STUB ---
// QoS ve Stream takibi için gerekli minimum yapı
pub struct RtcpPacket {
    pub version: u8,
    pub padding: bool,
    pub count: u8,
    pub packet_type: u8, // 200=SR, 201=RR
    pub length: u16,
    pub ssrc: u32,
}

impl RtcpPacket {
    pub fn new_sender_report(ssrc: u32) -> Self {
        RtcpPacket {
            version: 2,
            padding: false,
            count: 0,
            packet_type: 200, // Sender Report
            length: 6, // Header only stub
            ssrc,
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let b0 = (self.version << 6) | ((self.padding as u8) << 5) | (self.count & 0x1F);
        bytes.push(b0);
        bytes.push(self.packet_type);
        bytes.extend_from_slice(&self.length.to_be_bytes());
        bytes.extend_from_slice(&self.ssrc.to_be_bytes());
        // Gerçek implementasyonda burada NTP timestamp, RTP timestamp vb. olmalı.
        // Şimdilik bağlantıyı canlı tutmak için yeterli.
        bytes
    }
}