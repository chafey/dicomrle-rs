use crate::error::{Error};
use crate::diagnostics::{DecodeDiagnostics};
use crate::decode_segment::*;
use crate::get_segments::{get_segments};
use std::slice;
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

/// Decodes a DICOM RLE Image
/// 
/// # Arguments
///
/// * `encoded`   - The encoded RLE image
///
/// * `decoded`   - The decoded buffer, presized to the expected image size
/// 
#[allow(dead_code)]
pub fn decode(encoded: &[u8], decoded: &mut [u8]) -> Result<DecodeDiagnostics, Error> {

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

pub fn decode_u16(encoded: &[u8], decoded: &mut [u16]) -> Result<DecodeDiagnostics, Error> {
    let mut decoded_u8 = unsafe { 
        let ptr = decoded.as_mut_ptr() as *mut u8; 
        slice::from_raw_parts_mut(ptr, decoded.len() * 2) 
    };
    decode(encoded, &mut decoded_u8)
}

pub fn decode_i16(encoded: &[u8], decoded: &mut [i16]) -> Result<DecodeDiagnostics, Error> {
    let mut decoded_u8 = unsafe { 
        let ptr = decoded.as_mut_ptr() as *mut u8; 
        slice::from_raw_parts_mut(ptr, decoded.len() * 2) 
    };
    decode(encoded, &mut decoded_u8)
}


#[cfg(test)]
mod tests {
    use crate::test::tests::*;
    use super::{decode, decode_i16, decode_u16};
    use std::slice;

    #[test]
    fn verify_ct_decode() {
        compare_rle_to_raw("ct", 512 * 512 * 2).unwrap();
    }

    #[test]
    fn verify_ct_decode_i16() {
        // read rle encoded image
        let encoded = read_file(&format!("tests/rleimage/ct.rle")).unwrap();

        let mut decoded: Vec<i16> = Vec::new();
        decoded.resize(512 * 512, 0);

        // decode it
        let result = decode_i16(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, false);

        // read raw image
        let raw  = read_file(&format!("tests/rawimage/ct.raw")).unwrap();
        
        let raw_i16 = unsafe { 
            let ptr = raw.as_ptr() as *mut i16; 
            slice::from_raw_parts_mut(ptr, raw.len() / 2)
        };

        // compare decoded buffer with raw image
        images_are_same(&decoded, &raw_i16);
    }

    #[test]
    fn verify_ct_decode_u16() {
        // read rle encoded image
        let encoded = read_file(&format!("tests/rleimage/ct.rle")).unwrap();

        let mut decoded: Vec<u16> = Vec::new();
        decoded.resize(512 * 512, 0);

        // decode it
        let result = decode_u16(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, false);

        // read raw image
        let raw  = read_file(&format!("tests/rawimage/ct.raw")).unwrap();
        
        let raw_u16 = unsafe { 
            let ptr = raw.as_ptr() as *mut u16; 
            slice::from_raw_parts_mut(ptr, raw.len() / 2)
        };

        // compare decoded buffer with raw image
        images_are_same(&decoded, &raw_u16);
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
    }

}