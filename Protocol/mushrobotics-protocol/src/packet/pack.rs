//!
//! Trait used to pack data into a [u8] buffer
//! 

/// Trait implemented by data in packets to ensure it can be converted
/// into a u8 slice
///
/// In general, I would suggest using big endian, but I guess it doesn't really matter
pub trait Pack<const SIZE: usize> {
    /// Convert this data into a [u8] slice
    fn pack(self) -> [u8; SIZE];
}

/// Error Packing some data type
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PackError {
    // There is enough space
    NotEnoughSpace
}