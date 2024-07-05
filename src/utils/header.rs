#![allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub a_magic: Vec<u8>,
    pub a_flags: u8,
    pub a_cpu: u8,
    pub a_hdrlen: u8,
    pub a_unused: u8,
    pub a_version: u16,
    pub a_text: u32,
    pub a_data: u32,
    pub a_bss: u32,
    pub a_entry: u32,
    pub a_total: u32,
    pub a_syms: u32,
}

impl Header {
    pub fn new(bytes_data: &[u8]) -> Self {
        if bytes_data.len() < 32 {
            panic!("there's no enough length to extract header");
        }
        Self {
            a_magic: Vec::from(&bytes_data[0..2]),
            a_flags: u8::from(bytes_data[2]),
            a_cpu: u8::from(bytes_data[3]),
            a_hdrlen: u8::from(bytes_data[4]),
            a_unused: u8::from(bytes_data[5]),
            a_version: u16::from_le_bytes((&bytes_data[6..8]).try_into().unwrap()),
            a_text: u32::from_le_bytes((&bytes_data[8..12]).try_into().unwrap()),
            a_data: u32::from_le_bytes((&bytes_data[12..16]).try_into().unwrap()),
            a_bss: u32::from_le_bytes((&bytes_data[16..20]).try_into().unwrap()),
            a_entry: u32::from_le_bytes((&bytes_data[20..24]).try_into().unwrap()),
            a_total: u32::from_le_bytes((&bytes_data[24..28]).try_into().unwrap()),
            a_syms: u32::from_le_bytes((&bytes_data[28..32]).try_into().unwrap()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::utils::header::Header;

    #[test]
    fn test_header() {
        let testcases: &[u8] = &[
            0x01, 0x03, 0x20, 0x04, 0x20, 0x00, 0x00, 0x00, 0x40, 0x01, 0x00, 0x00, 0x14, 0x00,
            0x00, 0x00, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0xc0, 0x02, 0x00, 0x00,
        ];
        assert_eq!(
            Header::new(testcases),
            Header {
                a_magic: [0x01, 0x03].to_vec(),
                a_flags: 0x20,
                a_cpu: 0x04,
                a_hdrlen: 0x20,
                a_unused: 0x00,
                a_version: 0x00,
                a_text: 0x00000140,
                a_data: 0x00000014,
                a_bss: 0x00000042,
                a_entry: 0x00000000,
                a_total: 0x00010000,
                a_syms: 0x000002c0,
            }
        )
    }

    #[test]
    fn test_binary_data() {}
}
