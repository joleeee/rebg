use std::collections::HashMap;

#[derive(Debug)]
pub struct MCell {
    values: Vec<(u32, u8)>,
}

impl MCell {
    fn new() -> Self {
        Self { values: Vec::new() }
    }

    fn add_tick(&mut self, tick: u32, value: u8) -> Result<(), ()> {
        // is there already a tick after what we're adding?
        if Some(tick) < self.values.last().map(|x| x.0) {
            return Err(());
        }

        // is the previous tick equal to the current tick?
        if let Some((t, v)) = self.values.last_mut() {
            // note,       /\ this is not a comparison
            if *t == tick {
                *v = value;
                return Ok(());
            }
        }

        // otherwise, we can just push!
        self.values.push((tick, value));
        Ok(())
    }

    fn at_tick(&self, tick: u32) -> Option<u8> {
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
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    pub fn load8(&self, tick: u32, adr: u64) -> Option<u8> {
        self.cells.get(&adr)?.at_tick(tick)
    }

    pub fn load16(&self, tick: u32, adr: u64) -> Option<u16> {
        Some(u16::from_be_bytes([
            self.load8(tick, adr)?,
            self.load8(tick, adr + 1)?,
        ]))
    }

    pub fn load32(&self, tick: u32, adr: u64) -> Option<u32> {
        Some(u32::from_be_bytes([
            self.load8(tick, adr)?,
            self.load8(tick, adr + 1)?,
            self.load8(tick, adr + 2)?,
            self.load8(tick, adr + 3)?,
        ]))
    }

    pub fn load64(&self, tick: u32, adr: u64) -> Option<u64> {
        Some(u64::from_be_bytes([
            self.load8(tick, adr)?,
            self.load8(tick, adr + 1)?,
            self.load8(tick, adr + 2)?,
            self.load8(tick, adr + 3)?,
            self.load8(tick, adr + 4)?,
            self.load8(tick, adr + 5)?,
            self.load8(tick, adr + 6)?,
            self.load8(tick, adr + 7)?,
        ]))
    }

    pub fn store8(&mut self, tick: u32, adr: u64, val: u8) -> Result<(), ()> {
        if !self.cells.contains_key(&adr) {
            self.cells.insert(adr, MCell::new());
        }

        let cell = self.cells.get_mut(&adr).expect("logic error");

        cell.add_tick(tick, val)?;
        Ok(())
    }

    // TODO, <T: Num> or something?
    pub fn store16(&mut self, tick: u32, adr: u64, val: u16) -> Result<(), ()> {
        for (offset, byte) in val.to_be_bytes().into_iter().enumerate() {
            self.store8(tick, adr + offset as u64, byte)?;
        }
        Ok(())
    }

    pub fn store32(&mut self, tick: u32, adr: u64, val: u32) -> Result<(), ()> {
        for (offset, byte) in val.to_be_bytes().into_iter().enumerate() {
            self.store8(tick, adr + offset as u64, byte)?;
        }
        Ok(())
    }

    pub fn store64(&mut self, tick: u32, adr: u64, val: u64) -> Result<(), ()> {
        for (offset, byte) in val.to_be_bytes().into_iter().enumerate() {
            self.store8(tick, adr + offset as u64, byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::HistMem;

    use super::MCell;

    #[test]
    fn ticks() {
        let mut cell = MCell { values: vec![] };
        cell.add_tick(4, 0x11).unwrap();
        cell.add_tick(7, 0x22).unwrap();
        cell.add_tick(8, 0x33).unwrap();

        dbg!(&cell);

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
    fn overlapping_stores() {
        let mut m = HistMem::new();
        m.store64(0, 0x1234, 0x1111111111111111).unwrap();
        m.store64(0, 0x1234, 0x2222222222222222).unwrap();

        assert_eq!(m.load64(0, 0x1234), Some(0x2222222222222222))
    }

    #[test]
    fn stores() {
        const TICK: u32 = 555;
        let mut v = HistMem::new();

        v.store64(TICK - 1, 0, 0x1111111111111111).unwrap();
        v.store64(TICK - 1, 8, 0x2222222222222222).unwrap();

        // u64
        v.store64(TICK, 1, 0xFFFFFFFFFFFFFFFF).unwrap();

        assert_eq!(v.load64(TICK, 0), Some(0x11FFFFFFFFFFFFFF));
        assert_eq!(v.load64(TICK, 8), Some(0xFF22222222222222));

        // u32
        v.store32(TICK + 1, 7, 0x77777777).unwrap();
        assert_eq!(v.load64(TICK + 1, 0), Some(0x11FFFFFFFFFFFF77));
        assert_eq!(v.load64(TICK + 1, 8), Some(0x7777772222222222));

        v.store32(TICK + 2, 1, 0x44444444).unwrap();
        assert_eq!(v.load64(TICK + 2, 0), Some(0x1144444444FFFF77));
        assert_eq!(v.load64(TICK + 2, 8), Some(0x7777772222222222));

        v.store32(TICK + 3, 2, 0x55555555).unwrap();
        assert_eq!(v.load64(TICK + 3, 0), Some(0x114455555555FF77));
        assert_eq!(v.load64(TICK + 3, 8), Some(0x7777772222222222));

        v.store32(TICK + 4, 5, 0x66666666).unwrap();
        assert_eq!(v.load64(TICK + 4, 0), Some(0x1144555555666666));
        assert_eq!(v.load64(TICK + 4, 8), Some(0x6677772222222222));

        v.store32(TICK + 5, 10, 0x99999999).unwrap();
        assert_eq!(v.load64(TICK + 5, 0), Some(0x1144555555666666));
        assert_eq!(v.load64(TICK + 5, 8), Some(0x6677999999992222));

        v.store64(TICK + 6, 0, 0x0000000000000000).unwrap();
        v.store64(TICK + 6, 8, 0x0000000000000000).unwrap();
        for i in 1..12 {
            let mut val = 0;
            for ind in 0..8 {
                val |= i << (ind * 4);
            }
            v.store32(TICK + 7 + i, i.into(), val as u32).unwrap();
        }

        assert_eq!(v.load64(TICK + 30, 0), Some(0x0011223344556677));
        assert_eq!(v.load64(TICK + 30, 8), Some(0x8899aabbbbbbbb00));

        // u16
        v.store64(TICK + 100, 0, 0x4444444444444444).unwrap();
        v.store64(TICK + 100, 8, 0x4444444444444444).unwrap();

        v.store16(TICK + 101, 0, 0xffff).unwrap();
        v.store16(TICK + 102, 2, 0xeeee).unwrap();
        v.store16(TICK + 103, 4, 0xdddd).unwrap();
        v.store16(TICK + 104, 5, 0x9999).unwrap();
        v.store16(TICK + 105, 6, 0x8888).unwrap();
        v.store16(TICK + 106, 7, 0x7777).unwrap();
        v.store16(TICK + 107, 8, 0x6666).unwrap();
        v.store16(TICK + 108, 9, 0x5555).unwrap();
        v.store16(TICK + 109, 10, 0x0000).unwrap();

        // println!("{:016x}", v.load64(TICK + 110, 0).unwrap());
        // println!("{:016x}", v.load64(TICK + 110, 8).unwrap());

        assert_eq!(v.load64(TICK + 110, 0), Some(0xffffeeeedd998877));
        assert_eq!(v.load64(TICK + 110, 8), Some(0x6655000044444444));

        // u8
        v.store64(TICK + 200, 0, 0x2222222222222222).unwrap();
        v.store64(TICK + 200, 8, 0x2222222222222222).unwrap();

        v.store8(TICK + 201, 0, 0x33).unwrap();
        v.store8(TICK + 202, 3, 0x33).unwrap();
        v.store8(TICK + 203, 7, 0x88).unwrap();
        v.store8(TICK + 204, 8, 0x99).unwrap();
        v.store8(TICK + 205, 9, 0x77).unwrap();

        assert_eq!(v.load64(TICK + 210, 0), Some(0x3322223322222288));
        assert_eq!(v.load64(TICK + 210, 8), Some(0x9977222222222222));
    }

    #[test]
    fn loads() {
        const TICK: u32 = 555;

        let mut v = HistMem::new();

        v.store64(TICK, 0, 0x1111111111111111).unwrap();
        v.store64(TICK, 8, 0x2222222222222222).unwrap();

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
