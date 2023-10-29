//!

use std::collections::HashMap;

// We want to keep track of all memory. We do this by having a slot of each
// 64bit of memory, and then when it changes, adding an update.
//
// We may need to fetch or write to multiple, because load/store is not
// necessarily aligned. Since we use u64, and the max read is u64, we at most
// need to access 2 places.

pub struct MCell {
    value: u64,
}

pub struct HistMem {
    // ptr -> value
    cells: HashMap<u64, MCell>,
}

impl HistMem {
    fn align_down(address: u64) -> u64 {
        const SHIFT: usize = 3; // 2**3 = 8
        (address >> SHIFT) << SHIFT
    }

    const fn get_lower_bitmask(bytecnt: u64) -> u64 {
        match bytecnt {
            0 => 0,
            1 => 0xFF00000000000000,
            2 => 0xFFFF000000000000,
            3 => 0xFFFFFF0000000000,
            4 => 0xFFFFFFFF00000000,
            5 => 0xFFFFFFFFFF000000,
            6 => 0xFFFFFFFFFFFF0000,
            7 => 0xFFFFFFFFFFFFFF00,
            8 => 0xFFFFFFFFFFFFFFFF,
            _ => Self::get_lower_bitmask(8),
        }
    }

    const fn get_upper_bitmask(bytecnt: u64) -> u64 {
        match bytecnt {
            0 => 0,
            1 => 0xFF,
            2 => 0xFFFF,
            3 => 0xFFFFFF,
            4 => 0xFFFFFFFF,
            5 => 0xFFFFFFFFFF,
            6 => 0xFFFFFFFFFFFF,
            7 => 0xFFFFFFFFFFFFFF,
            8 => 0xFFFFFFFFFFFFFFFF,
            _ => Self::get_upper_bitmask(8),
        }
    }

    pub fn load64aligned(&self, adr: u64) -> Option<u64> {
        self.cells.get(&adr).map(|c| c.value)
    }

    pub fn load64(&self, address: u64) -> Option<u64> {
        let adr_lower = Self::align_down(address);
        let adr_upper = adr_lower + 8;

        if adr_lower == address {
            return self.load64aligned(adr_lower);
        }

        let upper_len = address - adr_lower;
        let lower_len = 8 - upper_len;

        let lower = self.load64aligned(adr_lower)?;
        let upper = self.load64aligned(adr_upper)?;

        // dbg!((address, adr_lower, adr_upper, lower_len, upper_len));
        // println!("{:016x}", lower & Self::get_lower_bitmask(lower_len));

        let lower = lower & Self::get_lower_bitmask(lower_len);
        let upper = upper & Self::get_upper_bitmask(upper_len);

        Some(lower | upper)
    }

    pub fn store64aligned(&mut self, address: u64, v: u64) {
        self.cells.insert(address, MCell { value: v });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::mem::HistMem;

    #[test]
    fn alignment() {
        // first 8 bytes reside at the same address
        for a in 0..8 {
            assert_eq!(HistMem::align_down(a), 0);
        }

        for a in 8..16 {
            assert_eq!(HistMem::align_down(a), 8);
        }

        let mut v = HistMem {
            cells: HashMap::new(),
        };

        v.store64aligned(0, 0x1111111111111111);
        v.store64aligned(8, 0x2222222222222222);

        for a in 0..8 {
            println!("{:02x}: {:016x}", a, v.load64(a).unwrap());
        }

        assert_eq!(v.load64(0), Some(0x1111111111111111));
        assert_eq!(v.load64(1), Some(0x1111111111111122));
        assert_eq!(v.load64(7), Some(0x1122222222222222));
        assert_eq!(v.load64(8), Some(0x2222222222222222));
        assert_eq!(v.load64(9), None);
    }
}
