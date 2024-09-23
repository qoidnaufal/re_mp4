use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::{Read, Seek};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MfhdBox {
    pub version: u8,
    pub flags: u32,
    pub sequence_number: u32,
}

impl Default for MfhdBox {
    fn default() -> Self {
        MfhdBox {
            version: 0,
            flags: 0,
            sequence_number: 1,
        }
    }
}

impl MfhdBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::MfhdBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4
    }
}

impl Mp4Box for MfhdBox {
    fn box_type(&self) -> BoxType {
        self.get_type()
    }

    fn box_size(&self) -> u64 {
        self.get_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("sequence_number={}", self.sequence_number);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for MfhdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;
        let sequence_number = reader.read_u32::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(MfhdBox {
            version,
            flags,
            sequence_number,
        })
    }
}
