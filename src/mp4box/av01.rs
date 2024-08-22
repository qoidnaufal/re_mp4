use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek};

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Av01Box {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,

    #[serde(with = "value_u32")]
    pub horizresolution: FixedPointU16,

    #[serde(with = "value_u32")]
    pub vertresolution: FixedPointU16,
    pub frame_count: u16,
    pub depth: u16,
    pub av1c: Av1CBox,
}

impl Av01Box {
    pub fn get_type(&self) -> BoxType {
        BoxType::Av01Box
    }

    pub fn get_size(&self) -> u64 {
        HEADER_SIZE + 8 + 70 + self.av1c.box_size()
    }
}

impl Mp4Box for Av01Box {
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
        Ok(("todo").into())
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Av01Box {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;

        reader.read_u32::<BigEndian>()?; // pre-defined, reserved
        reader.read_u64::<BigEndian>()?; // pre-defined
        reader.read_u32::<BigEndian>()?; // pre-defined
        let width = reader.read_u16::<BigEndian>()?;
        let height = reader.read_u16::<BigEndian>()?;
        let horizresolution = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
        let vertresolution = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
        reader.read_u32::<BigEndian>()?; // reserved
        let frame_count = reader.read_u16::<BigEndian>()?;
        skip_bytes(reader, 32)?; // compressorname
        let depth = reader.read_u16::<BigEndian>()?;
        reader.read_i16::<BigEndian>()?; // pre-defined

        let header = BoxHeader::read(reader)?;
        let BoxHeader { name, size: s } = header;
        if s > size {
            return Err(Error::InvalidData(
                "hev1 box contains a box with a larger size than it",
            ));
        }
        if name == BoxType::Av1CBox {
            let av1c = Av1CBox::read_box(reader, s)?;

            skip_bytes_to(reader, start + size)?;

            Ok(Av01Box {
                data_reference_index,
                width,
                height,
                horizresolution,
                vertresolution,
                frame_count,
                depth,
                av1c,
            })
        } else {
            Err(Error::InvalidData("av1c not found"))
        }
    }
}

impl<W: Write> WriteBox<&mut W> for Av01Box {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        writer.write_u32::<BigEndian>(0)?; // pre-defined, reserved
        writer.write_u64::<BigEndian>(0)?; // pre-defined
        writer.write_u32::<BigEndian>(0)?; // pre-defined
        writer.write_u16::<BigEndian>(self.width)?;
        writer.write_u16::<BigEndian>(self.height)?;
        writer.write_u32::<BigEndian>(self.horizresolution.raw_value())?;
        writer.write_u32::<BigEndian>(self.vertresolution.raw_value())?;
        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.frame_count)?;
        // skip compressorname
        write_zeros(writer, 32)?;
        writer.write_u16::<BigEndian>(self.depth)?;
        writer.write_i16::<BigEndian>(-1)?; // pre-defined

        self.av1c.write_box(writer)?;

        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Av1CBox {
    pub profile: u8,
    pub level: u8,
    pub tier: u8,
    pub bit_depth: u8,
    pub monochrome: bool,
    pub chroma_subsampling_x: u8,
    pub chroma_subsampling_y: u8,
    pub chroma_sample_position: u8,
    pub initial_presentation_delay_present: bool,
    pub initial_presentation_delay_minus_one: u8,
    pub config_obus: Vec<u8>, // Holds the variable-length configOBUs
}

impl Mp4Box for Av1CBox {
    fn box_type(&self) -> BoxType {
        BoxType::Av1CBox
    }

    fn box_size(&self) -> u64 {
        4 + self.config_obus.len() as u64
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(("todo").into())
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for Av1CBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let marker_byte = reader.read_u8()?;
        if marker_byte & 0x80 != 0x80 {
            return Err(Error::InvalidData("missing av1C marker bit"));
        }
        if marker_byte & 0x7f != 0x01 {
            return Err(Error::InvalidData("missing av1C marker bit"));
        }
        let profile_byte = reader.read_u8()?;
        let profile = (profile_byte & 0xe0) >> 5;
        let level = profile_byte & 0x1f;
        let flags_byte = reader.read_u8()?;
        let tier = (flags_byte & 0x80) >> 7;
        let bit_depth = match flags_byte & 0x60 {
            0x60 => 12,
            0x40 => 10,
            _ => 8,
        };
        let monochrome = flags_byte & 0x10 == 0x10;
        let chroma_subsampling_x = (flags_byte & 0x08) >> 3;
        let chroma_subsampling_y = (flags_byte & 0x04) >> 2;
        let chroma_sample_position = flags_byte & 0x03;
        let delay_byte = reader.read_u8()?;
        let initial_presentation_delay_present = (delay_byte & 0x10) == 0x10;
        let initial_presentation_delay_minus_one = if initial_presentation_delay_present {
            delay_byte & 0x0f
        } else {
            0
        };

        // av1c box has 4 fixed byte-sized fields
        // config obus are stored as bytes directly after the fixed fields
        // the header tells us how many bytes the box is in total
        let config_obus_size = size
            .checked_sub(HEADER_SIZE + 4) // header bytes + fixed field bytes
            .ok_or(Error::InvalidData("invalid box size"))?;

        let mut config_obus = vec![0u8; config_obus_size as usize];
        reader.read_exact(&mut config_obus)?;

        Ok(Av1CBox {
            profile,
            level,
            tier,
            bit_depth,
            monochrome,
            chroma_subsampling_x,
            chroma_subsampling_y,
            chroma_sample_position,
            initial_presentation_delay_present,
            initial_presentation_delay_minus_one,
            config_obus,
        })
    }
}

impl<W: Write> WriteBox<&mut W> for Av1CBox {
    fn write_box(&self, writer: &mut W) -> Result<u64> {
        let size = self.box_size();
        BoxHeader::new(self.box_type(), size).write(writer)?;

        let marker_byte = 0x80 | 0x01;
        writer.write_u8(marker_byte)?;

        let profile_byte = (self.profile << 5) | (self.level & 0x1f);
        writer.write_u8(profile_byte)?;

        let bit_depth_flag = match self.bit_depth {
            12 => 0x60,
            10 => 0x40,
            _ => 0x00, // Assuming 8-bit depth
        };
        let flags_byte = bit_depth_flag
            | ((self.tier & 0x01) << 7)
            | ((self.monochrome as u8) << 4)
            | ((self.chroma_subsampling_x & 0x01) << 3)
            | ((self.chroma_subsampling_y & 0x01) << 2)
            | (self.chroma_sample_position & 0x03);
        writer.write_u8(flags_byte)?;

        let delay_byte = if self.initial_presentation_delay_present {
            0x10 | (self.initial_presentation_delay_minus_one & 0x0f)
        } else {
            0x00
        };
        writer.write_u8(delay_byte)?;

        writer.write_all(&self.config_obus)?;

        Ok(size)
    }
}