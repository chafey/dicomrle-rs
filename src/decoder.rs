use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryFrom;
use std::io::Cursor;
use std::time::{Instant};
use crate::error::{Error};
use crate::decode_diagnostics::{DecodeDiagnostics};

#[allow(dead_code)]
fn decode(encoded: &Vec<u8>, decoded: &mut Vec<u8>) -> Result<DecodeDiagnostics, Error> {

    let now = Instant::now();
    let mut result = DecodeDiagnostics::new();

    // NOTE: DICOM RLE Header is 64 bytes (16 u32s)
    // 1st u32 is the number of segments
    // 2nd..16th u32 are positions of first byte in each segment
    let mut reader = Cursor::new(&encoded);

    // read number of segments from first 4 bytes of header (u32)
    // NOTE: use of unwrap is safe here because we are not targeting CPUs with
    // less than 32 bits
    // TODO: consider renaming num_segs to segment_count?
    let num_segs: usize = usize::try_from(reader.read_u32::<LittleEndian>()?).unwrap();

    // Validate num_segs <= 15 (as per dicom standard)
    if num_segs > 15 {
        return Err(Error::Format("invalid number of segments".to_owned()));
    }

    // read the starting position of each segment from header (u32)
    // TODO: consider changing segment_start_positions to be a tuple of start/end indexes for each segment
    let mut segment_start_positions: [usize; 16] = [0; 16];
    for segment in 0..15 {
        segment_start_positions[segment] = reader.read_u32::<LittleEndian>()? as usize;
        // if we have a non zero offset for a segment that we shouldn't have, 
        // set unexpected_segment_offsets to true
        if segment >= num_segs && segment_start_positions[segment] != 0 {
            result.unexpected_segment_offsets = true;
        }
    }

    // set the starting position of the segment following the number of segments we
    // actually have to the encoded buffer length so we can bound the segment
    // during decode below
    segment_start_positions[num_segs] = encoded.len();

    // iterate over each segment and decode them
    // TODO: consider renaming segment to segment_number or segment_index?
    for segment in 0..num_segs {
        let segment_start_index = segment_start_positions[segment];
        let segment_end_index = segment_start_positions[segment+1];
        let mut in_index = segment_start_index;
        
        // If two segments, we assume we have 16 bit grayscale data which requires us to 
        // read MSB first followed by LSB.  If not two segments, we just do normal byte
        // ordering for 8 bit grayscale and 8 bit color images
        let mut out_index = if num_segs == 2 {num_segs - 1 - segment} else { segment };
        
        // decode the segment by iterating from the start index to the
        // end index
        while in_index < segment_end_index {
            
            // read control byte
            let control = encoded[in_index];
            in_index += 1;
           
            if control <= 127 {
                // literal run case - copy them
                
                let _num_raw_bytes = control + 1;
                for _ in 0.._num_raw_bytes {
                    // read literal encoded byte safely
                    if in_index == segment_end_index {
                        break;
                    }
                    let _raw_value = encoded[in_index];
                    in_index += 1;

                    // write literal byte safely
                    if out_index < decoded.len() {
                        decoded[out_index] = _raw_value;
                    }
                    out_index += num_segs;
                }
            } else if control > 128 {
                // replicated run of values case

                // make sure were not past end of segment
                if in_index == segment_end_index {
                    continue;
                }
                
                // get the run value
                let run_value = encoded[in_index];
                in_index += 1;

                // calculate the run length
                let run_length = (0 - control as i8) as i32 + 1;

                // write out run_length count of run_value safely
                for _ in 0..run_length {
                    if out_index < decoded.len() {
                        decoded[out_index] = run_value;
                    }
                    out_index += num_segs;
                }
            } else {
                // output nothing, but set the useless_marker_count for diagnostic purposes
                result.useless_marker_count = result.useless_marker_count + 1;
            }
        }

        // Check for underflow on last segment
        if out_index < decoded.len() {
            result.underflow = true;
        }
    }

    result.duration = now.elapsed();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::error::{Error};
    use super::decode;
    use std::io::{Read};
    use byteorder::{LittleEndian, WriteBytesExt};

    fn read_file(filepath: &str) -> Result<Vec<u8>, Error> {
        let mut file = File::open(filepath)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn images_are_same(first: &Vec<u8>, second: &Vec<u8>) {
        let mut index = 0;
        let mut _byte_iter = first.bytes();
        while let Some(byte) = _byte_iter.next() {
            assert_eq!(byte.unwrap(), second[index]);
            index +=1 ;
        }
    }

    fn compare_rle_to_raw(image_name: &str, encoded_size: usize) -> Result<(), Error> {
        // read rle encoded image
        let encoded = read_file(&format!("tests/rleimage/{}.rle", image_name))?;

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(encoded_size, 0);

        // decode it
        let result = decode(&encoded, &mut decoded)?;
        assert_ne!(result.duration.as_micros(), 0);
        assert_eq!(result.underflow, false);
        assert_eq!(result.useless_marker_count, 0);
        assert_eq!(result.unexpected_segment_offsets, false);

        // read raw image
        let raw  = read_file(&format!("tests/rawimage/{}.raw", image_name))?;

        // compare decoded buffer with raw image
        images_are_same(&decoded, &raw);

        Ok(())
    }

    #[test]
    fn verify_ct_decode() -> Result<(), Error> {
        compare_rle_to_raw("ct", 512 * 512 * 2)?;
        Ok(())
    }

    #[test]
    fn verify_ct1_decode() -> Result<(), Error> {
        compare_rle_to_raw("ct1", 512 * 512 * 2)?; 
        Ok(())
    }

    #[test]
    fn verify_ct2_decode() -> Result<(), Error> {
        compare_rle_to_raw("ct2", 512 * 512 * 2)?;
        Ok(())
    }

    #[test]
    fn verify_us1_decode() -> Result<(), Error> {
        compare_rle_to_raw("us1", 640 * 480 * 3)?; 
        Ok(())
    }

    #[test]
    fn verify_rf1_decode() -> Result<(), Error> {
        compare_rle_to_raw("rf1", 512 * 512 * 1)?; 
        Ok(())
    }

    #[test]
    fn truncated_image_underflows() -> Result<(), Error> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;
        encoded.resize(encoded.len() - 1024, 0); // truncate the last 1024 bytes which should cause underflow

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0);

        // decode it
        let result = decode(&encoded, &mut decoded)?;
        assert_eq!(result.underflow, true);

        Ok(())
    }

    #[test]
    fn unexpected_segment_offsets_detected() -> Result<(), Error> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;

        // add an unexpected segment offset 
        encoded[8] = 1;

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0);

        // decode it
        let result = decode(&encoded, &mut decoded)?;
        assert_eq!(result.unexpected_segment_offsets, true);

        Ok(())
    }

    #[test]
    fn useless_marker_count_detected() -> Result<(), Error> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;

        // add a useless marker in the last byte of the last segment since it is ignored
        // due to even buffer padding
        let last_marker = encoded.len() -1;
        encoded[last_marker] = 128;

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0);

        // decode it
        let result = decode(&encoded, &mut decoded)?;
        assert_eq!(result.useless_marker_count, 1);

        Ok(())
    }

    #[test]
    fn verify_max_segment_count_check() -> Result<(), Error> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;

        // Modify the segment count to be 16
        let mut num_segments: Vec<u8> = Vec::new();
        num_segments.write_u32::<LittleEndian>(16).unwrap();
        encoded[0] = num_segments[0];
        encoded[1] = num_segments[1];
        encoded[2] = num_segments[2];
        encoded[3] = num_segments[3];

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0); 

        // decode it
        let result = decode(&encoded, &mut decoded);
        assert!(result.is_err(), "decode image with 16 segments should return error");

        Ok(())
    }

    #[test]
    fn verify_incomplete_header_at_num_segments_fails() -> Result<(), Error> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;

        // Resize buffer so num segments cannot be fully read
        encoded.resize(2, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0); 

        // decode it
        let result = decode(&encoded, &mut decoded);
        assert!(result.is_err(), "decode image with length 2 should return error");

        Ok(())
    }

    #[test]
    fn verify_incomplete_header_at_segment_offset_fails() -> Result<(), Error> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;

        // Resize buffer so num segments cannot be fully read
        encoded.resize(5, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0); 

        // decode it
        let result = decode(&encoded, &mut decoded);
        assert!(result.is_err(), "decode image with length 5 should return error");

        Ok(())
    }


}