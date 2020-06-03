use crate::error::{Error};
use crate::diagnostics::{DecodeDiagnostics};
use crate::decode_segment::*;
use crate::get_segments::{get_segments};

// If two segments, we assume we have 16 bit grayscale data which requires us to 
// read MSB first followed by LSB.  If not two segments, we just do normal byte
// ordering for 8 bit grayscale and 8 bit color images
fn calculate_start_index(segment_count: usize, segment_index: usize) -> usize {
    if segment_count == 2 {
        segment_count - 1 - segment_index
    } else { 
        segment_index 
    }
}

#[allow(dead_code)]
pub fn decode(encoded: &Vec<u8>, decoded: &mut Vec<u8>) -> Result<DecodeDiagnostics, Error> {

    let segments = get_segments(encoded)?;

    let mut decode_result = DecodeDiagnostics::new();

    let segment_count = segments.len();

    for segment_index in 0..segment_count {

        let segment = segments[segment_index];

        let start_index = calculate_start_index(segment_count, segment_index);

        let result = decode_segment(segment, &mut decoded[start_index..], segment_count);

        if result.bytes_decoded != decoded.len() / segment_count {
            decode_result.incomplete_decode = true;
        }

        decode_result.decode_segment_results.push(result);
    }

    Ok(decode_result)
}

#[cfg(test)]
mod tests {
    use crate::test::tests::*;
    use super::decode;

    #[test]
    fn verify_ct_decode() {
        compare_rle_to_raw("ct", 512 * 512 * 2).unwrap();
    }

    #[test]
    fn verify_ct1_decode() {
        compare_rle_to_raw("ct1", 512 * 512 * 2).unwrap();
    }

    #[test]
    fn verify_ct2_decode() {
        compare_rle_to_raw("ct2", 512 * 512 * 2).unwrap();
    }

    #[test]
    fn verify_us1_decode() {
        compare_rle_to_raw("us1", 640 * 480 * 3).unwrap();
    }

    #[test]
    fn verify_rf1_decode() {
        compare_rle_to_raw("rf1", 512 * 512 * 1).unwrap();
    }

    #[test]
    fn verify_partial_rf1_decode() {
        // read rle encoded image
        let mut encoded = crate::test::tests::read_file(&"tests/rleimage/rf1.rle").unwrap();

        encoded.resize(encoded.len() - 1024, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0);

        // decode it
        let result = decode(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, true);
        assert_eq!(result.useless_marker_count, 0);
        assert_eq!(result.unexpected_segment_offsets, false);
    }

    #[test]
    fn verify_partial_ct1_decode() {
        let mut encoded = crate::test::tests::read_file(&"tests/rleimage/ct1.rle").unwrap();
        encoded.resize(encoded.len() - 1024, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 2, 0);

        // decode it
        let result = decode(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, true);
        assert_eq!(result.useless_marker_count, 0);
        assert_eq!(result.unexpected_segment_offsets, false);
    }

    #[test]
    fn verify_partial_ct2_decode() {
        let mut encoded = crate::test::tests::read_file(&"tests/rleimage/ct2.rle").unwrap();
        encoded.resize(encoded.len() / 2, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 2, 0);

        // decode it
        let result = decode(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, true);
        assert_eq!(result.useless_marker_count, 0);
        assert_eq!(result.unexpected_segment_offsets, false);
    }

    #[test]
    fn verify_partial_us1_decode() {
        let mut encoded = crate::test::tests::read_file(&"tests/rleimage/us1.rle").unwrap();
        encoded.resize(encoded.len() - 150000, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(640 * 480 * 3, 0);

        // decode it
        let result = decode(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, true);
        assert_eq!(result.useless_marker_count, 0);
        assert_eq!(result.unexpected_segment_offsets, false);
    }

}