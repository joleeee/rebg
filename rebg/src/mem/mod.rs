//!

use std::collections::HashMap;

// We want to keep track of all memory. We do this by having a slot of each
// 64bit of memory, and then when it changes, adding an update.
//
// We may need to fetch or write to multiple, because load/store is not
// necessarily aligned. Since we use u64, and the max read is u64, we at most
// need to access 2 places.

pub struct MCell {
    values: Vec<(u32, u64)>,
}

impl MCell {
    fn new() -> Self {
        Self { values: Vec::new() }
    }

    fn add_tick(&mut self, tick: u32, value: u64) -> Result<(), ()> {
        if Some(tick) <= self.values.last().map(|x| x.0) {
            return Err(());
        }
        self.values.push((tick, value));
        Ok(())
    }

    fn at_tick(&self, tick: u32) -> Option<u64> {
        let idx = self.values.partition_point(|(t, _v)| *t <= tick);

        // we get idx 1 too high
        // so if it's 0, that means no too early (cus index -1)
        if idx == 0 {
            return None;
        }

        Some(self.values.get(idx - 1).unwrap().1)
    }
}

pub struct HistMem {
    // ptr -> value
    cells: HashMap<u64, MCell>,
}

impl HistMem {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

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

    pub fn load64aligned(&self, tick: u32, adr: u64) -> Option<u64> {
        self.cells.get(&adr)?.at_tick(tick)
    }

    pub fn load64(&self, tick: u32, address: u64) -> Option<u64> {
        let adr_lower = Self::align_down(address);
        let adr_upper = Self::align_down(address + 7);

        let upper_len = address - adr_lower;
        let lower_len = 8 - upper_len;

        let lower = self.load64aligned(tick, adr_lower)?;
        let upper = self.load64aligned(tick, adr_upper)?;

        let lower = lower & Self::get_lower_bitmask(lower_len);
        let upper = upper & Self::get_upper_bitmask(upper_len);

        Some(lower | upper)
    }

    pub fn load32(&self, tick: u32, address: u64) -> Option<u32> {
        let adr_lower = Self::align_down(address);
        let adr_upper = Self::align_down(address + 3);

        let upper_len = address - adr_lower;
        let lower_len = 8 - upper_len;

        let lower = self.load64aligned(tick, adr_lower)?;
        let upper = self.load64aligned(tick, adr_upper)?;

        let lower = lower & Self::get_lower_bitmask(lower_len);
        let upper = upper & Self::get_upper_bitmask(upper_len);

        let combined = lower | upper;
        let combined = (combined >> 32) as u32;

        Some(combined)
    }

    pub fn load16(&self, tick: u32, address: u64) -> Option<u16> {
        let adr_lower = Self::align_down(address);
        let adr_upper = Self::align_down(address + 1);

        let upper_len = address - adr_lower;
        let lower_len = 8 - upper_len;

        let lower = self.load64aligned(tick, adr_lower)?;
        let upper = self.load64aligned(tick, adr_upper)?;

        let lower = lower & Self::get_lower_bitmask(lower_len);
        let upper = upper & Self::get_upper_bitmask(upper_len);

        let combined = lower | upper;
        let combined = (combined >> 48) as u16;

        Some(combined)
    }

    pub fn load8(&self, tick: u32, address: u64) -> Option<u8> {
        let adr_lower = Self::align_down(address);
        let offset = address - adr_lower;

        let lower = self.load64aligned(tick, adr_lower)? >> (offset * 8);
        let lower = (lower & 0xFF) as u8;

        Some(lower)
    }

    pub fn store64aligned(&mut self, tick: u32, address: u64, v: u64) -> Result<(), ()> {
        if !self.cells.contains_key(&address) {
            self.cells.insert(address, MCell::new());
        }

        let cell = self.cells.get_mut(&address).expect("logic error");

        cell.add_tick(tick, v)?;
        Ok(())
    }

    pub fn store64(&mut self, tick: u32, address: u64, value: u64) -> Result<(), ()> {
        let adr_lower = Self::align_down(address);
        let adr_upper = Self::align_down(address + 7);

        let upper_len = address - adr_lower;
        let lower_len = 8 - upper_len;

        // instead of keeping only that which inside the bitmask, in this case
        // we actually want to replace those parts.

        let lower_existing = self.load64aligned(tick, adr_lower).ok_or(())?;
        let upper_existing = self.load64aligned(tick, adr_upper).ok_or(())?;

        let lower_existing = lower_existing & !Self::get_upper_bitmask(lower_len); // note the reversed upper
        let lower_overwrite = value >> (upper_len * 8);
        let lower = lower_existing | lower_overwrite;

        let upper_existing = upper_existing & !Self::get_lower_bitmask(upper_len);
        let upper_overwrite = value << (lower_len * 8);
        let upper = upper_existing | upper_overwrite;

        self.store64aligned(tick, adr_lower, lower)?;
        self.store64aligned(tick, adr_upper, upper)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::HistMem;

    use super::MCell;

    #[test]
    fn ticks() {
        let mut cell = MCell { values: vec![] };
        cell.add_tick(4, 0x11).unwrap();
        cell.add_tick(7, 0x22).unwrap();
        cell.add_tick(8, 0x33).unwrap();

        assert_eq!(cell.at_tick(0), None);
        assert_eq!(cell.at_tick(3), None);
        assert_eq!(cell.at_tick(4), Some(0x11));
        assert_eq!(cell.at_tick(5), Some(0x11));
        assert_eq!(cell.at_tick(6), Some(0x11));
        assert_eq!(cell.at_tick(7), Some(0x22));
        assert_eq!(cell.at_tick(8), Some(0x33));
        assert_eq!(cell.at_tick(9), Some(0x33));
        assert_eq!(cell.at_tick(9999999), Some(0x33));
    }

    #[test]
    fn alignment() {
        // first 8 bytes reside at the same address
        for a in 0..8 {
            assert_eq!(HistMem::align_down(a), 0);
        }

        for a in 8..16 {
            assert_eq!(HistMem::align_down(a), 8);
        }
    }

    #[test]
    fn stores() {
        const TICK: u32 = 555;
        let mut v = HistMem::new();

        v.store64aligned(TICK - 1, 0, 0x1111111111111111).unwrap();
        v.store64aligned(TICK - 1, 8, 0x2222222222222222).unwrap();

        v.store64(TICK, 1, 0xFFFFFFFFFFFFFFFF).unwrap();

        assert_eq!(v.load64aligned(TICK, 0), Some(0x11FFFFFFFFFFFFFF));
        assert_eq!(v.load64aligned(TICK, 8), Some(0xFF22222222222222));
    }

    #[test]
    fn loads() {
        const TICK: u32 = 555;

        let mut v = HistMem::new();

        v.store64aligned(TICK, 0, 0x1111111111111111).unwrap();
        v.store64aligned(TICK, 8, 0x2222222222222222).unwrap();

        // u64
        for a in 0..=8 {
            println!("64 {:02x}: {:016x}", a, v.load64(TICK, a).unwrap());
        }
        assert_eq!(v.load64(TICK, 0), Some(0x1111111111111111));
        assert_eq!(v.load64(TICK, 1), Some(0x1111111111111122));
        assert_eq!(v.load64(TICK, 7), Some(0x1122222222222222));
        assert_eq!(v.load64(TICK, 8), Some(0x2222222222222222));
        assert_eq!(v.load64(TICK, 9), None);

        // u32
        println!("");
        for a in 0..10 {
            println!("32 {:02x}: {:08x}", a, v.load32(TICK, a).unwrap());
        }
        assert_eq!(v.load32(TICK, 0), Some(0x11111111));
        assert_eq!(v.load32(TICK, 4), Some(0x11111111));
        assert_eq!(v.load32(TICK, 5), Some(0x11111122));
        assert_eq!(v.load32(TICK, 6), Some(0x11112222));
        assert_eq!(v.load32(TICK, 7), Some(0x11222222));
        assert_eq!(v.load32(TICK, 8), Some(0x22222222));
        assert_eq!(v.load32(TICK, 9), Some(0x22222222));
        assert_eq!(v.load32(TICK, 12), Some(0x22222222));
        assert_eq!(v.load32(TICK, 13), None);

        // u16
        println!("");
        for a in 0..10 {
            println!("16 {:02x}: {:04x}", a, v.load16(TICK, a).unwrap());
        }
        assert_eq!(v.load16(TICK, 0), Some(0x1111));
        assert_eq!(v.load16(TICK, 6), Some(0x1111));
        assert_eq!(v.load16(TICK, 7), Some(0x1122));
        assert_eq!(v.load16(TICK, 8), Some(0x2222));
        assert_eq!(v.load16(TICK, 14), Some(0x2222));
        assert_eq!(v.load16(TICK, 15), None);

        // u8
        for a in 0..8 {
            assert_eq!(v.load8(TICK, a), Some(0x11));
        }
        for a in 8..16 {
            assert_eq!(v.load8(TICK, a), Some(0x22));
        }
        assert_eq!(v.load8(TICK, 16), None);
    }
}
