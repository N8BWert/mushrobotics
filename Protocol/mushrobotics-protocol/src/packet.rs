mod pack;

pub use pack::{Pack, PackError};

use alloc::boxed::Box;
use alloc::vec::Vec;

/// Local address options (i.e. the local address is either
/// going to the parent or the child)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LocalAddress {
    ToParent = 0xA0,
    ToChild = 0x90,
}

/// Address field of a packet.  Either the packet is going to a
/// specific node in the address, or it is going between parent and child.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Address<'a> {
    // from and to are of the format [a, b, c, d, ...] to make things easier
    Network{from: &'a [u8], to: &'a [u8]},
    Local(LocalAddress),
}

impl<'a> Address<'a> {
    /// Convert the address into the first packet (i.e.) base packet that can be sent.
    ///
    /// Returns: (Packet Beginning, Address Length)
    fn to_first_packet(self) -> ([u8; 32], usize) {
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

impl<'a> From<&[u8; 32]> for Address<'a> {
    fn from(value: &[u8; 32]) -> Self {
        match value[0] {
            0xA0 => Address::Local(LocalAddress::ToParent),
            0x90 => Address::Local(LocalAddress::ToParent),
            _ => {
                let mut from_length = 0;
                for i in 0..32 {
                    if (value[i] & 0xF0) == 0 {
                        break;
                    } else {
                        from_length += 1;
                    }

                    if (value[i] & 0x0F) == 0 {
                        break;
                    } else {
                        from_length += 1;
                    }
                }

                let mut to_length = 0;

                Address::Network { from: &[1, 2, 3], to: &[3, 2, 1] }
            }
        }
    }
}

/// A packet to be sent over the mushrobotics network.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Packet<Data: Pack<SIZE>, const SIZE: usize> {
    first_packet: [u8; 32],
    prelude_length: usize,
    pub data: Data,
}

impl<'a, Data: Pack<SIZE>, const SIZE: usize> Packet<Data, SIZE> {
    /// Create a packet with given data addressed to a node's child
    pub fn to_child(data: Data) -> Self {
        let (first_packet, prelude_length) = Address::Local(LocalAddress::ToChild).to_first_packet();

        Self {
            first_packet,
            prelude_length,
            data,
        }
    }

    /// Create a packet with given data addresses to a node's parent
    pub fn to_parent(data: Data) -> Self {
        let (first_packet, prelude_length) = Address::Local(LocalAddress::ToParent).to_first_packet();

        Self {
            first_packet,
            prelude_length,
            data,
        }
    }

    /// Create a packet with given network address
    ///
    /// In this case, addresses are of the format [a, b, c, d, ...] to make this easier
    /// to use
    pub fn to_address(from: &[u8], to: &[u8], data: Data) -> Self {
        let (first_packet, prelude_length) = Address::Network { from, to }.to_first_packet();

        Self {
            first_packet,
            prelude_length,
            data,
        }
    }

    pub fn pack_payload(mut self) -> Result<Box<[[u8; 32]]>, PackError> {
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
        let address = Address::Network{ from: &from, to: &to };
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
        let address = Address::Network{ from: &from, to: &to };
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
        let address = Address::Network { from: &from, to: &to };
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
        let address = Address::Network { from: &from, to: &to };
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
        let packet = Packet::to_parent(0x3F21u16);
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [0u8; 32];
        expected_payload[0] = 0xA0;
        expected_payload[1] = 0x3F;
        expected_payload[2] = 0x21;

        assert_eq!(payload[0], expected_payload);
    }

    #[test]
    fn test_to_child_pack_u16() {
        let packet = Packet::to_child(0x3214u16);
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [0u8; 32];
        expected_payload[0] = 0x90;
        expected_payload[1] = 0x32;
        expected_payload[2] = 0x14;

        assert_eq!(payload[0], expected_payload);
    }

    #[test]
    fn test_to_network_pack_u16() {
        let packet = Packet::to_address(&[1, 2, 3], &[3, 2, 1], 0x1923u16);
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [0u8; 32];
        expected_payload[0] = 0x12;
        expected_payload[1] = 0x30;
        expected_payload[2] = 0x32;
        expected_payload[3] = 0x10;
        expected_payload[4] = 0x19;
        expected_payload[5] = 0x23;

        assert_eq!(payload[0], expected_payload);
    }

    #[test]
    fn test_to_parent_pack_large() {
        let mut data = [0u32; 32];
        for i in 0..32 {
            data[i] = i as u32;
        }
        let packet = Packet::to_parent(data);
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [[0u8; 32]; 5];
        expected_payload[0][0] = 0xA0;
        for i in 0..32 {
            let x = (4 * i) + 4;
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
        let packet = Packet::to_child(data);
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [[0u8; 32]; 5];
        expected_payload[0][0] = 0x90;
        for i in 0..32 {
            let x = (4 * i) + 4;
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
        let packet = Packet::to_address(&[1, 2, 3], &[3, 2, 1], data);
        let payload = packet.pack_payload().unwrap();

        let mut expected_payload = [[0u8; 32]; 5];
        expected_payload[0][0] = 0x12;
        expected_payload[0][1] = 0x30;
        expected_payload[0][2] = 0x32;
        expected_payload[0][3] = 0x10;
        for i in 0..32 {
            let x = (4 * i) + 7;
            expected_payload[x/32][x%32] = i as u8;
        }

        assert_eq!(*payload, expected_payload);
    }
}