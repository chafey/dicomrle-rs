use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryFrom;
use std::error::Error;
use std::io::Cursor;
use std::time::{Duration, Instant};

#[allow(dead_code)]
struct DecodeResult {
    duration: Duration,
    underflow: bool,
    useless_marker_count: usize,  // useless == marker value 128 
    unexpected_segment_offsets: bool
}

#[allow(dead_code)]
fn decode(_encoded: &Vec<u8>, decoded: &mut Vec<u8>) -> Result<DecodeResult, Box<dyn Error>> {

    let now = Instant::now();

    let mut result = DecodeResult {
        duration : Duration::new(0,0), 
        underflow: false, 
        useless_marker_count: 0,
        unexpected_segment_offsets: false
    };

    // DICOM RLE Header is 64 bytes (16 u32s)

    let mut _reader = Cursor::new(&_encoded);

    // read number of segments from first 4 bytes of header (u32)
    let _num_segments: u32 = _reader.read_u32::<LittleEndian>()?;
    let _num_segs: usize = usize::try_from(_num_segments)?;
    //println!("_num_segments = {:?} _num_segs={:?}", _num_segments, _num_segs);

    // TODO: Validate _num_segs <= 15 (as per dicom standard)

    // read the starting position of each segment from header (u32)
    let mut segment_start_positions: [u32; 15] = [0; 15];
    for segment in 0..15 {
        segment_start_positions[segment] = _reader.read_u32::<LittleEndian>()?;
        if segment < _num_segs {
            //println!("{:?}  = {:?}", segment, segment_start_positions[segment]);
        } 
        if segment >= _num_segs {
            // if we have a non zero offset for a segment that we shouldn't have, 
            // set unexpected_segment_offsets to true
            if segment_start_positions[segment] != 0 {
                result.unexpected_segment_offsets = true;
            }
        }
    }

    // iterate over each segment and decode them
    for segment in 0.._num_segs {
        //println!("decoding segment {:?}", segment);
        let start_index:usize = segment_start_positions[segment] as usize;
        let mut end_index:usize = segment_start_positions[segment+1] as usize;
        if segment == (_num_segs - 1) {
            end_index = _encoded.len();
        }

        let mut in_index:usize = start_index;
        // If two segments, we assume we have 16 bit grayscale data which requires us to 
        // read MSB first followed by LSB.  If not two segments, we just do normal byte
        // ordering for 8 bit grayscale and 8 bit color images
        let mut out_index = if _num_segs == 2 {_num_segs - 1 - segment} else { segment };
        while in_index < end_index {
            //println!("in_index={}", in_index);
            //if in_index == 248329 {
            //    let z = 0;
            //}
            let n:u8 = _encoded[in_index];
            in_index += 1;
           
            if n <= 127 {
                // literal run case - copy them
                let _num_raw_bytes = n + 1;
                for _ in 0.._num_raw_bytes {
                    // make sure were not past end of segment
                    if in_index == end_index {
                        break;
                    }
                    let _raw_value = _encoded[in_index];
                    in_index += 1;
                    if out_index < decoded.len() {
                        decoded[out_index] = _raw_value;
                    }
                    out_index += _num_segs;
                }
            } else if n > 128 {
                // make sure were not past end of segment
                if in_index == end_index {
                    continue;
                }
                // replicated run of values case
                let mut _run_length = n as i8;
                let _run_value = _encoded[in_index];
                in_index += 1;
                //println!("run of {:?} bytes where value = {:?}", -_run_length, _run_value);
                let _run_length2 = (0 - _run_length) as i32 + 1;
                for _ in 0.._run_length2 {
                    if out_index < decoded.len() {
                        decoded[out_index] = _run_value;
                    }
                    out_index += _num_segs;
                }
            } else {
                // output nothing
                result.useless_marker_count = result.useless_marker_count + 1;
            }
        }
        // Check for underflow on last segment
        if out_index < decoded.len() {
            result.underflow = true;
        }
    }

    result.duration = now.elapsed();

    //println!("RLE Decode took {} us", result.duration.as_micros());

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::error::Error;
    use super::decode;
    use std::io::{Read};

    fn read_file(filepath: &str) -> Result<Vec<u8>, Box<dyn Error>> {
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

    fn compare_rle_to_raw(image_name: &str, encoded_size: usize) -> Result<(), Box<dyn Error>> {
        // read rle encoded image
        let encoded = read_file(&format!("tests/rleimage/{}.rle", image_name))?;

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(encoded_size, 0); // TODO: see if we can resize without setting to 0

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
    fn verify_ct_decode() -> Result<(), Box<dyn Error>> {
        compare_rle_to_raw("ct", 512 * 512 * 2)?;
        Ok(())
    }

    #[test]
    fn verify_ct1_decode() -> Result<(), Box<dyn Error>> {
        compare_rle_to_raw("ct1", 512 * 512 * 2)?; 
        Ok(())
    }

    #[test]
    fn verify_ct2_decode() -> Result<(), Box<dyn Error>> {
        compare_rle_to_raw("ct2", 512 * 512 * 2)?;
        Ok(())
    }

    #[test]
    fn verify_us1_decode() -> Result<(), Box<dyn Error>> {
        compare_rle_to_raw("us1", 640 * 480 * 3)?; 
        Ok(())
    }

    #[test]
    fn verify_rf1_decode() -> Result<(), Box<dyn Error>> {
        compare_rle_to_raw("rf1", 512 * 512 * 1)?; 
        Ok(())
    }

    #[test]
    fn truncated_image_underflows() -> Result<(), Box<dyn Error>> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;
        encoded.resize(encoded.len() - 1024, 0); // truncate the last 1024 bytes which should cause underflow

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0); // TODO: see if we can resize without setting to 0

        // decode it
        let result = decode(&encoded, &mut decoded)?;
        assert_eq!(result.underflow, true);

        Ok(())
    }

    #[test]
    fn unexpected_segment_offsets_detected() -> Result<(), Box<dyn Error>> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;

        // add an unexpected segment offset 
        encoded[8] = 1;

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0); // TODO: see if we can resize without setting to 0

        // decode it
        let result = decode(&encoded, &mut decoded)?;
        assert_eq!(result.unexpected_segment_offsets, true);

        Ok(())
    }

    #[test]
    fn useless_marker_count_detected() -> Result<(), Box<dyn Error>> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;

        // add a useless marker in the last byte of the last segment since it is ignored
        // due to even buffer padding
        let last_marker = encoded.len() -1;
        encoded[last_marker] = 128;

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0); // TODO: see if we can resize without setting to 0

        // decode it
        let result = decode(&encoded, &mut decoded)?;
        assert_eq!(result.useless_marker_count, 1);

        Ok(())
    }


}