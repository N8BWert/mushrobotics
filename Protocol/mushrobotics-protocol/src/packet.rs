//!
//! The Implementation of a Packet for the Mushrobotics Protocol
//! 

use alloc::boxed::Box;
use alloc::vec::Vec;

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

/// Error Packing some data type
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PackError {
    // There is enough space
    NotEnoughSpace
}

/// Trait implemented by data in packets to ensure it can be converted from
/// frames back to the original data
pub trait Unpack {
    /// Decipher the Address and Data from a slice of Frames
    fn unpack(data: &[Frame]) -> (Address, Self);
}

/// Local address options (i.e. the local address is either
/// going to the parent or the child)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LocalAddress {
    ToParent = 0xA0,
    ToChild = 0x90,
}

/// Address field of a packet.  Either the packet is going to a
/// specific node in the address, or it is going between parent and child.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Address {
    // from and to are of the format [a, b, c, d, ...] to make things easier
    Network{from: Box<[u8]>, to: Box<[u8]>},
    Local(LocalAddress),
}

impl Address {
    /// Convert the address into the first packet (i.e.) base packet that can be sent.
    ///
    /// Returns: (Packet Beginning, Address Length)
    fn to_first_packet(self) -> (Frame, usize) {
        let mut packet = [0u8; 32];

        match self {
            Address::Local(destination) => {
                packet[0] = destination as u8;

                (packet, 1)
            },
            Address::Network { from, to } => {
                for i in 0..(from.len() / 2) {
                    packet[i] = (from[2*i] << 4) | from[(2*i)+1];
                }

                if from.len() % 2 == 1 {
                    packet[from.len() / 2] = from[from.len()-1] << 4;

                    for i in 0..(to.len() / 2) {
                        packet[(from.len()/2)+1+i] = (to[2*i] << 4) | to[(2*i)+1];
                    }

                    if to.len() % 2 == 1 {
                        packet[(from.len()/2)+(to.len()/2)+1] = to[to.len()-1] << 4;
                    }
                } else {
                    for i in 0..(to.len() / 2) {
                        packet[(from.len()/2)+i] |= to[2*i];
                        packet[(from.len()/2)+i+1] = to[(2*i)+1] << 4;
                    }

                    if to.len() % 2 == 1 {
                        packet[(from.len()/2)+(to.len()/2)] |= to[to.len()-1];
                    } else {
                        return (packet, (from.len() / 2) + (to.len() / 2) + 1);
                    }
                }

            (packet, (from.len() / 2) + (to.len() / 2) + 2)
            }
        }
    }
}

impl From<&Frame> for Address {
    fn from(value: &Frame) -> Self {
        match value[0] {
            0xA0 => Address::Local(LocalAddress::ToParent),
            0x90 => Address::Local(LocalAddress::ToChild),
            _ => {
                let mut from_length = 0;
                for i in 0..32 {
                    // Check High Nibble
                    if (value[i] & 0xF0) == 0 {
                        from_length = 2 * i;
                        break;
                    }

                    // Check Low Nibble
                    if (value[i] & 0x0F) == 0 {
                        from_length = 2 * i + 1;
                        break;
                    }
                }

                let mut to_length = 0;
                for i in (from_length+1)..64 {
                    if i % 2 == 0 {
                        if (value[i/2] & 0xF0) == 0 {
                            to_length = i - (from_length + 1);
                            break;
                        }
                    } else {
                        if (value[i/2] & 0x0F) == 0 {
                            to_length = i - (from_length + 1);
                            break;
                        }
                    }
                }

                let mut from = Vec::with_capacity(from_length);
                for i in 0..from_length {
                    if i % 2 == 0 {
                        from.push((value[i/2] & 0xF0) >> 4);
                    } else {
                        from.push(value[i/2] & 0x0F);
                    }
                }

                let mut to = Vec::with_capacity(to_length);
                for i in (from_length+1)..(from_length+1+to_length) {
                    if i % 2 == 0 {
                        to.push((value[i/2] % 0xF0) >> 4);
                    } else {
                        to.push(value[i/2] & 0x0F);
                    }
                }

                Address::Network { from: from.into(), to: to.into() }
            }
        }
    }
}

/// A packet to be sent over the mushrobotics network.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Packet<Data: Pack<SIZE>, const SIZE: usize> {
    first_packet: Frame,
    prelude_length: usize,
    pub data: Data,
}

impl<Data: Pack<SIZE>, const SIZE: usize> Packet<Data, SIZE> {
    /// Create a packet with given data addressed to a node's child
    pub fn to_child(data: Data) -> Result<Self, PackError> {
        let (mut first_packet, prelude_length) = Address::Local(LocalAddress::ToChild).to_first_packet();

        let total_size = (prelude_length + 2 + SIZE) / 32;
        if total_size > (u16::MAX as usize) {
            return Err(PackError::NotEnoughSpace);
        }
        let bytes = total_size.to_le_bytes();
        first_packet[prelude_length] = bytes[1];
        first_packet[prelude_length+1] = bytes[0];

        Ok(Self {
            first_packet,
            prelude_length: prelude_length + 2,
            data,
        })
    }

    /// Create a packet with given data addresses to a node's parent
    pub fn to_parent(data: Data) -> Result<Self, PackError> {
        let (mut first_packet, prelude_length) = Address::Local(LocalAddress::ToParent).to_first_packet();

        let total_size = (prelude_length + 2 + SIZE) / 32;
        if total_size > (u16::MAX as usize) {
            return Err(PackError::NotEnoughSpace);
        }
        let bytes = total_size.to_le_bytes();
        first_packet[prelude_length] = bytes[1];
        first_packet[prelude_length+1] = bytes[0];

        Ok(Self {
            first_packet,
            prelude_length: prelude_length + 2,
            data,
        })
    }

    /// Create a packet with given network address
    ///
    /// In this case, addresses are of the format [a, b, c, d, ...] to make this easier
    /// to use
    pub fn to_address(from: &[u8], to: &[u8], data: Data) -> Result<Self, PackError> {
        let (mut first_packet, prelude_length) = Address::Network { from: from.into(), to: to.into() }.to_first_packet();

        let total_size = (prelude_length + 2 + SIZE) / 32;
        if total_size > (u16::MAX as usize) {
            return Err(PackError::NotEnoughSpace);
        }
        let bytes = total_size.to_le_bytes();
        first_packet[prelude_length] = bytes[1];
        first_packet[prelude_length+1] = bytes[0];

        Ok(Self {
            first_packet,
            prelude_length: prelude_length + 2,
            data,
        })
    }

    pub fn pack_payload(mut self) -> Result<Box<[Frame]>, PackError> {
        let total_size = self.prelude_length + SIZE;
        if total_size > (u16::MAX as usize) {
            return Err(PackError::NotEnoughSpace);
        }

        let data = self.data.pack();

        if total_size <= 32 {
            self.first_packet[self.prelude_length..(self.prelude_length+SIZE)].copy_from_slice(&data[..]);
            return Ok(Box::new([self.first_packet]));
        }

        self.first_packet[self.prelude_length..].copy_from_slice(&data[..(32-self.prelude_length)]);
        
        let total_frames = total_size.div_ceil(32);
        let mut frames = Vec::with_capacity(total_frames);
        frames.push(self.first_packet);

        let data = &data[(32-self.prelude_length)..];
        for i in 0..(total_frames-2) {
            let mut buffer = [0u8; 32];
            buffer.copy_from_slice(&data[(32*i)..(32*(i+1))]);
            frames.push(buffer);
        }

        let mut buffer = [0u8; 32];
        buffer[..(total_size%32)].copy_from_slice(&data[(32*(total_frames-2))..]);
        frames.push(buffer);

        Ok(frames.into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Pack<2> for u16 {
        fn pack(self) -> [u8; 2] {
            [((self & 0xFF00) >> 8) as u8, (self & 0xFF) as u8]
        }
    }

    impl Pack<128> for [u32; 32] {
        fn pack(self) -> [u8; 128] {
            let mut buffer = [0u8; 128];

            for (i, value) in self.iter().enumerate() {
                buffer[(i*4)+0] = ((value & (0xFF << 24)) >> 24) as u8;
                buffer[(i*4)+1] = ((value & (0xFF << 16)) >> 16) as u8;
                buffer[(i*4)+2] = ((value & (0xFF << 8)) >> 8) as u8;
                buffer[(i*4)+3] = (value & 0xFF) as u8;
            }

            buffer
        }
    }

    #[test]
    fn test_to_packet_parent_first_packet() {
        let address = Address::Local(LocalAddress::ToParent);
        let (first_packet, size) = address.to_first_packet();
        let mut expected_packet = [0u8; 32];
        expected_packet[0] = 0xA0;
        let expected_size = 1;

        assert_eq!(expected_packet, first_packet);
        assert_eq!(expected_size, size);
    }

    #[test]
    fn test_to_packet_child_first_packet() {
        let address = Address::Local(LocalAddress::ToChild);
        let (first_packet, size) = address.to_first_packet();
        let mut expected_packet = [0u8; 32];
        expected_packet[0] = 0x90;
        let expected_size = 1;

        assert_eq!(expected_packet, first_packet);
        assert_eq!(expected_size, size);
    }

    #[test]
    fn test_to_packet_network_both_even() {
        let from = [1, 2, 3, 4];
        let to = [4, 3, 2, 1];
        let address = Address::Network{ from: Box::new(from), to: Box::new(to) };
        let (first_packet, size) = address.to_first_packet();

        let mut expected_packet = [0u8; 32];
        expected_packet[0] = 0x12;
        expected_packet[1] = 0x34;
        expected_packet[2] = 0x04;
        expected_packet[3] = 0x32;
        expected_packet[4] = 0x10;
        let expected_size = 5;

        assert_eq!(expected_packet, first_packet);
        assert_eq!(expected_size, size);
    }

    #[test]
    fn test_to_packet_network_even_from_odd_to() {
        let from = [1, 2, 3, 4];
        let to = [5, 4, 3, 2, 1];
        let address = Address::Network{ from: Box::new(from), to: Box::new(to) };
        let (first_packet, size) = address.to_first_packet();

        let mut expected_packet = [0u8; 32];
        expected_packet[0] = 0x12;
        expected_packet[1] = 0x34;
        expected_packet[2] = 0x05;
        expected_packet[3] = 0x43;
        expected_packet[4] = 0x21;
        expected_packet[5] = 0x00;
        let expected_size = 6;

        assert_eq!(expected_packet, first_packet);
        assert_eq!(expected_size, size);
    }

    #[test]
    fn test_to_packet_network_odd_from_even_to() {
        let from = [1, 2, 3, 4, 5];
        let to = [4, 3, 2, 1];
        let address = Address::Network { from: Box::new(from), to: Box::new(to) };
        let (first_packet, size) = address.to_first_packet();

        let mut expected_packet = [0u8; 32];
        expected_packet[0] = 0x12;
        expected_packet[1] = 0x34;
        expected_packet[2] = 0x50;
        expected_packet[3] = 0x43;
        expected_packet[4] = 0x21;
        expected_packet[5] = 0x00;
        let expected_size = 6;

        assert_eq!(expected_packet, first_packet);
        assert_eq!(expected_size, size);
    }

    #[test]
    fn test_to_packet_network_both_odd() {
        let from = [1, 2, 3, 4, 5];
        let to = [5, 4, 3, 2, 1];
        let address = Address::Network { from: Box::new(from), to: Box::new(to) };
        let (first_packet, size) = address.to_first_packet();

        let mut expected_packet = [0u8; 32];
        expected_packet[0] = 0x12;
        expected_packet[1] = 0x34;
        expected_packet[2] = 0x50;
        expected_packet[3] = 0x54;
        expected_packet[4] = 0x32;
        expected_packet[5] = 0x10;
        let expected_size = 6;

        assert_eq!(expected_packet, first_packet);
        assert_eq!(expected_size, size);
    }

    #[test]
    fn test_to_parent_pack_u16() {
        let packet = Packet::to_parent(0x3F21u16).unwrap();
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [0u8; 32];
        expected_payload[0] = 0xA0;
        expected_payload[1] = 0x00;
        expected_payload[2] = 0x00;
        expected_payload[3] = 0x3F;
        expected_payload[4] = 0x21;

        assert_eq!(payload[0], expected_payload);
    }

    #[test]
    fn test_to_child_pack_u16() {
        let packet = Packet::to_child(0x3214u16).unwrap();
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [0u8; 32];
        expected_payload[0] = 0x90;
        expected_payload[1] = 0x00;
        expected_payload[2] = 0x00;
        expected_payload[3] = 0x32;
        expected_payload[4] = 0x14;

        assert_eq!(payload[0], expected_payload);
    }

    #[test]
    fn test_to_network_pack_u16() {
        let packet = Packet::to_address(&[1, 2, 3], &[3, 2, 1], 0x1923u16).unwrap();
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [0u8; 32];
        expected_payload[0] = 0x12;
        expected_payload[1] = 0x30;
        expected_payload[2] = 0x32;
        expected_payload[3] = 0x10;
        expected_payload[4] = 0x00;
        expected_payload[5] = 0x00;
        expected_payload[6] = 0x19;
        expected_payload[7] = 0x23;

        assert_eq!(payload[0], expected_payload);
    }

    #[test]
    fn test_to_parent_pack_large() {
        let mut data = [0u32; 32];
        for i in 0..32 {
            data[i] = i as u32;
        }
        let packet = Packet::to_parent(data).unwrap();
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [[0u8; 32]; 5];
        expected_payload[0][0] = 0xA0;
        expected_payload[0][1] = 0x00;
        expected_payload[0][2] = 0x04;
        for i in 0..32 {
            let x = (4 * i) + 6;
            expected_payload[x/32][x%32] = i as u8;
        }

        assert_eq!(*payload, expected_payload);
    }

    #[test]
    fn test_to_child_pack_large() {
        let mut data = [0u32; 32];
        for i in 0..32 {
            data[i] = i as u32;
        }
        let packet = Packet::to_child(data).unwrap();
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [[0u8; 32]; 5];
        expected_payload[0][0] = 0x90;
        expected_payload[0][1] = 0x00;
        expected_payload[0][2] = 0x04;
        for i in 0..32 {
            let x = (4 * i) + 6;
            expected_payload[x/32][x%32] = i as u8;
        }

        assert_eq!(*payload, expected_payload);
    }

    #[test]
    fn to_network_pack_large() {
        let mut data = [0u32; 32];
        for i in 0..32 {
            data[i] = i as u32;
        }
        let packet = Packet::to_address(&[1, 2, 3], &[3, 2, 1], data).unwrap();
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [[0u8; 32]; 5];
        expected_payload[0][0] = 0x12;
        expected_payload[0][1] = 0x30;
        expected_payload[0][2] = 0x32;
        expected_payload[0][3] = 0x10;
        expected_payload[0][4] = 0x00;
        expected_payload[0][5] = 0x04;
        for i in 0..32 {
            let x = (4 * i) + 9;
            expected_payload[x/32][x%32] = i as u8;
        }

        assert_eq!(*payload, expected_payload);
    }

    #[test]
    fn from_packet_to_parent() {
        let mut packet = [0u8; 32];
        packet[0] = 0xA0;

        assert_eq!(Address::Local(LocalAddress::ToParent), (&packet).into());
    }

    #[test]
    fn from_packet_to_child() {
        let mut packet = [0u8; 32];
        packet[0] = 0x90;

        assert_eq!(Address::Local(LocalAddress::ToChild), (&packet).into());
    }

    #[test]
    /// 1 -> 5
    fn from_packet_to_address_one_hop() {
        let mut packet = [0u8; 32];
        packet[0] = 0x10;
        packet[1] = 0x50;

        assert_eq!(
            Address::Network { from: Box::new([1]), to: Box::new([5]) },
            (&packet).into()
        );
    }

    #[test]
    /// 1.2.3 -> 4.3.2
    fn from_packet_to_address_multi_hop() {
        let mut packet = [0u8; 32];
        packet[0] = 0x12;
        packet[1] = 0x30;
        packet[2] = 0x43;
        packet[3] = 0x20;

        assert_eq!(
            Address::Network { from: Box::new([1, 2, 3]), to: Box::new([4, 3, 2]) },
            (&packet).into()
        );
    }

    #[test]
    /// 1.2.3. -> 4.3
    fn from_packet_to_address_odd_from_even_to() {
        let mut packet = [0u8; 32];
        packet[0] = 0x12;
        packet[1] = 0x30;
        packet[2] = 0x43;

        assert_eq!(
            Address::Network { from: Box::new([1, 2, 3]), to: Box::new([4, 3]) },
            (&packet).into(),
        );
    }

    #[test]
    /// 1.2 -> 3.2.1
    fn from_packet_to_address_even_from_odd_to() {
        let mut packet = [0u8; 32];
        packet[0] = 0x12;
        packet[1] = 0x03;
        packet[2] = 0x21;

        assert_eq!(
            Address::Network { from: Box::new([1, 2]), to: Box::new([3, 2, 1]) },
            (&packet).into()
        );
    }

    #[test]
    /// 1.2 -> 3.2
    fn from_packet_to_address_even_from_even_to() {
        let mut packet = [0u8; 32];
        packet[0] = 0x12;
        packet[1] = 0x03;
        packet[2] = 0x20;

        assert_eq!(
            Address::Network { from: Box::new([1, 2]), to: Box::new([3, 2]) },
            (&packet).into()
        );
    }
}