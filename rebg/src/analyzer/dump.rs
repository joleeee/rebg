use super::Analyzer;
use crate::{
    arch::Arch,
    backend::ParsedStep,
    launcher::Launcher,
    rstate,
    state::{MemoryOp, MemoryOpKind, State, Step},
    syms::SymbolTable,
};
use capstone::Capstone;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

/// Dumps the log
pub struct TraceDumper {}

impl Analyzer for TraceDumper {
    fn analyze<STEP, LAUNCHER, BACKEND, ITER, const N: usize>(
        // to read files
        launcher: &LAUNCHER,
        // inferred from BACKEND
        mut iter: ITER,
        arch: Arch,
    ) where
        STEP: Step<N> + std::fmt::Debug,
        // for inferance
        LAUNCHER: Launcher,
        <LAUNCHER as Launcher>::Error: std::fmt::Debug,
        BACKEND: crate::backend::Backend<STEP, N, ITER = ITER>,
        ITER: Iterator<Item = ParsedStep<STEP, N>>,
    {
        let cs = arch.make_capstone().unwrap();

        let offsets = match iter.next().unwrap() {
            ParsedStep::LibLoad(x) => x,
            ParsedStep::TraceStep(s) => panic!("Expected libload: {:#?}", s),
            ParsedStep::Final(f) => {
                let code = f.status.code();
                if code == Some(139) {
                    panic!("Segmentation fault");
                } else {
                    panic!("Expected libload: {:#?}", f);
                }
            }
        };

        // get symbol table from all binaries
        let mut symbol_tables = Vec::new();
        for path in offsets.keys() {
            let contents = launcher.read_file(&PathBuf::from(path)).unwrap();
            let elf = goblin::elf::Elf::parse(&contents).unwrap();

            let pie = offsets.get(path).unwrap();
            let table = SymbolTable::from_elf(elf).add_offset(pie.0);

            symbol_tables.push(table);
        }
        // merge into a single table
        let table = symbol_tables
            .into_iter()
            .reduce(|accum, item| accum.join(item))
            .unwrap();

        let mut trace = Vec::new();
        let result = loop {
            let v = match iter.next() {
                Some(v) => v,
                None => panic!("prematurely closed"),
            };

            match v {
                ParsedStep::LibLoad(_) => panic!("Unexpected libload"),
                ParsedStep::TraceStep(step) => {
                    trace.push(step);
                }
                ParsedStep::Final(f) => {
                    // make sure it's done
                    match iter.next() {
                        None => (),
                        Some(_) => panic!("Got message after final"),
                    }
                    break f;
                }
            }
        };

        print_trace(&trace, &cs, Some(&table));

        if !result.status.success() {
            println!("Failed with code: {}", result.status);
        }
        if !result.stdout.is_empty() {
            println!("stdout:\n{}", String::from_utf8(result.stdout).unwrap());
        }
        if !result.stderr.is_empty() {
            println!("stderr:\n{}", String::from_utf8(result.stderr).unwrap());
        }
    }
}

fn print_trace<STEP, const N: usize>(trace: &[STEP], cs: &Capstone, syms: Option<&SymbolTable>)
where
    <STEP as Step<N>>::STATE: State<N>,
    STEP: Step<N>,
{
    let mut previous_state: Option<STEP::STATE> = None;

    for step in trace {
        if let Some(previous) = previous_state {
            let current = step.state();

            let diff = rstate::diff(&previous, current);
            diff.print::<STEP::STATE>();
            println!();
        }

        let address = step.address();
        let code = step.code();

        let disasm = cs.disasm_all(code, address).unwrap();
        assert_eq!(disasm.len(), 1);
        let op = inst_to_str(disasm.first().unwrap(), syms);

        let symbol = syms.and_then(|s| s.lookup(address));

        let location = if let Some(ref symbol) = symbol {
            let symbol = format!("<{}>", symbol);
            format!("{:>18}", symbol)
        } else {
            format!("0x{:016x}", address)
        };

        println!("{}: {}", location, op);
        // TODO: for some reason the pc is not always the same as the address, especially after cbnz, bl, etc, but also str...
        // EDIT: it seems like it happens when branching to somewhere doing a syscall. it results in two regs| messages, and the last one is the one that "counts"..., i guess where it jump to after the syscall is done or something...?
        assert_eq!(address, step.state().pc());

        if let Some(strace) = step.strace() {
            println!("syscall: {}", strace);
        }

        // only print memory changes if we're in the user binary
        for MemoryOp {
            address,
            kind,
            value,
        } in step.memory_ops()
        {
            let arrow = match kind {
                MemoryOpKind::Read => "->",
                MemoryOpKind::Write => "<-",
            };

            println!("0x{:016x} {} 0x{:x}", address, arrow, value.as_u64());
        }

        previous_state = Some(step.state().clone());
    }

    let bytes = std::mem::size_of_val(&trace[0]) * trace.len();
    eprintln!(
        "Used {}kB of memory for {} steps",
        bytes / 1024,
        trace.len()
    );
}

fn inst_to_str(inst: &capstone::Insn, table: Option<&SymbolTable>) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(.*)0x([0-9a-fA-F]*)(.*)"#).unwrap();
    }

    let mn = inst.mnemonic().unwrap();
    let op = inst.op_str().unwrap();

    let op = match RE.captures(op).zip(table) {
        Some((caps, table)) => {
            let mut caps = caps.iter();

            let _whole = caps.next().unwrap().unwrap().as_str();

            let parts: Vec<_> = caps.map(|x| x.unwrap()).map(|x| x.as_str()).collect();

            let (first, rest) = parts.split_first().unwrap();
            let (last, middle) = rest.split_last().unwrap();

            let mut middle: Vec<_> = middle
                .iter()
                .map(|x| u64::from_str_radix(x, 16).unwrap())
                .map(|x| match table.lookup(x) {
                    Some(sym) => format!("<{}>", sym),
                    None => format!("0x{:x}", x),
                })
                .collect();

            let mut strs = vec![];
            strs.push(first.to_string());
            strs.append(&mut middle);
            strs.push(last.to_string());

            strs.join("")
        }
        None => op.to_string(),
    };

    format!("{} {}", mn, op)
}
