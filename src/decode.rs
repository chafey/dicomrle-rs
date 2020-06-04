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

        decode_result.decode_segment_results[segment_index] = Some(result);
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

