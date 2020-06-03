
// the result of decoding a single rle segment
pub struct DecodeSegmentResult {
    // the number of bytes actually decoded
    pub bytes_decoded: usize,

    // true if the encoded data included a literal run but did not have
    // enough bytes in the buffer for the literal run length.  This can 
    // occur if:
    //  * incomplete buffer stream
    //  * bug in encoder
    //  * zero padding on last byte (valid as per DICOM)
    pub literal_run_underflow: bool,

    // true if the encoded data included a replicated run but did not have
    // enough bytes in the buffer for the run value.  This can occur if:
    //  * incomplete buffer stream
    //  * bug in encoder
    pub replicated_run_underflow: bool,

    // true if the decoding would overflow the decoded buffer.  This can occur if:
    //  * bug in encoder
    //  * caller did not allocate big enough buffer for encoded
    pub decoded_overflow: bool,

    // true if an invalid control byte of value 128 was encountered.  This is illegal
    // as per the DICOM standard and indicates a bug in the encoder
    pub invalid_prefix: bool,
}

/// Decodes a single DICOM RLE Segment
/// 
/// # Arguments
///
/// * `segment`   - The encoded RLE segment
/// 
/// * `decoded`   - The decoded bytes will be stored here.  Must be presized to
///                 the expected number of decoded bytes.
///
/// * `increment` - The number of bytes to increment after each byte is decoded.
///                 This is usually the number of segments. 
pub fn decode_segment(segment: &[u8], decoded: &mut[u8], increment: usize) -> DecodeSegmentResult {
    let mut segment_index = 0;
    let mut decoded_index = 0;
    
    let mut result = DecodeSegmentResult {
        bytes_decoded: 0, 
        literal_run_underflow: false,
        replicated_run_underflow: false,
        decoded_overflow: false,
        invalid_prefix: false
    };

    while segment_index < segment.len() {

        let control = segment[segment_index];
        segment_index += 1;

        if control <= 127 { // literal run of values case

            // calculate the literal run length
            let literal_run_length = (control + 1) as usize;

            // detect if we will read past the end of segment.  This can happen if:
            //  * incomplete buffer stream
            //  * bug in encoder
            //  * zero padding on last byte (valid as per DICOM)
            if (segment_index + literal_run_length) > segment.len() {
                result.literal_run_underflow = true;
                break;
            }

            // detect if will write past end of decoded buffer. This can happen if:
            //  * bug in encoder
            //  * caller did not allocate big enough buffer for encoded
            if (decoded_index + (literal_run_length * increment)) > decoded.len() {
                result.decoded_overflow = true;
                break;
            }

            // copy run_length run_values to decoded vector
            for _ in 0..literal_run_length {
                decoded[decoded_index] = segment[segment_index];
                decoded_index = decoded_index + increment;
                segment_index += 1;
            }
        } else if control > 128 { // replicated run of values case

            // calculate the run length
            let run_length = (0 - control as i8) as usize + 1;

            // detect if will read past end of segment.  This can happen if:
            //  * incomplete buffer stream
            //  * bug in encoder
            if (segment_index + 1) > segment.len() {
                result.replicated_run_underflow = true;
                break;
            }
            // detect if will write past end of decoded buffer. This can happen if:
            //  * bug in encoder
            //  * caller did not allocate big enough buffer for encoded
            if (decoded_index + (run_length - 1 * increment)) >= decoded.len() {
                result.decoded_overflow = true;
                break;
            }

            // get the run value
            let run_value = segment[segment_index];
            segment_index += 1;

            // write out the run to decoded buffer
            for _ in 0..run_length {
                decoded[decoded_index] = run_value;
                decoded_index = decoded_index + increment;
            }
        } else {
            // a control value of 128 is illegal as per the DICOM standard
            // http://dicom.nema.org/medical/Dicom/2016e/output/chtml/part05/sect_G.3.html
            result.invalid_prefix = true;
            break;
        }
    }

    result.bytes_decoded = decoded_index / increment;

    result
}

#[cfg(test)]
mod tests {
    use super::decode_segment;

    fn compare(expected: &[u8], actual: &[u8]) {
        assert_eq!(expected.len(), actual.len());
        for index in 0..expected.len() {
            assert_eq!(expected[index], actual[index]);
        }
    }

    #[test]
    fn literal_then_run() {
        let segment = vec![0,0,255,0];
        let mut decoded = Vec::new();
        decoded.resize(3, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(3, result.bytes_decoded);
        assert_eq!(false, result.literal_run_underflow);
        assert_eq!(false, result.replicated_run_underflow);
        assert_eq!(false, result.decoded_overflow);
        assert_eq!(false, result.invalid_prefix);
        compare(&[0,0,0], &decoded);
    }

    #[test]
    fn run_then_literal() {
        let segment = vec![255,0,0,0];
        let mut decoded = Vec::new();
        decoded.resize(3, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(3, result.bytes_decoded);
        assert_eq!(false, result.literal_run_underflow);
        assert_eq!(false, result.replicated_run_underflow);
        assert_eq!(false, result.decoded_overflow);
        assert_eq!(false, result.invalid_prefix);
        compare(&[0,0,0], &decoded);
    }

    #[test]
    fn run_only() {
        let segment = vec![255,0];
        let mut decoded = Vec::new();
        decoded.resize(2, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(2, result.bytes_decoded);
        assert_eq!(false, result.literal_run_underflow);
        assert_eq!(false, result.replicated_run_underflow);
        assert_eq!(false, result.decoded_overflow);
        assert_eq!(false, result.invalid_prefix);
        compare(&[0,0], &decoded);
    }

    #[test]
    fn literal_only() {
        let segment = vec![0,0];
        let mut decoded = Vec::new();
        decoded.resize(1, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(1, result.bytes_decoded);
        assert_eq!(false, result.literal_run_underflow);
        assert_eq!(false, result.replicated_run_underflow);
        assert_eq!(false, result.decoded_overflow);
        assert_eq!(false, result.invalid_prefix);
        compare(&[0], &decoded);
    }

    #[test]
    fn invalid_prefix() {
        let segment = vec![128];
        let mut decoded = Vec::new();
        decoded.resize(0, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(0, result.bytes_decoded);
        assert_eq!(false, result.literal_run_underflow);
        assert_eq!(false, result.replicated_run_underflow);
        assert_eq!(false, result.decoded_overflow);
        assert_eq!(true, result.invalid_prefix);
    }

    #[test]
    fn literal_run_underflow() {
        let segment = vec![0];
        let mut decoded = Vec::new();
        decoded.resize(0, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(0, result.bytes_decoded);
        assert_eq!(true, result.literal_run_underflow);
        assert_eq!(false, result.replicated_run_underflow);
        assert_eq!(false, result.decoded_overflow);
        assert_eq!(false, result.invalid_prefix);
    }

    #[test]
    fn replicated_run_underflow() {
        let segment = vec![255];
        let mut decoded = Vec::new();
        decoded.resize(0, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(0, result.bytes_decoded);
        assert_eq!(false, result.literal_run_underflow);
        assert_eq!(true, result.replicated_run_underflow);
        assert_eq!(false, result.decoded_overflow);
        assert_eq!(false, result.invalid_prefix);
    }

    #[test]
    fn decoded_overflow_on_literal_run() {
        let segment = vec![0,0];
        let mut decoded = Vec::new();
        decoded.resize(0, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(0, result.bytes_decoded);
        assert_eq!(false, result.literal_run_underflow);
        assert_eq!(false, result.replicated_run_underflow);
        assert_eq!(true, result.decoded_overflow);
        assert_eq!(false, result.invalid_prefix);
    }

    #[test]
    fn decoded_overflow_on_replicated_run() {
        let segment = vec![255,0];
        let mut decoded = Vec::new();
        decoded.resize(0, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(0, result.bytes_decoded);
        assert_eq!(false, result.literal_run_underflow);
        assert_eq!(false, result.replicated_run_underflow);
        assert_eq!(true, result.decoded_overflow);
        assert_eq!(false, result.invalid_prefix);
    }

    #[test]
    fn zero_length_segment() {
        let segment = vec![];
        let mut decoded = Vec::new();
        decoded.resize(0, 0);
        let result = decode_segment(&segment, &mut decoded, 1);
        assert_eq!(0, result.bytes_decoded);
        assert_eq!(false, result.literal_run_underflow);
        assert_eq!(false, result.replicated_run_underflow);
        assert_eq!(false, result.decoded_overflow);
        assert_eq!(false, result.invalid_prefix);
    }
}