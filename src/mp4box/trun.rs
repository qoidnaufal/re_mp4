use byteorder::{BigEndian, ReadBytesExt};
use serde::Serialize;
use std::io::{Read, Seek};
use std::mem::size_of;

use crate::mp4box::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct TrunBox {
    pub version: u8,
    pub flags: u32,
    pub sample_count: u32,
    pub data_offset: Option<i32>,
    pub first_sample_flags: Option<u32>,

    #[serde(skip_serializing)]
    pub sample_durations: Vec<u32>,
    #[serde(skip_serializing)]
    pub sample_sizes: Vec<u32>,
    #[serde(skip_serializing)]
    pub sample_flags: Vec<u32>,
    #[serde(skip_serializing)]
    pub sample_cts: Vec<u32>,
}

impl TrunBox {
    pub const FLAG_DATA_OFFSET: u32 = 0x01;
    pub const FLAG_FIRST_SAMPLE_FLAGS: u32 = 0x04;
    pub const FLAG_SAMPLE_DURATION: u32 = 0x100;
    pub const FLAG_SAMPLE_SIZE: u32 = 0x200;
    pub const FLAG_SAMPLE_FLAGS: u32 = 0x400;
    pub const FLAG_SAMPLE_CTS: u32 = 0x800;

    pub fn get_type(&self) -> BoxType {
        BoxType::TrunBox
    }

    pub fn get_size(&self) -> u64 {
        let mut sum = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if TrunBox::FLAG_DATA_OFFSET & self.flags > 0 {
            sum += 4;
        }
        if TrunBox::FLAG_FIRST_SAMPLE_FLAGS & self.flags > 0 {
            sum += 4;
        }
        if TrunBox::FLAG_SAMPLE_DURATION & self.flags > 0 {
            sum += 4 * self.sample_count as u64;
        }
        if TrunBox::FLAG_SAMPLE_SIZE & self.flags > 0 {
            sum += 4 * self.sample_count as u64;
        }
        if TrunBox::FLAG_SAMPLE_FLAGS & self.flags > 0 {
            sum += 4 * self.sample_count as u64;
        }
        if TrunBox::FLAG_SAMPLE_CTS & self.flags > 0 {
            sum += 4 * self.sample_count as u64;
        }
        sum
    }
}

impl Mp4Box for TrunBox {
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
        let s = format!("sample_size={}", self.sample_count);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadBox<&mut R> for TrunBox {
    fn read_box(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_box_header_ext(reader)?;

        let header_size = HEADER_SIZE + HEADER_EXT_SIZE;
        let other_size = size_of::<u32>() // sample_count
            + if TrunBox::FLAG_DATA_OFFSET & flags > 0 { size_of::<i32>() } else { 0 } // data_offset
            + if TrunBox::FLAG_FIRST_SAMPLE_FLAGS & flags > 0 { size_of::<u32>() } else { 0 }; // first_sample_flags
        let sample_size = if TrunBox::FLAG_SAMPLE_DURATION & flags > 0 { size_of::<u32>() } else { 0 } // sample_duration
            + if TrunBox::FLAG_SAMPLE_SIZE & flags > 0 { size_of::<u32>() } else { 0 } // sample_size
            + if TrunBox::FLAG_SAMPLE_FLAGS & flags > 0 { size_of::<u32>() } else { 0 } // sample_flags
            + if TrunBox::FLAG_SAMPLE_CTS & flags > 0 { size_of::<u32>() } else { 0 }; // sample_composition_time_offset

        let sample_count = reader.read_u32::<BigEndian>()?;

        let data_offset = if TrunBox::FLAG_DATA_OFFSET & flags > 0 {
            Some(reader.read_i32::<BigEndian>()?)
        } else {
            None
        };

        let first_sample_flags = if TrunBox::FLAG_FIRST_SAMPLE_FLAGS & flags > 0 {
            Some(reader.read_u32::<BigEndian>()?)
        } else {
            None
        };

        let mut sample_durations = Vec::new();
        let mut sample_sizes = Vec::new();
        let mut sample_flags = Vec::new();
        let mut sample_cts = Vec::new();
        if u64::from(sample_count) * sample_size as u64
            > size
                .saturating_sub(header_size)
                .saturating_sub(other_size as u64)
        {
            return Err(Error::InvalidData(
                "trun sample_count indicates more values than could fit in the box",
            ));
        }
        if TrunBox::FLAG_SAMPLE_DURATION & flags > 0 {
            sample_durations.reserve(sample_count as usize);
        }
        if TrunBox::FLAG_SAMPLE_SIZE & flags > 0 {
            sample_sizes.reserve(sample_count as usize);
        }
        if TrunBox::FLAG_SAMPLE_FLAGS & flags > 0 {
            sample_flags.reserve(sample_count as usize);
        }
        if TrunBox::FLAG_SAMPLE_CTS & flags > 0 {
            sample_cts.reserve(sample_count as usize);
        }

        for _ in 0..sample_count {
            if TrunBox::FLAG_SAMPLE_DURATION & flags > 0 {
                let duration = reader.read_u32::<BigEndian>()?;
                sample_durations.push(duration);
            }

            if TrunBox::FLAG_SAMPLE_SIZE & flags > 0 {
                let sample_size = reader.read_u32::<BigEndian>()?;
                sample_sizes.push(sample_size);
            }

            if TrunBox::FLAG_SAMPLE_FLAGS & flags > 0 {
                let sample_flag = reader.read_u32::<BigEndian>()?;
                sample_flags.push(sample_flag);
            }

            if TrunBox::FLAG_SAMPLE_CTS & flags > 0 {
                let cts = reader.read_u32::<BigEndian>()?;
                sample_cts.push(cts);
            }
        }

        skip_bytes_to(reader, start + size)?;

        Ok(TrunBox {
            version,
            flags,
            sample_count,
            data_offset,
            first_sample_flags,
            sample_durations,
            sample_sizes,
            sample_flags,
            sample_cts,
        })
    }
}
