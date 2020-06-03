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

    #[allow(dead_code)]
    pub fn images_are_same(first: &Vec<u8>, second: &Vec<u8>) {
        let mut index = 0;
        let mut _byte_iter = first.bytes();
        while let Some(byte) = _byte_iter.next() {
            let decoded = byte.unwrap();
            let raw = second[index];

            if decoded != raw {
                assert!(false, format!("different found at position {} {}!={}", index, decoded, raw));
            }
            index +=1;
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