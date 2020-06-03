use crate::decode_segment::{DecodeSegmentResult};

#[allow(dead_code)]
/// Diagnostic information related to decoding an RLE image
pub struct DecodeDiagnostics {
    /// true if the decoded buffer was not fully populated during the
    /// decode process of the last segment.  This indicates either a
    /// truncated buffer or an invalid rle encoding.  In both cases,
    /// the decoded image cannot be considered complete or valid.
    /// Note that a truncated buffer or invalid rle encoding can
    /// also result in an IoError
    pub incomplete_decode: bool,
    
    /// true if the header included non zero offset values for segments not
    /// included in the encoded bitstream.  This condition does not effect
    /// the decoded image 
    pub unexpected_segment_offsets: bool,

    pub decode_segment_results: Vec<DecodeSegmentResult>,

    // TODO: get rid of this..
    pub useless_marker_count: usize
}

impl DecodeDiagnostics {
    pub fn new() -> DecodeDiagnostics {
        DecodeDiagnostics {
            incomplete_decode: false,
            useless_marker_count: 0,
            unexpected_segment_offsets: false,
            decode_segment_results : Vec::new()
        }
    }
}
