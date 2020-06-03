use crate::header::read_header;
use crate::error::{Error};
use crate::diagnostics::{DecodeDiagnostics};
use crate::decode_segment::*;

fn to_segment_bounds(segment_offsets:&Vec<usize>, encoded_length: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for segment_index in 0..segment_offsets.len()-1 {
        let start = segment_offsets[segment_index];
        let end = segment_offsets[segment_index+1];
        result.push((start, end))
    }
    result.push((segment_offsets[segment_offsets.len()-1], encoded_length));
    result
}

// If two segments, we assume we have 16 bit grayscale data which requires us to 
// read MSB first followed by LSB.  If not two segments, we just do normal byte
// ordering for 8 bit grayscale and 8 bit color images
fn calculate_start_index(segment_count: usize, segment_index: usize) -> usize {
    let start_index = if segment_count == 2 {segment_count - 1 - segment_index} else { segment_index };

    start_index
}

pub fn read_and_validate_header(encoded: &Vec<u8>, allow_partial_decode: bool) -> Result<Vec<usize>, Error> {
    let segment_offsets = read_header(encoded)?;

    // if we are not allowing partial decode, do a quick sanity check to see if
    // any segment offsets in the header are beyond the encoded buffer
    if !allow_partial_decode {
        if let Some(last_offset) = segment_offsets.last() {
            if last_offset > &encoded.len() {
                return Err(Error::Format("unexpected eof decoding segment".to_owned()));
            }
        }
    }

    Ok(segment_offsets)
}

fn header_to_segment_bounds(encoded: &Vec<u8>, allow_partial_decode: bool) -> Result<Vec<(usize, usize)>, Error> {
    let segment_offsets = read_and_validate_header(encoded, allow_partial_decode)?;

    let segment_bounds = to_segment_bounds(&segment_offsets, encoded.len());

    Ok(segment_bounds)
}

#[allow(dead_code)]
pub fn decode(encoded: &Vec<u8>, decoded: &mut Vec<u8>, allow_partial_decode: bool) -> Result<DecodeDiagnostics, Error> {

    let segment_bounds = header_to_segment_bounds(encoded, allow_partial_decode)?;

    let mut decode_result = DecodeDiagnostics::new();

    let segment_count = segment_bounds.len();

    for segment_index in 0..segment_count {

        let segment = segment_bounds[segment_index];

        let start_index = calculate_start_index(segment_count, segment_index);

        let result = decode_segment(&encoded[segment.0..segment.1], &mut decoded[start_index..], segment_count);

        // handle the case where we didn't decode the expected number of bytes
        if result.bytes_decoded != decoded.len() / segment_count {
            decode_result.incomplete_decode = true;
            if !allow_partial_decode {
                return Err(Error::Format("unexpected eof decoding segment".to_owned()));
            }
        }

        decode_result.decode_segment_results.push(result);
    }

    Ok(decode_result)
}

#[cfg(test)]
mod tests {
    use crate::test::tests::*;
    use crate::error::{Error};
    use crate::diagnostics::{DecodeDiagnostics};
    use super::decode;

    fn decode_bridge(encoded: &Vec<u8>, decoded: &mut Vec<u8>) -> Result<DecodeDiagnostics, Error> {
        decode(encoded, decoded, false)
    }

    #[test]
    fn verify_ct_decode() {
        compare_rle_to_raw("ct1", 512 * 512 * 2, decode_bridge).unwrap();
    }
}