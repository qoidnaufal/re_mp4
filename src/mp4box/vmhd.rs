use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::{Read, Seek};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct VmhdBox {
    pub version: u8,
    pub flags: u32,
    pub graphics_mode: u16,
    pub op_color: RgbColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct RgbColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl VmhdBox {
    pub fn get_type(&self) -> BoxType {
        BoxType::VmhdBox
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8
    }
}

impl Mp4Box for VmhdBox {
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
            "graphics_mode={} op_color={}{}{}",
            self.graphics_mode, self.op_color.red, self.op_color.green, self.op_color.blue
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for VmhdBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let graphics_mode = reader.read_u16::<BigEndian>()?;
        let op_color = RgbColor {
            red: reader.read_u16::<BigEndian>()?,
            green: reader.read_u16::<BigEndian>()?,
            blue: reader.read_u16::<BigEndian>()?,
        };

        skip_bytes_to(reader, start + size)?;

        Ok(VmhdBox {
            version,
            flags,
            graphics_mode,
            op_color,
        })
    }
}
