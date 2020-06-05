use crate::error::Error;
use crate::header::read_header;

fn to_segment_bounds(segment_offsets: &[usize], encoded_length: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for segment_index in 0..segment_offsets.len() - 1 {
        let start = segment_offsets[segment_index];
        let end = segment_offsets[segment_index + 1];
        result.push((start, end))
    }
    result.push((segment_offsets[segment_offsets.len() - 1], encoded_length));
    result
}

fn header_to_segment_bounds(encoded: &[u8]) -> Result<Vec<(usize, usize)>, Error> {
    let segment_offsets = read_header(encoded)?;

    let segment_bounds = to_segment_bounds(&segment_offsets, encoded.len());

    Ok(segment_bounds)
}

/// Returns a vector of u8 slices for each segment in the RLE encoded bitstream.
/// If the encoded buffer is truncated, the correct number of u8 slices will
/// be returned, but their length may be zero or truncated.
///
/// # Arguments
///
/// * `encoded`   - The encoded RLE image
///
/// * `decoded`   - The decoded buffer, presized to the expected image size
///
pub fn get_segments(encoded: &[u8]) -> Result<Vec<&[u8]>, Error> {
    let mut segments = Vec::new();

    let segment_bounds = header_to_segment_bounds(encoded)?;

    for segment in segment_bounds {
        if segment.0 > encoded.len() {
            segments.push(&encoded[0..0])
        } else if segment.1 > encoded.len() {
            segments.push(&encoded[segment.0..])
        } else {
            segments.push(&encoded[segment.0..segment.1])
        }
    }

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::get_segments;
    use crate::test::tests::make_header;

    fn make_two_segment_rle_data() -> Vec<u8> {
        let mut encoded = make_header(&mut vec![2, 64, 67]);

        encoded.resize(70, 0);
        encoded[64] = 255; // literal run of 2
        encoded[65] = 0;
        encoded[66] = 0;
        encoded[67] = 255; // segment 2
        encoded[68] = 0;
        encoded[69] = 0;

        encoded
    }

    #[test]
    fn two_segments() {
        let encoded = make_two_segment_rle_data();

        let segments = get_segments(&encoded).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].len(), 3);
        assert_eq!(segments[1].len(), 3);
    }

    #[test]
    fn two_segments_truncated_in_second() {
        let mut encoded = make_two_segment_rle_data();
        encoded.resize(encoded.len() - 1, 0);

        let segments = get_segments(&encoded).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].len(), 3);
        assert_eq!(segments[1].len(), 2);
    }

    #[test]
    fn two_segments_truncated_in_first() {
        let mut encoded = make_two_segment_rle_data();
        encoded.resize(encoded.len() - 4, 0);

        let segments = get_segments(&encoded).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].len(), 2);
        assert_eq!(segments[1].len(), 0);
    }
}
