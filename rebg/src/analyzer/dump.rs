use super::Analyzer;
use crate::{
    arch::Arch,
    backend::ParsedStep,
    host::Host,
    rstate,
    state::{MemoryOp, MemoryOpKind, State, Step},
    syms::SymbolTable,
};
use capstone::Capstone;
use goblin::elf::Elf;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: std::fmt::Debug,
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
            let table = SymbolTable::from_elf(&elf).add_offset(pie.0);

            // TODO also add debug symbols if they are missing from the binary itself

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

        print_trace(&trace, launcher, &cs, Some(table.clone()), arch);

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

fn decompose_syscall(strace: &str) -> Option<(String, Vec<String>, String)> {
    lazy_static! {
        // kinda low effort, will fail if there is a string argument with a comma
        static ref RE: Regex = Regex::new(r#"(\w+)\(([^)]*)\) = (\w+)"#).unwrap();
    }

    let captures = RE.captures(&strace);

    let captures = captures.map(|c| {
        c.iter()
            .skip(1)
            .flatten()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
    })?;

    let mut captures = captures.into_iter();

    let name = captures.next().expect("no name");

    let arguments = captures.next().expect("no arguments group");
    let arguments = arguments.split(",").map(|x| x.to_string()).collect();

    let ret = captures.next().expect("no return value").to_string();

    Some((name.to_string(), arguments, ret))
}

#[derive(Debug, Clone, thiserror::Error)]
enum SyscallError {
    #[error("bad format")]
    BadFormat,
    #[error("missing hex prefix")]
    MissingHexPrefix,
    #[error("unqoted string literal")]
    UnqotedStringLiteral,
    #[error("unknown fd")]
    UnknownFd,
    #[error("parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
}

struct SyscallState {
    fds: HashMap<i32, String>,
}

enum StateUpdate {
    Mmap {
        path: String,
        addr: u64,
        offset: u64,
        size: u64,
    },
    Munmap {
        addr: u64,
        size: u64,
    },
}

impl SyscallState {
    fn new() -> Self {
        Self {
            fds: HashMap::new(),
        }
    }

    fn register(&mut self, raw: &str) -> Result<Option<StateUpdate>, SyscallError> {
        let (name, args, ret) = decompose_syscall(raw).ok_or(SyscallError::BadFormat)?;

        let combined = args.join(", ");
        println!("parsed: {}({}) -> {}", name, combined, ret);

        match name.as_str() {
            "openat" => {
                let fd = ret.parse::<i32>()?;
                let file = args[1]
                    .strip_prefix('"')
                    .ok_or(SyscallError::UnqotedStringLiteral)?
                    .strip_suffix('"')
                    .ok_or(SyscallError::UnqotedStringLiteral)?
                    .to_string();
                self.fds.insert(fd, file);
                Ok(None)
            }
            "close" => {
                let fd = args[0].parse::<i32>()?;
                let err = ret.parse::<i32>()?;
                if err == 0 {
                    self.fds.remove(&fd);
                }
                Ok(None)
            }
            "mmap" => {
                let len = args[1].parse::<u64>()?;
                let fd = args[4].parse::<i32>()?;
                let addr = u64::from_str_radix(
                    ret.strip_prefix("0x")
                        .ok_or(SyscallError::MissingHexPrefix)?,
                    16,
                )?;
                let offset = if args[5] == "0" {
                    0
                } else {
                    u64::from_str_radix(
                        args[5]
                            .strip_prefix("0x")
                            .ok_or(SyscallError::MissingHexPrefix)?,
                        16,
                    )?
                };

                if fd != -1 {
                    let path = self
                        .fds
                        .get(&fd)
                        .ok_or(SyscallError::UnknownFd)?
                        .to_string();
                    println!("mmap {} {} {} {}", fd, path, offset, len);

                    Ok(Some(StateUpdate::Mmap {
                        path,
                        addr,
                        offset,
                        size: len,
                    }))
                } else {
                    Ok(None)
                }
            }
            "munmap" => {
                let addr = if args[0] == "NULL" {
                    0
                } else {
                    u64::from_str_radix(
                        args[0]
                            .strip_prefix("0x")
                            .ok_or(SyscallError::MissingHexPrefix)?,
                        16,
                    )?
                };
                let len = args[1].parse::<u64>().unwrap();

                Ok(Some(StateUpdate::Munmap { addr, size: len }))
            }
            _ => Ok(None),
        }
    }
}

fn find_debug_elf<'a, LAUNCHER>(launcher: &LAUNCHER, buildid: &str, arch: Arch) -> Option<Vec<u8>>
where
    LAUNCHER: Host,
    <LAUNCHER as Host>::Error: std::fmt::Debug,
{
    let prefix = &buildid[..2];
    let suffix = &buildid[2..];

    for platform in [
        "/usr/lib/debug/.build-id",
        "/usr/x86_64-linux-gnu/lib/debug/.build-id",
        "/usr/aarch64-linux-gnu/lib/debug/.build-id",
    ] {
        let debug_sym_path = format!("{platform}/{prefix}/{suffix}.debug",);

        println!("Trying {}", debug_sym_path);

        let debug_sym = match launcher.read_file(&PathBuf::from(&debug_sym_path)) {
            Ok(x) => x,
            Err(e) => {
                println!("nope {:?}", e);
                continue;
            }
        };

        let elf = match Elf::parse(&debug_sym) {
            Ok(x) => x,
            Err(e) => {
                println!("cant read elf {:?}", e);
                continue;
            }
        };

        let dbg_arch = match Arch::from_elf(elf.header.e_machine) {
            Ok(x) => x,
            Err(e) => {
                println!("cant read arch {:?}", e);
                continue;
            }
        };

        if dbg_arch != arch {
            println!("wrong arch {:?}", dbg_arch);
            continue;
        }

        println!(
            "Found {} {:?} with {}",
            debug_sym_path,
            dbg_arch,
            elf.syms.len()
        );

        return Some(debug_sym);
    }

    None
}

fn print_trace<STEP, LAUNCHER, const N: usize>(
    trace: &[STEP],
    launcher: &LAUNCHER,
    cs: &Capstone,
    mut syms: Option<SymbolTable>,
    arch: Arch,
) where
    <STEP as Step<N>>::STATE: State<N>,
    STEP: Step<N>,
    LAUNCHER: Host,
    <LAUNCHER as Host>::Error: std::fmt::Debug,
{
    let mut previous_state: Option<STEP::STATE> = None;

    let mut syscall_state = SyscallState::new();

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
        let op = inst_to_str(disasm.first().unwrap(), syms.as_ref());

        let symbol = syms.as_ref().and_then(|s| s.lookup(address));

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

            let update = syscall_state.register(strace);
            match update {
                Ok(Some(StateUpdate::Mmap {
                    path,
                    addr,
                    offset,
                    size,
                })) => {
                    //if offset != 0 {
                    //    panic!("offset not implemented. mmap called with {}, for path {}", offset, path);
                    //}

                    let contents = launcher.read_file(Path::new(&path)).unwrap();

                    //if size < contents.len() as u64 {
                    //    panic!("cutting binary not implemented. binary is {}, but mmap called with {}, for path {}", contents.len(), size, path);
                    //}

                    let elf = Elf::parse(&contents);

                    println!("MEMMMM");

                    if let Ok(elf) = elf {
                        let mut new_symbol_table = SymbolTable::from_elf(&elf);

                        if elf.syms.is_empty() {
                            eprintln!("No symbols, trying to read debug symbols elsewhere. we have {} offsets", new_symbol_table.offsets.len());

                            // .note.gnu.build-id
                            let buildid = elf
                                .section_headers
                                .iter()
                                .find(|s| {
                                    elf.shdr_strtab.get_at(s.sh_name) == Some(".note.gnu.build-id")
                                })
                                .unwrap();

                            let buildid = {
                                let id = &contents[buildid.file_range().unwrap()];
                                // only use the last 20 bytes!!
                                let id = &id[id.len() - 20..];
                                hex::encode(id)
                            };

                            let bin = find_debug_elf(launcher, &buildid, arch);
                            if let Some(bin) = bin {
                                let bin = Elf::parse(&bin).ok();
                                if let Some(bin) = bin {
                                    new_symbol_table = new_symbol_table.extend_with_debug(
                                        &bin,
                                        offset,
                                        offset + size,
                                    );
                                }
                            }
                        }

                        // TODO size
                        new_symbol_table = new_symbol_table.add_offset(addr);

                        syms = if let Some(inner) = syms {
                            Some(inner.join(new_symbol_table))
                        } else {
                            Some(new_symbol_table)
                        };
                    }
                }
                Ok(Some(StateUpdate::Munmap { addr: _, size: _ })) => {
                    // TODO remove the symbols
                }
                Ok(None) => {}
                Err(e) => {
                    println!("Error decoding syscall: {:?}", e);
                }
            }
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
