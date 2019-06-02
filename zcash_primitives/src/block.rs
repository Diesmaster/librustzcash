use sha2::{Digest, Sha256};
use std::fmt;
use std::io::{self, Read, Write};
use std::ops::Deref;

use serialize::Vector;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockHash(pub [u8; 32]);

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &b in self.0.iter().rev() {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl BlockHash {
    /// Constructs a [`BlockHash`] from the given slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the slice is not exactly 32 bytes.
    pub fn from_slice(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), 32);
        let mut hash = [0; 32];
        hash.copy_from_slice(&bytes);
        BlockHash(hash)
    }
}

/// A Zcash block header.
pub struct BlockHeader {
    hash: BlockHash,
    data: BlockHeaderData,
}

impl Deref for BlockHeader {
    type Target = BlockHeaderData;

    fn deref(&self) -> &BlockHeaderData {
        &self.data
    }
}

pub struct BlockHeaderData {
    pub version: i32,
    pub prev_block: BlockHash,
    pub merkle_root: [u8; 32],
    pub final_sapling_root: [u8; 32],
    pub time: u32,
    pub bits: u32,
    pub nonce: [u8; 32],
    pub solution: Vec<u8>,
}

impl BlockHeaderData {
    pub fn freeze(self) -> io::Result<BlockHeader> {
        BlockHeader::from_data(self)
    }
}

impl BlockHeader {
    fn from_data(data: BlockHeaderData) -> io::Result<Self> {
        let mut header = BlockHeader {
            hash: BlockHash([0; 32]),
            data,
        };
        let mut raw = vec![];
        header.write(&mut raw)?;
        header
            .hash
            .0
            .copy_from_slice(&Sha256::digest(&Sha256::digest(&raw)));
        Ok(header)
    }

    /// Returns the hash of this header.
    pub fn hash(&self) -> BlockHash {
        self.hash
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;
        let version = i32::from_le_bytes(buf);

        let mut prev_block = BlockHash([0; 32]);
        reader.read_exact(&mut prev_block.0)?;

        let mut merkle_root = [0; 32];
        reader.read_exact(&mut merkle_root)?;

        let mut final_sapling_root = [0; 32];
        reader.read_exact(&mut final_sapling_root)?;

        reader.read_exact(&mut buf)?;
        let time = u32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let bits = u32::from_le_bytes(buf);

        let mut nonce = [0; 32];
        reader.read_exact(&mut nonce)?;

        let solution = Vector::read(&mut reader, |r| {
            let mut buf = [0; 1];
            r.read_exact(&mut buf)?;
            Ok(buf[0])
        })?;

        BlockHeader::from_data(BlockHeaderData {
            version,
            prev_block,
            merkle_root,
            final_sapling_root,
            time,
            bits,
            nonce,
            solution,
        })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.prev_block.0)?;
        writer.write_all(&self.merkle_root)?;
        writer.write_all(&self.final_sapling_root)?;
        writer.write_all(&self.time.to_le_bytes())?;
        writer.write_all(&self.bits.to_le_bytes())?;
        writer.write_all(&self.nonce)?;
        Vector::write(&mut writer, &self.solution, |w, b| w.write_all(&[*b]))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BlockHeader;

    const HEADER_MAINNET_415000: [u8; 1487] = [
        0x04, 0x00, 0x00, 0x00, 0x52, 0x74, 0xb4, 0x3b, 0x9e, 0x4a, 0xd8, 0xf4, 0x3e, 0x93, 0xf7,
        0x84, 0x63, 0xd2, 0x4d, 0xcf, 0xe5, 0x31, 0xae, 0xb4, 0x71, 0x98, 0x19, 0xf4, 0xf9, 0x7f,
        0x7e, 0x03, 0x00, 0x00, 0x00, 0x00, 0x66, 0x30, 0x73, 0xbc, 0x4b, 0xfa, 0x95, 0xc9, 0xbe,
        0xc3, 0x6a, 0xad, 0x72, 0x68, 0xa5, 0x73, 0x04, 0x97, 0x97, 0xbd, 0xfc, 0x5a, 0xa4, 0xc7,
        0x43, 0xfb, 0xe4, 0x82, 0x0a, 0xa3, 0x93, 0xce, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa8, 0xbe, 0xcc, 0x5b, 0xe1,
        0xab, 0x03, 0x1c, 0xc2, 0xfd, 0x60, 0x7c, 0x77, 0x6a, 0x7a, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x3e, 0xb2, 0x18, 0x19, 0xfd, 0x40, 0x05, 0x00, 0x94, 0x9d, 0x55, 0xde, 0x0c, 0xc6,
        0x33, 0xe0, 0xcc, 0xe4, 0x1e, 0x46, 0x49, 0xef, 0x4a, 0xa3, 0x34, 0x9f, 0x01, 0x00, 0x29,
        0x0f, 0xfe, 0x28, 0x1b, 0x94, 0x7b, 0x3b, 0x53, 0xfb, 0xd2, 0xf3, 0x5b, 0x1c, 0xe2, 0x92,
        0x64, 0x9b, 0x96, 0xac, 0x6e, 0x08, 0x83, 0xaf, 0x3a, 0x68, 0x44, 0xb9, 0x55, 0x92, 0xe7,
        0x45, 0x56, 0xda, 0x34, 0x4b, 0x47, 0x01, 0x96, 0x1c, 0xd4, 0x13, 0x0c, 0x68, 0x21, 0x9c,
        0xfa, 0x13, 0x41, 0xd5, 0xaf, 0xb5, 0x04, 0x9e, 0xb0, 0xe8, 0xbe, 0x4a, 0x2d, 0x92, 0xd6,
        0x78, 0xc4, 0x07, 0x85, 0xe3, 0x37, 0x05, 0x54, 0x8b, 0x5f, 0x3a, 0x54, 0xf0, 0xa4, 0xc3,
        0x9a, 0x2f, 0x58, 0xee, 0x78, 0x4a, 0x24, 0x16, 0x3c, 0xd8, 0x6f, 0x54, 0x81, 0x23, 0x27,
        0xdf, 0x55, 0xe1, 0xd5, 0x5c, 0xa8, 0x4b, 0x6e, 0x7b, 0x88, 0x7a, 0x7c, 0xbf, 0xb9, 0x09,
        0x1a, 0x58, 0x5b, 0xdb, 0x8e, 0xa4, 0x75, 0x93, 0x07, 0xc5, 0x6c, 0x1b, 0x3d, 0xaf, 0xc6,
        0x69, 0x24, 0x5a, 0x6f, 0x65, 0x4b, 0x6f, 0x73, 0x00, 0x52, 0x26, 0x6a, 0x01, 0xad, 0x4f,
        0x9c, 0x0b, 0x59, 0xed, 0x4e, 0x17, 0x71, 0x2b, 0x3e, 0x72, 0xdf, 0x04, 0x98, 0xaa, 0x8d,
        0xe4, 0x88, 0x8f, 0x99, 0x35, 0x31, 0xc6, 0x0a, 0xcd, 0xed, 0x1d, 0x4b, 0x66, 0xe8, 0x9d,
        0xe0, 0xb6, 0x48, 0x2c, 0xcc, 0xd4, 0xa7, 0x12, 0xf5, 0xcf, 0x9d, 0x4c, 0xa8, 0x3b, 0xe0,
        0xf9, 0x22, 0xde, 0x2c, 0x1d, 0xbb, 0x3a, 0x14, 0x07, 0x48, 0x0d, 0xbe, 0x87, 0x95, 0x99,
        0x3d, 0x8b, 0xe6, 0x40, 0x98, 0x8a, 0xbf, 0xe7, 0xa8, 0xa1, 0xb3, 0x3a, 0x12, 0x13, 0x1c,
        0x45, 0x1e, 0x1a, 0xbc, 0x0d, 0x83, 0xfb, 0x85, 0x18, 0x62, 0xc6, 0x37, 0xce, 0x72, 0x4d,
        0x5f, 0xe9, 0x7a, 0xa9, 0xa8, 0x06, 0xcf, 0x34, 0xba, 0xb5, 0x09, 0xf4, 0x55, 0x4b, 0x0c,
        0xd1, 0x0a, 0x7d, 0xdf, 0xd5, 0x82, 0x1b, 0x09, 0x1a, 0xd2, 0xc9, 0x0c, 0x1a, 0xa1, 0xd8,
        0x1e, 0xb3, 0xd7, 0x2d, 0xb4, 0x19, 0x93, 0xb6, 0x48, 0xf4, 0x1e, 0x21, 0x38, 0xff, 0x95,
        0x31, 0xa3, 0x0f, 0xf7, 0x3b, 0x22, 0x14, 0x0e, 0x4e, 0xbd, 0x7b, 0xaa, 0x33, 0x84, 0x8e,
        0x51, 0x2d, 0x99, 0x30, 0x0c, 0x5c, 0x13, 0x1c, 0x6e, 0x75, 0xf5, 0x71, 0x4a, 0x5c, 0x6d,
        0xcb, 0x17, 0x8b, 0x4a, 0x49, 0x78, 0xda, 0xc8, 0x3a, 0xd4, 0x12, 0xfb, 0xd6, 0x92, 0x01,
        0x92, 0x50, 0xc5, 0x53, 0x04, 0x9a, 0xad, 0x45, 0x79, 0x84, 0xbe, 0xdf, 0xc9, 0x6a, 0xe7,
        0x01, 0xc6, 0x59, 0xbc, 0x70, 0x07, 0xa9, 0x7d, 0x0a, 0x90, 0x02, 0xb9, 0x45, 0xbd, 0xec,
        0x45, 0xa9, 0x45, 0xef, 0x62, 0x85, 0xb2, 0xcd, 0x55, 0x3b, 0x4c, 0x09, 0xd9, 0x07, 0xc6,
        0x27, 0x86, 0x3f, 0x03, 0x99, 0xe8, 0x72, 0x5b, 0x4f, 0xf7, 0xfc, 0x59, 0x79, 0xe3, 0xcf,
        0xf2, 0x28, 0x14, 0x50, 0x84, 0x48, 0xef, 0x8b, 0x98, 0x31, 0xc2, 0x85, 0x95, 0x93, 0x33,
        0x39, 0x6a, 0xa3, 0x62, 0xa5, 0x1c, 0xf2, 0x05, 0x09, 0x7a, 0xfa, 0xbe, 0xc1, 0x5e, 0x41,
        0xfb, 0x6e, 0x30, 0xb6, 0x22, 0x37, 0x4b, 0xf5, 0x8b, 0x37, 0xef, 0x9d, 0x1b, 0x24, 0x1e,
        0xad, 0x5a, 0x68, 0x2b, 0x98, 0xb6, 0x57, 0x49, 0xa5, 0x75, 0x68, 0xe2, 0x38, 0xd5, 0x0a,
        0xfd, 0x41, 0x7e, 0x1e, 0x96, 0x0e, 0x7b, 0x5a, 0x06, 0x4f, 0xd9, 0xf6, 0x94, 0xd7, 0x83,
        0xa2, 0xcb, 0xcd, 0x58, 0x55, 0x2d, 0xed, 0xbb, 0x9e, 0x5e, 0x11, 0x23, 0x67, 0x4e, 0xf7,
        0x3a, 0x52, 0x41, 0x96, 0xcf, 0x05, 0xd3, 0xe5, 0x24, 0x66, 0x05, 0x49, 0xff, 0xe7, 0xbd,
        0x65, 0x68, 0x05, 0x71, 0x35, 0xff, 0xd5, 0xaf, 0xd9, 0x43, 0xf6, 0xda, 0x11, 0xcb, 0xb5,
        0x97, 0xe8, 0xcc, 0xec, 0xd7, 0x7e, 0xcb, 0xe9, 0x09, 0xde, 0x06, 0x31, 0xbf, 0xa2, 0x9c,
        0xd3, 0xe3, 0xd5, 0x54, 0x46, 0x71, 0xba, 0x80, 0x25, 0x61, 0x53, 0xd6, 0xe9, 0x99, 0x0b,
        0x88, 0xad, 0x8e, 0x0c, 0xf4, 0x98, 0x9b, 0xef, 0x4b, 0xe4, 0x57, 0xf9, 0xc7, 0xb0, 0xf1,
        0xaa, 0xcd, 0x6e, 0x0e, 0xf3, 0x20, 0x60, 0x5c, 0x29, 0xed, 0x0c, 0xd2, 0xeb, 0x6c, 0xfc,
        0xe2, 0x16, 0xc5, 0x2a, 0x31, 0x75, 0x80, 0x20, 0x1c, 0xad, 0x7a, 0x09, 0x43, 0xd2, 0x4b,
        0x7b, 0x06, 0xd5, 0xbf, 0x75, 0x87, 0x61, 0xdd, 0x96, 0xe1, 0x19, 0x70, 0xb5, 0xde, 0xd6,
        0x97, 0x22, 0x2b, 0x2c, 0x77, 0xe7, 0xf2, 0x56, 0xa6, 0x05, 0xac, 0x75, 0x55, 0x49, 0xc1,
        0x65, 0x1f, 0x25, 0xad, 0xfc, 0x9d, 0x53, 0xd9, 0x11, 0x7e, 0x3a, 0x0b, 0xb4, 0x09, 0xee,
        0xe4, 0xa6, 0x00, 0x12, 0x04, 0x72, 0x94, 0x9c, 0x7d, 0xda, 0x1c, 0x2e, 0xdb, 0x3c, 0x33,
        0x0c, 0x7f, 0x96, 0x17, 0x99, 0x82, 0x91, 0x64, 0x57, 0xd3, 0x31, 0xe9, 0x63, 0x09, 0xdd,
        0x24, 0xdf, 0x74, 0xee, 0xdd, 0x00, 0xe7, 0xdb, 0x49, 0x7e, 0xe1, 0x30, 0xf7, 0x7d, 0xe6,
        0x66, 0xeb, 0x55, 0x7f, 0xb3, 0x16, 0xe8, 0x7a, 0xda, 0xf1, 0x81, 0x3c, 0xe4, 0x26, 0xa4,
        0x58, 0xa6, 0xee, 0xe3, 0xa8, 0x5b, 0x2a, 0xb8, 0x8f, 0x65, 0x53, 0xaa, 0xda, 0xe8, 0xde,
        0x65, 0x2e, 0x21, 0x1a, 0x1d, 0x9f, 0x33, 0x4d, 0x59, 0x6b, 0x5e, 0xb6, 0x17, 0x34, 0x07,
        0xef, 0xcc, 0x2e, 0x81, 0x54, 0xbb, 0x9c, 0xa1, 0x21, 0x2a, 0xa9, 0xa1, 0xa1, 0x12, 0x1d,
        0x2f, 0x5a, 0x77, 0x12, 0xcf, 0x25, 0xcc, 0x81, 0x48, 0xb8, 0x05, 0x2e, 0x0d, 0x2e, 0x09,
        0xf2, 0x0e, 0x5b, 0xa2, 0xa9, 0x82, 0x77, 0xe9, 0x75, 0xb0, 0xee, 0xd9, 0xa8, 0x92, 0x06,
        0x96, 0x63, 0x37, 0x16, 0x3f, 0x21, 0x5c, 0x9d, 0x04, 0xa6, 0x59, 0x8b, 0x09, 0x58, 0xd3,
        0x33, 0xd8, 0x46, 0x77, 0x3c, 0x69, 0xe5, 0xab, 0xfd, 0x0a, 0x04, 0x27, 0xf3, 0x66, 0x06,
        0x14, 0xdd, 0x82, 0xb7, 0x9a, 0xdb, 0x85, 0x1a, 0x0d, 0x58, 0xb6, 0x2d, 0xf5, 0xf0, 0xb3,
        0xac, 0x83, 0x6e, 0x6e, 0x25, 0xf3, 0xa5, 0x1f, 0x49, 0xa9, 0x9a, 0xde, 0x57, 0x79, 0x6f,
        0xe9, 0xfc, 0xc2, 0x6f, 0x0a, 0x1f, 0x94, 0xff, 0x08, 0x19, 0xfe, 0x52, 0xb7, 0x50, 0x87,
        0xed, 0xbe, 0xd3, 0xa8, 0x16, 0x26, 0xeb, 0x54, 0x16, 0xc6, 0x65, 0x57, 0xf1, 0x1c, 0x0f,
        0xce, 0xdf, 0xf2, 0x23, 0xd6, 0xaa, 0x8c, 0xd5, 0xc3, 0x53, 0x86, 0xe5, 0xb4, 0xb9, 0x5a,
        0x0f, 0x03, 0x92, 0xca, 0x30, 0x1a, 0x38, 0xb3, 0x68, 0x7d, 0x09, 0x44, 0x93, 0xb9, 0xe9,
        0xd2, 0x64, 0xd0, 0x7a, 0x19, 0x0c, 0xe5, 0x7d, 0x11, 0x68, 0x04, 0x38, 0x2a, 0x3f, 0xab,
        0xe1, 0x5a, 0xf4, 0xdf, 0x4f, 0xa0, 0x43, 0xf0, 0x28, 0x7a, 0xa1, 0xed, 0x55, 0x68, 0xd9,
        0xef, 0x5d, 0x12, 0x51, 0x0d, 0x01, 0x0c, 0xcd, 0xab, 0x4e, 0xb6, 0x16, 0xf6, 0xdf, 0x13,
        0xbb, 0x31, 0x26, 0xef, 0x43, 0xd9, 0xd6, 0x57, 0x35, 0xe4, 0xe4, 0xc0, 0x4b, 0x57, 0x63,
        0x48, 0xd0, 0x40, 0xb5, 0x35, 0x05, 0x5a, 0x3d, 0x5a, 0xe1, 0x91, 0xb7, 0x5f, 0x06, 0x12,
        0xf3, 0xb2, 0x40, 0x66, 0xa0, 0x52, 0x45, 0xf2, 0x7f, 0xe5, 0x7b, 0xda, 0x66, 0xbd, 0x6d,
        0xec, 0x7e, 0x4f, 0xc9, 0xcb, 0x23, 0x68, 0x02, 0x06, 0x2a, 0xdd, 0xe3, 0xcd, 0x0e, 0x31,
        0x34, 0x82, 0xc9, 0x2a, 0x0c, 0x72, 0x11, 0x02, 0xb1, 0xf3, 0x8b, 0x01, 0x5a, 0xb8, 0xd0,
        0x15, 0x59, 0xcb, 0xcb, 0x40, 0xf6, 0x74, 0xe9, 0xef, 0xad, 0x5e, 0xe9, 0xc2, 0xfe, 0x13,
        0x3f, 0xaa, 0x55, 0xca, 0x1d, 0xd0, 0xff, 0x26, 0x71, 0x0f, 0x9d, 0xa8, 0x19, 0xcc, 0x14,
        0x59, 0xcb, 0x7e, 0xd2, 0x60, 0xda, 0xd3, 0xdb, 0x05, 0x96, 0x25, 0x8d, 0x47, 0xc7, 0x4c,
        0x32, 0xa8, 0xb8, 0x52, 0xb6, 0x71, 0xc5, 0xa0, 0xca, 0xa2, 0x00, 0x16, 0x03, 0xd9, 0x0c,
        0x91, 0xa7, 0xdf, 0x2e, 0x2d, 0x4e, 0xe9, 0xae, 0x9b, 0xf1, 0xa6, 0xb1, 0xec, 0x88, 0x15,
        0x1c, 0x62, 0x36, 0x0d, 0x03, 0x02, 0x4d, 0x2e, 0x2d, 0x01, 0x14, 0x08, 0x4f, 0x6b, 0x88,
        0xc5, 0xbb, 0xa2, 0x4a, 0xa7, 0xce, 0xcf, 0xac, 0x16, 0xe9, 0x1e, 0x0b, 0xaf, 0x3d, 0x86,
        0x53, 0xe2, 0x18, 0x09, 0x3e, 0x81, 0xd2, 0xa6, 0x3c, 0x32, 0xef, 0xf1, 0xd9, 0x03, 0x0f,
        0x9e, 0x14, 0x14, 0xec, 0xe4, 0x20, 0xda, 0xa2, 0x4e, 0x0d, 0xd5, 0xb8, 0x45, 0xb3, 0x27,
        0x4b, 0xb8, 0x39, 0xca, 0x1c, 0x53, 0xbc, 0xc0, 0x19, 0x42, 0x42, 0xd7, 0x4b, 0x26, 0x31,
        0xb9, 0x49, 0x5a, 0x65, 0x4f, 0xbb, 0xdc, 0xbf, 0xad, 0x77, 0x9f, 0x73, 0x22, 0xb6, 0x07,
        0x36, 0x24, 0x98, 0x80, 0x60, 0x48, 0x21, 0xd9, 0x69, 0x24, 0xe3, 0xfa, 0x39, 0x7f, 0x35,
        0x4a, 0x5e, 0xcc, 0xa3, 0x4f, 0x61, 0x4d, 0xa5, 0x45, 0x6f, 0x9b, 0x36, 0x33, 0x8c, 0x37,
        0xd8, 0xf6, 0xfb, 0xf6, 0x26, 0xbe, 0x98, 0x34, 0x77, 0x76, 0x60, 0x22, 0x87, 0x27, 0x46,
        0xda, 0x10, 0xa1, 0x77, 0x1c, 0xeb, 0x02, 0xdd, 0x8a, 0xac, 0x01, 0xba, 0x18, 0x6b, 0xf1,
        0x48, 0x86, 0x30, 0x47, 0x9e, 0x12, 0x84, 0xda, 0x01, 0x90, 0xfc, 0xe8, 0xb5, 0x9a, 0xc6,
        0xb0, 0xfd, 0x41, 0x6b, 0xee, 0x56, 0xb7, 0x2f, 0x0a, 0x58, 0x45, 0x15, 0x35, 0x57, 0xff,
        0x0f, 0x49, 0x50, 0xa0, 0xdc, 0x5b, 0xe6, 0x5c, 0xe9, 0x42, 0xd2, 0x2e, 0x18, 0x53, 0x4c,
        0x4e, 0x0e, 0xfa, 0xbb, 0x2d, 0x15, 0x25, 0xdc, 0x48, 0x58, 0xb9, 0xb0, 0xf7, 0x7d, 0x47,
        0x4a, 0x12, 0x5e, 0xbc, 0x25, 0x0e, 0x08, 0xfe, 0xdb, 0xfa, 0xa6, 0x6f, 0x45, 0x3d, 0x90,
        0x93, 0x2c, 0xab, 0x3f, 0xf4, 0x52, 0x21, 0x90, 0x99, 0x68, 0xe5, 0x1e, 0x6b, 0xc2, 0x54,
        0xd5, 0x09, 0xad, 0xeb, 0x75, 0xcb, 0xa7, 0x6d, 0x48, 0xfe, 0x02, 0x4e, 0x3e, 0x66, 0xd8,
        0xdf, 0x5e,
    ];

    #[test]
    fn header_read_write() {
        let header = BlockHeader::read(&HEADER_MAINNET_415000[..]).unwrap();
        assert_eq!(
            format!("{}", header.hash()),
            "0000000001ab37793ce771262b2ffa082519aa3fe891250a1adb43baaf856168"
        );
        let mut encoded = Vec::with_capacity(HEADER_MAINNET_415000.len());
        header.write(&mut encoded).unwrap();
        assert_eq!(&HEADER_MAINNET_415000[..], &encoded[..]);
    }
}
