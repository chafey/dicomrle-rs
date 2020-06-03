use crate::error::{Error};
use std::io::Cursor;
use std::convert::TryFrom;
use byteorder::{LittleEndian, ReadBytesExt};

#[allow(dead_code)]
pub fn read_header(header_bytes: &[u8]) -> Result<Vec<usize>, Error> {
    // The DICOM RLE header is 64 bytes, validate to make sure we have
    // at least 64 bytes
    if header_bytes.len() < 64 {
        return Err(Error::Format("unexpected eof reading header".to_owned()));
    }

    // Create a Cursor so we can convert bytes to u32s
    let mut reader = Cursor::new(&header_bytes);

    // The first u32 in the header is the number of segments - read it and validate
    // it.  Note that the se of unwrap here is safe because we previously
    // validated there is enough bytes to read from and we are not targeting CPU's <
    // 32 bits so u32 will always convert into usize without loss
    let segment_count = usize::try_from(reader.read_u32::<LittleEndian>().unwrap()).unwrap();

    // validate number of segments
    if segment_count > 15 {
        return Err(Error::Format("invalid header - cannot have more than 15 segments".to_owned()));
    }
    if segment_count == 0 {
        return Err(Error::Format("invalid header - cannot have zero segments".to_owned()));
    }

    // read each segment offset into a vector
    let mut segment_offsets:Vec<usize> = Vec::new();

    for _ in 0..segment_count {
        let segment_offset = usize::try_from(reader.read_u32::<LittleEndian>().unwrap()).unwrap();
        segment_offsets.push(segment_offset);
    }

    // validate segment_offset #1 is 64
    if segment_offsets[0] != 64 {
        return Err(Error::Format("invalid header - segment 1 offset must be 64".to_owned()));
    }

    // validate each segment offset is > the one before it
    for segment_index in 1..segment_count {
        if segment_offsets[segment_index] < segment_offsets[segment_index - 1] {
            return Err(Error::Format("invalid header - unexpected value for segment offset".to_owned()));
        }
    }

    Ok(segment_offsets)
}

#[cfg(test)]
mod tests {
    use super::read_header;
    use byteorder::{LittleEndian, ByteOrder};

    fn make_header(values: &mut Vec<u32>) -> Vec<u8> {
        // make sure we have exactly 16 u32s
        values.resize(16, 0);

        // allocate size for the header bytes
        let mut header = Vec::new();
        header.resize(64, 0);

        // write the u32s to the header bytes
        LittleEndian::write_u32_into(&values, &mut header);

        header
    }

    #[test]
    fn one_segment_header() {
        let encoded = make_header(&mut vec![1,64]);

        let header = read_header(&encoded).unwrap();

        assert_eq!(header.len(), 1);
        assert_eq!(header[0], 64);
    }

    #[test]
    fn two_segment_header() {
        let encoded = make_header(&mut vec![2,64,128]);

        let header = read_header(&encoded).unwrap();

        assert_eq!(header.len(), 2);
        assert_eq!(header[0], 64);
        assert_eq!(header[1], 128);
    }

    #[test]
    fn three_segment_header() {
        let encoded = make_header(&mut vec![3,64,128,256]);

        let header = read_header(&encoded).unwrap();

        assert_eq!(header.len(), 3);
        assert_eq!(header[0], 64);
        assert_eq!(header[1], 128);
        assert_eq!(header[2], 256);
    }

    #[test]
    #[should_panic]
    fn zero_segments_panics() {
        let encoded = make_header(&mut vec![0]);

        read_header(&encoded).unwrap();
    }

    #[test]
    #[should_panic]
    fn more_than_15_segments_panics() {
        let encoded = make_header(&mut vec![16,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78]);

        read_header(&encoded).unwrap();
    }

    #[test]
    #[should_panic]
    fn header_less_than_64_bytes_panics() {
        let mut encoded = make_header(&mut vec![3,64,128,256]);
        encoded.resize(1, 0);

        read_header(&encoded).unwrap();
    }
}