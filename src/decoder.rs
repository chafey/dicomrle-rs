//use std::io::{Read};
use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryFrom;
use std::error::Error;
use std::io::Cursor;

#[allow(dead_code)]
fn decode(_encoded: &mut Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    // DICOM RLE Header is 64 bytes (16 u32s)

    let mut _reader = Cursor::new(&_encoded);

    // read number of segments from first 4 bytes of header (u32)
    let _num_segments: u32 = _reader.read_u32::<LittleEndian>()?;
    let _num_segs: usize = usize::try_from(_num_segments)?;
    println!("_num_segments = {:?} _num_segs={:?}", _num_segments, _num_segs);

    // read the starting position of each segment from header (u32)
    let mut segment_start_positions: [u32; 15] = [0; 15];
    for segment in 0..15 {
        segment_start_positions[segment] = _reader.read_u32::<LittleEndian>()?;
        if segment < _num_segs {
            println!("{:?}  = {:?}", segment, segment_start_positions[segment]);
        }
    }

    // iterate over each segment and decode them
    let mut decoded: Vec<u8> = Vec::new();
    // TODO: deal with hardcoded size
    decoded.resize(512 * 512 * 2, 0); // 512 x 512 x 16 bit

    for segment in 0.._num_segs {
        println!("decoding segment {:?}", segment);
        let start_index:usize = segment_start_positions[segment] as usize;
        let mut end_index:usize = segment_start_positions[segment+1] as usize;
        if segment == (_num_segs - 1) {
            end_index = _encoded.len();
        }

        let mut in_index:usize = start_index;
        let mut out_index = _num_segs - 1 - segment;
        while in_index < end_index {
            let n:u8 = _encoded[in_index];
            in_index += 1;
            if n <= 127 {
                // literal run case - copy them
                let _num_raw_bytes = n + 1;
                for _ in 0.._num_raw_bytes {
                    let _raw_value = _encoded[in_index];
                    in_index += 1;
                    decoded[out_index] = _raw_value;
                    out_index += _num_segs;
                }
            } else if n > 128 {
                // replicated run of values case
                let mut _run_length = n as i8;
                let _run_value = _encoded[in_index];
                in_index += 1;
                //println!("run of {:?} bytes where value = {:?}", -_run_length, _run_value);
                let _run_length2 = (0 - _run_length) as i32 + 1;
                for _ in 0.._run_length2 {
                    decoded[out_index] = _run_value;
                    out_index += _num_segs;
                }
            } else {
                // output nothing
                println!("OUTPUT NOTHING 128");
            }
         }
    }
    
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::error::Error;
    use super::decode;
    use std::io::{Read};

    #[test]
    fn happy_path() -> Result<(), Box<dyn Error>> {
        // read ct1.rle
        let mut rle = File::open("tests/rleimage/ct2.rle")?;
        let mut encoded = Vec::new();
        // read the whole file
        rle.read_to_end(&mut encoded)?;

        println!("Length of file = {:?}", encoded.len());

        // decode it
        let decoded = decode(&mut encoded)?;
        
        println!("decoded length = {:?}", decoded.len());

        // read ct2.raw
        let raw = File::open("tests/rawimage/ct2.raw")?;

        // compare decoded buffer with ct1.raw
        let mut index = 0;
        let mut _byte_iter = raw.bytes();
        while let Some(byte) = _byte_iter.next() {
            //println!("index = {:?}", index);
            assert_eq!(byte.unwrap(), decoded[index]);
            index +=1 ;
        }

        Ok(())
    }
}