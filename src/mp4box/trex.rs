use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::{Read, Seek};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct TrexBox {
    pub version: u8,
    pub flags: u32,
    pub track_id: u32,
    pub default_sample_description_index: u32,
    pub default_sample_duration: u32,
    pub default_sample_size: u32,
    pub default_sample_flags: u32,
}

impl TrexBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::TrexBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 20
    }
}

impl Mp4Box for TrexBox {
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
        let s = format!(
            "track_id={} default_sample_duration={}",
            self.track_id, self.default_sample_duration
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TrexBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let track_id = reader.read_u32::<BigEndian>()?;
        let default_sample_description_index = reader.read_u32::<BigEndian>()?;
        let default_sample_duration = reader.read_u32::<BigEndian>()?;
        let default_sample_size = reader.read_u32::<BigEndian>()?;
        let default_sample_flags = reader.read_u32::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(TrexBox {
            version,
            flags,
            track_id,
            default_sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
        })
    }
}
