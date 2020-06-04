#[cfg(test)]
pub mod tests {
    use std::fs::File;
    use std::io::{Read};
    use crate::error::{Error};
    use crate::decode::{decode};
    use byteorder::{LittleEndian, ByteOrder};

    pub fn make_header(values: &mut Vec<u32>) -> Vec<u8> {
        // make sure we have exactly 16 u32s
        values.resize(16, 0);

        // allocate size for the header bytes
        let mut header = Vec::new();
        header.resize(64, 0);

        // write the u32s to the header bytes
        LittleEndian::write_u32_into(&values, &mut header);

        header
    }

    #[allow(dead_code)]
    pub fn read_file(filepath: &str) -> Result<Vec<u8>, Error> {
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        Ok(buffer)
    }

    pub fn images_are_same<T>(a: &[T], b: &[T])
    where T: PartialEq + std::fmt::Display
    {
        assert_eq!(a.len(), b.len());
        for i in 0..a.len() {
            if a[1] != b[1] {
                assert!(false, format!("difference found at position {} {}!={}", i, a[i], b[i]));
            }
        }
    }

    #[allow(dead_code)]
    pub fn compare_rle_to_raw(image_name: &str, decoded_size: usize) -> Result<(), Error> {
        // read rle encoded image
        let encoded = crate::test::tests::read_file(&format!("tests/rleimage/{}.rle", image_name))?;

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(decoded_size, 0);

        // decode it
        let result = decode(&encoded, &mut decoded)?;
        assert_eq!(result.incomplete_decode, false);

        // read raw image
        let raw  = read_file(&format!("tests/rawimage/{}.raw", image_name))?;

        // compare decoded buffer with raw image
        images_are_same(&decoded, &raw);

        Ok(())
    }


}