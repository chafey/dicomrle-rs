use std::time::{Duration};

#[allow(dead_code)]
pub struct DecodeDiagnostics {
    pub duration: Duration,
    pub underflow: bool,
    pub useless_marker_count: usize,  // useless == marker value 128 
    pub unexpected_segment_offsets: bool
}

impl DecodeDiagnostics {
    pub fn new() -> DecodeDiagnostics {
        DecodeDiagnostics {
            duration: Duration::new(0,0),
            underflow: false,
            useless_marker_count: 0,
            unexpected_segment_offsets: false
        }
    }
}
