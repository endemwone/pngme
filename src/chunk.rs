use crc::{Crc, CRC_32_ISO_HDLC};
use std::convert::TryFrom;
use std::fmt;

use crate::chunk_type::ChunkType;
use crate::{Error, Result};

/// A validated PNG chunk. See the PNG Spec for more details
/// http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html
#[derive(Debug, Clone)]
pub struct Chunk {
    chunk_type: ChunkType, // The type of this chunk
    data: Vec<u8>,         // The data contained in this chunk
}

impl Chunk {
    pub const DATA_LENGTH_BYTES: usize = 4;
    pub const CHUNK_TYPE_BYTES: usize = 4;
    pub const CRC_BYTES: usize = 4;

    /// Total size of the of the metadata making up a chunk
    pub const METADATA_BYTES: usize =
        Chunk::DATA_LENGTH_BYTES + Chunk::CHUNK_TYPE_BYTES + Chunk::CRC_BYTES;

    /// Creates a new `Chunk` with the given `ChunkType` and data.
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        Self { chunk_type, data }
    }

    /// The length of the data portion of this chunk.
    pub fn length(&self) -> u32 {
        self.data.len() as u32
    }

    /// The `ChunkType` of this chunk
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    /// The raw data contained in this chunk in bytes
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The CRC of this chunk
    pub fn crc(&self) -> u32 {
        const CRC_PNG: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

        CRC_PNG.checksum(
            &self
                .chunk_type
                .bytes()
                .iter()
                .chain(self.data.iter())
                .copied()
                .collect::<Vec<u8>>(),
        )
    }

    /// Returns the data stored in this chunk as a `String`. This function will return an error
    /// if the stored data is not valid UTF-8.
    pub fn data_as_string(&self) -> Result<String> {
        let s = String::from_utf8(self.data.clone())?;
        Ok(s)
    }

    /// Returns this chunk as a byte sequences described by the PNG spec.
    /// The following data is included in this byte sequence in order:
    /// 1. Length of the data *(4 bytes)*
    /// 2. Chunk type *(4 bytes)*
    /// 3. The data itself *(`length` bytes)*
    /// 4. The CRC of the chunk type and data *(4 bytes)*
    pub fn as_bytes(&self) -> Vec<u8> {
        let length = self.length();
        length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.data.iter())
            .chain(self.crc().to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Chunk::METADATA_BYTES {
            return Err(Box::new(ChunkError::InputTooShort));
        }

        // Read the length of the data
        let (data_length_bytes, bytes) = bytes.split_at(Chunk::DATA_LENGTH_BYTES);
        let data_length = u32::from_be_bytes(data_length_bytes.try_into()?);

        // Read the chunk type
        let (chunk_type_bytes, bytes) = bytes.split_at(Chunk::CHUNK_TYPE_BYTES);
        let chunk_type_bytes: [u8; 4] = chunk_type_bytes.try_into()?;
        let chunk_type = ChunkType::try_from(chunk_type_bytes)?;

        if !chunk_type.is_valid() {
            return Err(Box::new(ChunkError::InvalidChunkType));
        }

        // Read the data
        let (data_bytes, bytes) = bytes.split_at(data_length as usize);
        let data = data_bytes.to_vec();

        // Read the CRC
        let (crc_bytes, _) = bytes.split_at(Chunk::CRC_BYTES);
        let crc = u32::from_be_bytes(crc_bytes.try_into()?);

        let chunk = Chunk::new(chunk_type, data);

        if chunk.crc() != crc {
            return Err(Box::new(ChunkError::InvalidCrc(chunk.crc(), crc)));
        }

        Ok(chunk)
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chunk {{",)?;
        writeln!(f, "  Length: {}", self.length())?;
        writeln!(f, "  Type: {}", self.chunk_type())?;
        writeln!(f, "  Data: {} bytes", self.data().len())?;
        writeln!(f, "  Crc: {}", self.crc())?;
        writeln!(f, "}}",)?;
        Ok(())
    }
}

/// Errors that can occur when constructing a `Chunk`
#[derive(Debug)]
pub enum ChunkError {
    InputTooShort,
    InvalidCrc(u32, u32),
    InvalidChunkType,
}

impl std::error::Error for ChunkError {}

impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ChunkError::InputTooShort => {
                write!(f, "At least 12 bytes must be supplied to construct a chunk")
            }
            ChunkError::InvalidCrc(expected, actual) => write!(
                f,
                "Invalid CRC when constructing chunk. Expected {} but found {}",
                expected, actual
            ),
            ChunkError::InvalidChunkType => write!(f, "Invalid chunk type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
