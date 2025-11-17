use crate::Error;

// Both overview and sub grid headers have 11 fields of 16 bytes each.
pub(super) const HEADER_SIZE: usize = 11 * 16;

// Buffer offsets for the NTv2 overview header
const HEAD_NUM_RECORDS: usize = 8; // (i32) number of records in the file

/// This NTv2 grid parser is based on the following documents:
/// - https://web.archive.org/web/20140127204822if_/http://www.mgs.gov.on.ca:80/stdprodconsume/groups/content/@mgs/@iandit/documents/resourcelist/stel02_047447.pdf
/// - http://mimaka.com/help/gs/html/004_NTV2%20Data%20Format.htm
/// - https://github.com/Esri/ntv2-file-routines/blob/master/README.md
///
/// And inspired by existing implementations in
/// - https://github.com/proj4js/proj4js/blob/master/lib/nadgrid.js
/// - https://github.com/3liz/proj4rs/blob/main/src/nadgrids/grid.rs
pub struct NTv2Parser {
    buf: Box<[u8]>,
    is_big_endian: bool,
}

impl NTv2Parser {
    pub fn new(buf: Box<[u8]>) -> Self {
        // A NTv2 header is expected to have 11 records
        let is_big_endian = buf[HEAD_NUM_RECORDS] != 11;
        Self { buf, is_big_endian }
    }

    pub fn get_f64(&self, offset: usize) -> f64 {
        match self.is_big_endian {
            true => f64::from_be_bytes(self.buf[offset..offset + 8].try_into().unwrap()),
            false => f64::from_le_bytes(self.buf[offset..offset + 8].try_into().unwrap()),
        }
    }

    pub fn get_f32(&self, offset: usize) -> f32 {
        match self.is_big_endian {
            true => f32::from_be_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
            false => f32::from_le_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
        }
    }

    pub fn get_u32(&self, offset: usize) -> u32 {
        match self.is_big_endian {
            true => u32::from_be_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
            false => u32::from_le_bytes(self.buf[offset..offset + 4].try_into().unwrap()),
        }
    }

    pub fn get_str(&self, offset: usize, len: usize) -> Result<&str, Error> {
        std::str::from_utf8(&self.buf[offset..offset + len]).map_err(Error::from)
    }

    pub fn cmp_str(&self, offset: usize, s: &str) -> bool {
        self.get_str(offset, s.len())
            .map(|x| x == s)
            .unwrap_or(false)
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }
}
