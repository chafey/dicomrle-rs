use crate::error::{Error};
use std::io::Cursor;
use std::convert::TryFrom;
use byteorder::{LittleEndian, ReadBytesExt};

#[allow(dead_code)]
pub fn read_header(header_bytes: &Vec<u8>) -> Result<Vec<usize>, Error> {
    // The DICOM RLE header is 64 bytes, validate to make sure we have
    // at least 64 bytes
    if header_bytes.len() < 64 {
        return Err(Error::Format("unexpected eof reading header".to_owned()));
    }

    // Create an Cursor so we can convert bytes to u32s
    let mut reader = Cursor::new(&header_bytes);

    // The first u32 in the header is the number of segments - read it and validate
    // it.  Note that the se of unwrap here is safe because we previously
    // validated there is enough bytes to read from and we are not targeting CPU's <
    // 32 bits so u32 will always convert into usize without loss
    let segment_count = usize::try_from(reader.read_u32::<LittleEndian>().unwrap()).unwrap();

    // validate number of segments
    if segment_count > 15 {
        return Err(Error::Format("invalid number of segments".to_owned()));
    }

    // read each segment offset into a vector
    let mut segment_offsets:Vec<usize> = Vec::new();

    for _ in 0..segment_count {
        let entry = usize::try_from(reader.read_u32::<LittleEndian>().unwrap()).unwrap();
        segment_offsets.push(entry);
    }

    Ok(segment_offsets)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::error::{Error};
    use super::read_header;
    use std::io::{Read};

    fn read_file(filepath: &str) -> Result<Vec<u8>, Error> {
        let mut file = File::open(filepath)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
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
        let result = read_header(&encoded);
        if let Err(result) = result  {
            eprintln!("Error: {:#}", result);
        } else {
            assert!(result.is_err(), "decode image with length 5 should return error");
        }

        Ok(())
    }

    #[test]
    fn verify_incomplete_header_at_segment_offset_fails() -> Result<(), Error> {
        // read rle encoded image
        let mut encoded = read_file("tests/rleimage/rf1.rle")?;

        // Resize buffer so first segment offset cannot be read
        encoded.resize(5, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0); 

        // decode it
        let result = read_header(&encoded);
        if let Err(result) = result  {
            eprintln!("Error: {:#}", result);
        } else {
            assert!(false, "decode image with length 5 should return error");
        }
        
        Ok(())
    }
}