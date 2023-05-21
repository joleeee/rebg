use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct SymbolReference {
    pub symbol: Symbol,
    pub offset: u64, // how much after the start of the symbol
}

impl Display for SymbolReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}+{:x}", self.symbol.name, self.offset)
    }
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub from: u64,
    to: u64,
}

#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

// https://developer.arm.com/documentation/100748/0620/Mapping-Code-and-Data-to-the-Target/Loading-armlink-generated-ELF-files-that-have-complex-scatter-files
impl SymbolTable {
    pub fn from_elf(elf: goblin::elf::Elf) -> Self {
        // TODO support different vaddr and paddr
        for ph in &elf.program_headers {
            assert_eq!(ph.p_vaddr, ph.p_paddr);
        }

        #[derive(Debug)]
        struct ProgramOffset {
            offset: u64,
            addr: u64,
            size: u64,
        }

        let offsets = elf
            .program_headers
            .iter()
            .filter(|ph| ph.p_type == goblin::elf::program_header::PT_LOAD)
            .map(|ph| ProgramOffset {
                offset: ph.p_offset,
                addr: ph.p_vaddr,
                size: ph.p_filesz, // memsz is bigger because it contains bss (uninitd data)
            })
            .collect::<Vec<_>>();

        // X64
        // ProgramHeader {
        //     p_type: "PT_LOAD",
        //     p_flags: 0x4,
        //     p_offset: 0x0,
        //     p_vaddr: 0x0,
        //     p_paddr: 0x0,
        //     p_filesz: 0x628,
        //     p_memsz: 0x628,
        //     p_align: 4096,
        // },
        // ProgramHeader {
        //     p_type: "PT_LOAD",
        //     p_flags: 0x5,
        //     p_offset: 0x1000,
        //     p_vaddr: 0x1000,
        //     p_paddr: 0x1000,
        //     p_filesz: 0x1a1,
        //     p_memsz: 0x1a1,
        //     p_align: 4096,
        // },

        // ARM64
        //
        // ProgramHeader {
        //     p_type: "PT_LOAD",
        //     p_flags: 0x5,
        //     p_offset: 0x0,
        //     p_vaddr: 0x400000,
        //     p_paddr: 0x400000,
        //     p_filesz: 0x7cc,
        //     p_memsz: 0x7cc,
        //     p_align: 65536,
        // },

        // we have to take all symbols and move them
        // symbol.from = symbol.from - offset + vaddr/paddr

        let mut symbols = Vec::new();
        for sym in &elf.syms {
            let name = elf
                .strtab
                .get_at(sym.st_name)
                .expect("back to you, elf is sketchy");
            let base = sym.st_value;
            let size = sym.st_size;

            // find the header it is in
            let offset = offsets
                .iter()
                .find(|o| o.addr <= base && base < o.addr + o.size);
            let offset = if let Some(offset) = offset {
                offset
            } else {
                //println!("{} not in any header", name);
                continue;
            };

            // remove any offset which in the symbol so it's now just realtive to the runtime
            // address
            let base = base + offset.offset - offset.addr;

            // then add the location of the binary (cus PIE)

            if name == "main" {
                println!("container, base: {:x}", base);
                dbg!(&offset);
            }

            symbols.push(Symbol {
                name: name.to_string(),
                from: base,
                to: base + size,
            });

            // 40059c: get_sum
        }

        Self { symbols }
    }

    /// offset based on where the binary is loaded
    pub fn pie(self, base: u64) -> Self {
        let symbols = self
            .symbols
            .into_iter()
            .map(|Symbol { name, from, to }| Symbol {
                name,
                from: from + base,
                to: to + base,
            })
            .collect();

        Self { symbols }
    }

    pub fn lookup(&self, adr: u64) -> Option<SymbolReference> {
        self.symbols
            .iter()
            .find(|s| s.from <= adr && adr < s.to)
            .map(|s| s.clone())
            .map(|s| SymbolReference {
                offset: adr - s.from,
                symbol: s,
            })
    }
}
