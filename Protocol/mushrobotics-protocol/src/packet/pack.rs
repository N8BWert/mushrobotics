//!
//! Trait used to pack data into a [u8] buffer
//! 

/// A Frame is a 32 Byte window of data
pub type Frame = [u8; 32];

/// Trait implemented by data in packets to ensure it can be converted
/// into a u8 slice
///
/// In general, I would suggest using big endian, but I guess it doesn't really matter
pub trait Pack<const SIZE: usize> {
    /// Convert this data into a [u8] slice
    fn pack(self) -> [u8; SIZE];
}

/// Trait implemented by data in packets to ensure it can be converted
/// from a slice of frames
pub trait Unpack {
    /// Unpack Some Data From a slice of frames
    fn unpack(data: &[Frame]) -> Self;
}

/// Error Packing some data type
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PackError {
    // There is enough space
    NotEnoughSpace
}