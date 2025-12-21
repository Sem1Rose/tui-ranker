#[derive(Debug, PartialEq, Clone)]
pub struct BitMask(Vec<u8>);

impl BitMask {
    // pub fn append(&mut self, value: u8) {
    //     self.0.push(value);
    // }
    pub fn extend_by_bytes(&mut self, extension: u16) {
        for _ in 0..extension {
            self.0.push(0u8);
        }
        // self.0.append(&mut vec![0u8; extension as usize]);
    }
    fn shrink_by_bytes(&mut self, shrinkage: u16) {
        if self.0.len() < shrinkage as usize {
            panic!(
                "trying to shrink by {shrinkage} bytes but bitmask size is {}",
                self.0.len()
            );
        }
        for _ in 0..shrinkage {
            _ = self.0.pop();
        }
    }
    pub fn fit_to_number_of_bits(&mut self, size: u16) {
        let bytes = (size >> 3) + if size & 0b111 != 0 { 1 } else { 0 };
        self.fit_to_bytes(bytes);
    }
    pub fn fit_to_bytes(&mut self, bytes: u16) {
        if self.0.len() > bytes as usize {
            self.shrink_by_bytes(self.0.len() as u16 - bytes);
        } else if self.0.len() < bytes as usize {
            self.extend_by_bytes(bytes - self.0.len() as u16);
        }
    }
    pub fn discard_bit(&mut self, id: u16) {
        let div = id >> 3; // div by 8
        let rem = id & 7; // rem of 8

        if self.0.len() <= div as usize {
            panic!(
                "index out of bounds: trying to discard from byte {div} but the length is {}",
                self.0.len()
            );
        }

        let mut prev_carry = 0;
        for i in ((div as usize + 1)..self.0.len()).rev() {
            let new_carry = self.0[i] & 1;
            self.0[i] >>= 1;
            self.0[i] |= prev_carry << 7;
            prev_carry = new_carry;
        }

        self.0[div as usize] =
            ((self.0[div as usize] & 0xffu8.checked_shl(rem as u32 + 1).unwrap_or(0)) >> 1)
                | (self.0[div as usize] & 0xffu8.checked_shr(8 - rem as u32).unwrap_or(0))
                | (prev_carry << 7);
    }
    pub fn get_num_ones(&self) -> u16 {
        self.0.iter().fold(0, |acc, x| acc + x.count_ones() as u16)
    }
    pub fn check_bit(&self, rhs: u16) -> bool {
        let div = rhs >> 3; // div by 8
        let rem = rhs & 7; // rem of 8

        if self.0.len() <= div as usize {
            panic!(
                "index out of bounds: trying to get byte {} but the length is {}",
                div,
                self.0.len(),
            );
        }

        (self.0[div as usize] >> rem) & 1 == 1
    }
    pub fn set_bit(&mut self, index: u16) {
        let div = index >> 3; // div by 8
        let rem = index & 7; // rem of 8

        if self.0.len() <= div as usize {
            self.extend_by_bytes(div + 1 - self.0.len() as u16);
        }

        self.0[div as usize] |= 1 << rem;
    }
    pub fn reset_bit(&mut self, index: u16) {
        let div = index >> 3; // div by 8
        let rem = index & 7; // rem of 8

        if self.0.len() <= div as usize {
            panic!(
                "index out of bounds: trying to reset byte {} but the length is {}",
                div,
                self.0.len(),
            );
        }

        self.0[div as usize] &= 0xff ^ (1 << rem);
    }

    // pub fn get_byte(&self, index: u16) -> u8 {
    //     if index as usize >= self.0.len() {
    //         panic!(
    //             "index out of bounds: trying to access byte {} but the length is {}",
    //             index,
    //             self.0.len(),
    //         );
    //     }

    //     self.0[index as usize]
    // }
    // pub fn inc_byte(&mut self, index: u16) {
    //     if index as usize >= self.0.len() {
    //         panic!(
    //             "index out of bounds: trying to increment byte {} but the length is {}",
    //             index,
    //             self.0.len(),
    //         );
    //     }

    //     self.0[index as usize] += 1;
    // }
    // pub fn set_byte(&mut self, index: u16, value: u8) {
    //     if index as usize >= self.0.len() {
    //         panic!(
    //             "index out of bounds: trying to reset byte {} but the length is {}",
    //             index,
    //             self.0.len(),
    //         );
    //     }

    //     self.0[index as usize] = value;
    // }
    // pub fn discard_byte(&mut self, index: u16) {
    //     if index as usize >= self.0.len() {
    //         panic!(
    //             "index out of bounds: trying to discard byte {} but the length is {}",
    //             index,
    //             self.0.len(),
    //         );
    //     }

    //     self.0.remove(index as usize);
    // }
    pub fn ref_vec(&self) -> &Vec<u8> {
        &self.0
    }
}

impl From<u16> for BitMask {
    // TODO: check if the num is larger than the max num items 2040
    fn from(value: u16) -> Self {
        let div = value >> 3; // div by 8
        let rem = value & 7; // rem of 8

        let mut vec = vec![0u8; div as usize];
        vec.push(1u8 << rem);

        Self(vec)
    }
}
impl From<Vec<u8>> for BitMask {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod bitmask_tests {
    use super::*;

    #[test]
    fn test_from() {
        let bitmask: BitMask = 10.into();
        assert_eq!(bitmask, vec![0, 0b00000100].into());
        let bitmask: BitMask = 0.into();
        assert_eq!(bitmask, vec![0b00000001].into())
    }

    #[test]
    #[should_panic = "index out of bounds: trying to get byte 2 but the length is 2"]
    fn test_and_or() {
        let mut bitmask: BitMask = 10.into();
        assert_eq!(bitmask, vec![0, 0x04].into());

        bitmask.set_bit(5);
        assert!(bitmask.check_bit(10));
        assert!(bitmask.check_bit(5));
        assert_eq!(bitmask, vec![0x20, 0x04].into());

        _ = bitmask.check_bit(16);
    }

    #[test]
    #[should_panic = "trying to shrink by 1 bytes but bitmask size is 0"]
    fn test_shrink_extend() {
        let mut bitmask: BitMask = vec![].into();

        bitmask.extend_by_bytes(1);
        assert_eq!(bitmask, vec![0].into());

        bitmask.extend_by_bytes(17);
        bitmask.shrink_by_bytes(18);

        bitmask.shrink_by_bytes(1)
    }

    #[test]
    fn test_discard() {
        let mut bitmask: BitMask = 0.into();
        bitmask.set_bit(8);
        bitmask.discard_bit(0);
        bitmask.discard_bit(7);

        assert_eq!(bitmask, vec![0, 0].into());

        bitmask = vec![0b11011101, 0b10010001, 0b00000010].into();
        bitmask.discard_bit(3);
        assert_eq!(bitmask, vec![0b11101101, 0b01001000, 0b00000001].into());
        bitmask.discard_bit(14);
        assert_eq!(bitmask, vec![0b11101101, 0b10001000, 0].into());
    }
}
