#[cfg(test)]
pub mod tests {
    use byteorder::{ByteOrder, LittleEndian};

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
}
