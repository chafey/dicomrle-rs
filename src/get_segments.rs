use crate::header::read_header;
use crate::error::{Error};

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

fn header_to_segment_bounds(encoded: &Vec<u8>) -> Result<Vec<(usize, usize)>, Error> {
    let segment_offsets = read_header(encoded)?;

    let segment_bounds = to_segment_bounds(&segment_offsets, encoded.len());

    Ok(segment_bounds)
}

pub fn get_segments(encoded: &Vec<u8>) -> Result<Vec<&[u8]>, Error> {
    let mut foo = Vec::new();

    let segment_bounds = header_to_segment_bounds(encoded)?;

    for segment in segment_bounds {
        if segment.0 > encoded.len() {
            foo.push(&encoded[0..0])
        } else {
            if segment.1 > encoded.len() {
                foo.push(&encoded[segment.0..])
            } else {
                foo.push(&encoded[segment.0..segment.1])
            }
        }
    }

    Ok(foo)
}

#[cfg(test)]
mod tests {
    use super::get_segments;
    use crate::test::tests::{make_header};

    fn make_two_segment_rle_data() -> Vec<u8> {
        let mut encoded = make_header(&mut vec![2,64,67]);

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