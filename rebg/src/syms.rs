use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolReference<'a> {
    pub symbol: &'a Symbol,
    pub offset: u64, // how much after the start of the symbol
}

impl<'a> Display for SymbolReference<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol.name)?;
        if self.symbol.to != self.symbol.from {
            write!(f, "+{}", self.offset)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub from: u64,
    pub to: u64,
}

#[derive(Debug, Clone)]
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
                size: ph.p_filesz, // memsz is possibly bigger because it contains bss (zeroed variables / data)
            })
            .collect::<Vec<_>>();

        let mut symbols = Vec::new();
        for sym in &elf.syms {
            let name = elf
                .strtab
                .get_at(sym.st_name)
                .expect("back to you, elf is sketchy");

            // https://sourceware.org/binutils/docs/as/AArch64-Mapping-Symbols.html
            if matches!(name, "$d" | "$x") {
                continue;
            }

            let base = sym.st_value;
            let size = sym.st_size;

            // find the header it is in
            let offset = offsets
                .iter()
                .find(|o| o.addr <= base && base < o.addr + o.size);
            let offset = if let Some(offset) = offset {
                offset
            } else {
                continue;
            };

            // remove any offset which in the symbol so it's now just realtive to the runtime
            // address
            let base = base + offset.offset - offset.addr;

            symbols.push(Symbol {
                name: name.to_string(),
                from: base,
                to: base + size,
            });
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
            .find(|s| s.from <= adr && adr <= s.to)
            .map(|s| SymbolReference {
                offset: adr - s.from,
                symbol: s,
            })
    }

    pub fn merge(self, mut other: Self) -> Self {
        let mut symbols = self.symbols;
        symbols.append(&mut other.symbols);
        Self { symbols }
    }
}

#[cfg(test)]
mod tests {
    use super::{Symbol, SymbolReference, SymbolTable};

    #[test]
    fn pie() {
        // setup
        let sym1 = Symbol {
            name: "main".to_string(),
            from: 0x0,
            to: 0x200,
        };

        let sym2 = Symbol {
            name: "somevar".to_string(),
            from: 0x800,
            to: 0x800,
        };

        let table = SymbolTable {
            symbols: vec![sym1.clone(), sym2.clone()],
        };

        // do the pie offset
        let pie_table = table.pie(0x40_000);

        // this was inside main, but now main is offset by 0x40_000
        assert_eq!(pie_table.lookup(0x100), None);

        // main should now be here
        let sym1_pie = Symbol {
            name: sym1.name,
            from: sym1.from + 0x40_000,
            to: sym1.to + 0x40_000,
        };

        // test boundries
        assert_eq!(pie_table.lookup(0x39_fff), None);
        assert_eq!(
            pie_table.lookup(0x40_000),
            Some(SymbolReference {
                symbol: &sym1_pie,
                offset: 0x000
            })
        );
        assert_eq!(
            pie_table.lookup(0x40_200),
            Some(SymbolReference {
                symbol: &sym1_pie,
                offset: 0x200
            })
        );
        assert_eq!(pie_table.lookup(0x40_201), None);

        // the variable
        assert!(pie_table.lookup(0x40_7ff).is_none());
        assert!(pie_table.lookup(0x40_800).is_some());
        assert!(pie_table.lookup(0x40_801).is_none());
    }
}
