use std::fmt::{Display, Formatter};
use tracing::debug;

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
    pub offsets: Vec<ProgramOffset>,
    pub fallback: Option<Box<SymbolTable>>,
    /// Source of the binary this symbols applies to. If there is a separate
    /// debug info file, that should be disregarded wrt. this variable.
    pub binary_path: String,
}
#[derive(Debug, Clone, Copy)]
pub struct ProgramOffset {
    offset: u64,
    addr: u64,
    size: u64,
}

// https://developer.arm.com/documentation/100748/0620/Mapping-Code-and-Data-to-the-Target/Loading-armlink-generated-ELF-files-that-have-complex-scatter-files
impl SymbolTable {
    /// The PT_LOAD headers
    fn get_offsets(elf: &goblin::elf::Elf) -> Vec<ProgramOffset> {
        elf.program_headers
            .iter()
            .filter(|ph| ph.p_type == goblin::elf::program_header::PT_LOAD)
            .map(|ph| ProgramOffset {
                offset: ph.p_offset,
                addr: ph.p_vaddr,
                size: ph.p_memsz, // memsz is possibly bigger than filesz because it contains bss
                                  // (default zeroed variables / data)
            })
            .collect()
    }

    pub fn intermediary_symbols(elf: &goblin::elf::Elf) -> Vec<(String, u64, u64)> {
        elf.syms
            .iter()
            .map(|sym| {
                let name = elf
                    .strtab
                    .get_at(sym.st_name)
                    .expect("back to you, elf is sketchy");
                let addr = sym.st_value;
                let size = sym.st_size;
                (name.to_string(), addr, size)
            })
            .collect()
    }

    /// Extend an existing elf with more debug symbols
    pub fn extend_with_debug(self, debug_elf: &goblin::elf::Elf, from: u64, to: u64) -> Self {
        // TODO support different vaddr and paddr
        for ph in &debug_elf.program_headers {
            assert_eq!(ph.p_vaddr, ph.p_paddr);
        }

        let syms = Self::intermediary_symbols(debug_elf);

        for o in &self.offsets {
            debug!("offset: {:#x} {:#x}", o.addr, o.addr + o.size);
        }

        let mut symbols = self.symbols;

        for (name, addr, size) in syms {
            // https://sourceware.org/binutils/docs/as/AArch64-Mapping-Symbols.html
            if matches!(name.as_str(), "$d" | "$x") {
                continue;
            }

            // not sure why a symbol would be empty, but it happens
            if name.as_str().is_empty() {
                continue;
            }

            // find the header it is in
            let offset = &self.offsets.iter().find(|offset| {
                let offset_lower = offset.addr;
                let offset_upper = offset.addr + offset.size;

                let sym_lower = addr;
                let sym_upper = addr + size;

                offset_lower <= sym_lower && sym_upper <= offset_upper
            });
            let offset = if let Some(offset) = offset {
                offset
            } else {
                continue;
            };

            // remove any offset which in the symbol so it's now just realtive to the runtime
            // address
            let sym_from = addr + offset.offset - offset.addr;
            let sym_to = sym_from + size;

            if sym_from < from || sym_to > to {
                continue;
            }

            symbols.push(Symbol {
                name: name.to_string(),
                from: sym_from,
                to: sym_to,
            });
        }

        debug!("Now {} symbols", symbols.len());

        Self { symbols, ..self }
    }

    pub fn from_elf(path: String, elf: &goblin::elf::Elf) -> Self {
        let offsets = Self::get_offsets(elf);

        let empty = Self {
            offsets,
            symbols: vec![],
            fallback: None,
            binary_path: path,
        };

        empty.extend_with_debug(elf, u64::MIN, u64::MAX)
    }

    /// offset based on where the binary is loaded
    pub fn add_offset(self, base: u64) -> Self {
        let symbols = self
            .symbols
            .into_iter()
            .map(|Symbol { name, from, to }| Symbol {
                name,
                from: from + base,
                to: to + base,
            })
            .collect();

        Self { symbols, ..self }
    }

    pub fn lookup(&self, adr: u64) -> Option<SymbolReference> {
        self.symbols
            .iter()
            .find(|s| s.from <= adr && adr <= s.to)
            .map(|s| SymbolReference {
                offset: adr - s.from,
                symbol: s,
            })
            .or_else(|| self.fallback.as_ref().and_then(|f| f.lookup(adr)))
    }

    /// Will traverse through self's fallbacks until it comes to the end, it will then add other as
    /// a fallback to that one
    pub fn push_table(&mut self, other: Self) {
        if let Some(fallback) = self.fallback.as_mut() {
            fallback.push_table(other);
        } else {
            self.fallback = Some(Box::new(other));
        }
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
            offsets: vec![],
            binary_path: "/test/file".to_string(),
            fallback: None,
        };

        // do the pie offset
        let pie_table = table.add_offset(0x40_000);

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
