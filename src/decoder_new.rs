use crate::header::read_header;
use crate::error::{Error};
use crate::diagnostics::{DecodeDiagnostics};
use crate::decode_segment::*;

fn to_segment_bounds(segment_offsets:Vec<usize>, encoded_length: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for segment_index in 0..segment_offsets.len()-1 {
        let start = segment_offsets[segment_index];
        let end = segment_offsets[segment_index+1];
        result.push((start, end))
    }
    result.push((segment_offsets[segment_offsets.len()-1], encoded_length));
    result
}

#[allow(dead_code)]
pub fn decode(encoded: &Vec<u8>, decoded: &mut Vec<u8>) -> Result<DecodeDiagnostics, Error> {

    let segment_offsets = read_header(encoded)?;
    let segment_count = segment_offsets.len();

    println!("segment_offsets: {:?}", segment_offsets);

    let segment_bounds = to_segment_bounds(segment_offsets, encoded.len());

    for segment_index in 0..segment_count {

        let segment = segment_bounds[segment_index];

        println!("segment: {:?}", segment);

        // If two segments, we assume we have 16 bit grayscale data which requires us to 
        // read MSB first followed by LSB.  If not two segments, we just do normal byte
        // ordering for 8 bit grayscale and 8 bit color images
        let start_index = if segment_count == 2 {segment_count - 1 - segment_index} else { segment_index };

        let result = decode_segment(&encoded[segment.0..segment.1], &mut decoded[start_index..], segment_count);
        println!("decoded_size: {:?}", result.bytes_decoded);
    }

    let mut _result = DecodeDiagnostics::new();

    Ok(_result)
}

#[cfg(test)]
mod tests {
    use crate::test::tests::*;
    use crate::error::{Error};
    //use crate::diagnostics::{DecodeDiagnostics};
    use super::decode;

    #[test]
    fn verify_ct_decode() -> Result<(), Error> {
        compare_rle_to_raw("ct1", 512 * 512 * 2, decode)?;
        Ok(())
    }
}