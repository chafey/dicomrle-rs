use crate::decode_segment::DecodeSegmentResult;

#[allow(dead_code)]
#[derive(Default)]
/// Diagnostic information related to decoding an RLE image
pub struct DecodeDiagnostics {
    /// true if the decoded buffer was not fully populated during the
    /// decode process of the last segment.  This indicates either a
    /// truncated buffer or an invalid rle encoding.  In both cases,
    /// the decoded image cannot be considered complete or valid.
    pub incomplete_decode: bool,

    /// diagnostic information about each decoded segment
    pub decode_segment_results: [Option<DecodeSegmentResult>; 15],
}

impl DecodeDiagnostics {
    pub fn new() -> DecodeDiagnostics {
        DecodeDiagnostics {
            incomplete_decode: false,
            decode_segment_results: [None; 15],
        }
    }
}
