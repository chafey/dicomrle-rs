use crate::error::Error;
use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryFrom;
use std::io::Cursor;

// helper function to read usize from the header
fn read_usize(cursor: &mut Cursor<&&[u8]>) -> usize {
    // read u32 from the cursor.  Note that the unwrap() is safe here because
    // we previously validated the length of the header
    let value = cursor.read_u32::<LittleEndian>().unwrap();

    // convert from u32 to usize.  Note that unwrap() is safe here because
    // we are not targeting platforms with less than 32 bits
    usize::try_from(value).unwrap()
}

/// Parses the DICOM RLE Header and returns the starting offset for each
/// segment.  Retruns errors in the following cases
///     1) Header is not long enough
///     2) Number of segments is invalid - must be 1..15 inclusive
///     3) Segment offsets are ascending in value
///
/// # Arguments
///
/// * `header_bytes` - The DICOM RLE Header
///
#[allow(dead_code)]
pub fn read_header(header_bytes: &[u8]) -> Result<Vec<usize>, Error> {
    // The DICOM RLE header is 64 bytes, validate to make sure we have
    // at least 64 bytes
    if header_bytes.len() < 64 {
        return Err(Error::Format("unexpected eof reading header".to_owned()));
    }

    // Create a Cursor on the header bytes so we can read usizes
    let mut reader = Cursor::new(&header_bytes);

    // Read the segment count from the beginning of header
    let segment_count = read_usize(&mut reader);

    // validate number of segments
    if segment_count > 15 {
        return Err(Error::Format(
            "invalid header - cannot have more than 15 segments".to_owned(),
        ));
    }
    if segment_count == 0 {
        return Err(Error::Format(
            "invalid header - cannot have zero segments".to_owned(),
        ));
    }

    // read each segment offset into a vector
    let mut segment_offsets: Vec<usize> = Vec::new();
    for _ in 0..segment_count {
        let segment_offset = read_usize(&mut reader);
        segment_offsets.push(segment_offset);
    }

    // validate segment_offset #1 is 64
    if segment_offsets[0] != 64 {
        return Err(Error::Format(
            "invalid header - segment 1 offset must be 64".to_owned(),
        ));
    }

    // validate each segment offset is > the one before it
    for segment_index in 1..segment_count {
        if segment_offsets[segment_index] < segment_offsets[segment_index - 1] {
            return Err(Error::Format(
                "invalid header - unexpected value for segment offset".to_owned(),
            ));
        }
    }

    Ok(segment_offsets)
}

#[cfg(test)]
mod tests {
    use super::read_header;
    use crate::test::tests::make_header;

    #[test]
    fn one_segment_header() {
        let encoded = make_header(&mut vec![1, 64]);

        let header = read_header(&encoded).unwrap();

        assert_eq!(header.len(), 1);
        assert_eq!(header[0], 64);
    }

    #[test]
    fn two_segment_header() {
        let encoded = make_header(&mut vec![2, 64, 128]);

        let header = read_header(&encoded).unwrap();

        assert_eq!(header.len(), 2);
        assert_eq!(header[0], 64);
        assert_eq!(header[1], 128);
    }

    #[test]
    fn three_segment_header() {
        let encoded = make_header(&mut vec![3, 64, 128, 256]);

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
        let encoded = make_header(&mut vec![
            16, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78,
        ]);

        read_header(&encoded).unwrap();
    }

    #[test]
    #[should_panic]
    fn header_less_than_64_bytes_panics() {
        let mut encoded = make_header(&mut vec![3, 64, 128, 256]);
        encoded.resize(1, 0);

        read_header(&encoded).unwrap();
    }
}
