
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{Read};
    use dicomrle::error::{Error};
    use dicomrle::decode::{decode, decode_i16, decode_u16};
    use std::slice;

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
        let encoded = read_file(&format!("tests/rleimage/{}.rle", image_name))?;

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

    #[test]
    fn verify_ct_decode() {
        compare_rle_to_raw("ct", 512 * 512 * 2).unwrap();
    }

    #[test]
    fn verify_ct_decode_i16() {
        // read rle encoded image
        let encoded = read_file(&format!("tests/rleimage/ct.rle")).unwrap();

        let mut decoded: Vec<i16> = Vec::new();
        decoded.resize(512 * 512, 0);

        // decode it
        let result = decode_i16(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, false);

        // read raw image
        let raw  = read_file(&format!("tests/rawimage/ct.raw")).unwrap();
        
        let raw_i16 = unsafe { 
            let ptr = raw.as_ptr() as *mut i16; 
            slice::from_raw_parts_mut(ptr, raw.len() / 2)
        };

        // compare decoded buffer with raw image
        images_are_same(&decoded, &raw_i16);
    }

    #[test]
    fn verify_ct_decode_u16() {
        // read rle encoded image
        let encoded = read_file(&format!("tests/rleimage/ct.rle")).unwrap();

        let mut decoded: Vec<u16> = Vec::new();
        decoded.resize(512 * 512, 0);

        // decode it
        let result = decode_u16(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, false);

        // read raw image
        let raw  = read_file(&format!("tests/rawimage/ct.raw")).unwrap();
        
        let raw_u16 = unsafe { 
            let ptr = raw.as_ptr() as *mut u16; 
            slice::from_raw_parts_mut(ptr, raw.len() / 2)
        };

        // compare decoded buffer with raw image
        images_are_same(&decoded, &raw_u16);
    }

    #[test]
    fn verify_ct1_decode() {
        compare_rle_to_raw("ct1", 512 * 512 * 2).unwrap();
    }

    #[test]
    fn verify_ct2_decode() {
        compare_rle_to_raw("ct2", 512 * 512 * 2).unwrap();
    }

    #[test]
    fn verify_us1_decode() {
        compare_rle_to_raw("us1", 640 * 480 * 3).unwrap();
    }

    #[test]
    fn verify_rf1_decode() {
        compare_rle_to_raw("rf1", 512 * 512 * 1).unwrap();
    }

    #[test]
    fn verify_partial_rf1_decode() {
        // read rle encoded image
        let mut encoded = read_file(&"tests/rleimage/rf1.rle").unwrap();

        encoded.resize(encoded.len() - 1024, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 1, 0);

        // decode it
        let result = decode(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, true);
    }

    #[test]
    fn verify_partial_ct1_decode() {
        let mut encoded = read_file(&"tests/rleimage/ct1.rle").unwrap();
        encoded.resize(encoded.len() - 1024, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 2, 0);

        // decode it
        let result = decode(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, true);
    }

    #[test]
    fn verify_partial_ct2_decode() {
        let mut encoded = read_file(&"tests/rleimage/ct2.rle").unwrap();
        encoded.resize(encoded.len() / 2, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(512 * 512 * 2, 0);

        // decode it
        let result = decode(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, true);
    }

    #[test]
    fn verify_partial_us1_decode() {
        let mut encoded = read_file(&"tests/rleimage/us1.rle").unwrap();
        encoded.resize(encoded.len() - 150000, 0);

        let mut decoded: Vec<u8> = Vec::new();
        decoded.resize(640 * 480 * 3, 0);

        // decode it
        let result = decode(&encoded, &mut decoded).unwrap();
        assert_eq!(result.incomplete_decode, true);
    }

}