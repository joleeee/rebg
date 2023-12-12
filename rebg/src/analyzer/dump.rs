use crate::analyzer::Analysis;
use crate::binary::Binary;
use crate::dis::{self, Dis, Instruction};
use crate::mem::HistMem;
use crate::state::{Branching, Instrument, MemoryValue};
use crate::{
    arch::Arch,
    host::Host,
    rstate,
    state::{Instrumentation, MemoryOp, MemoryOpKind, State, Step},
    syms::SymbolTable,
    tracer::ParsedStep,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::cell::RefCell;
use std::rc::Rc;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tracing::{debug, trace as trace_log, warn};

/// Dumps the log
pub struct TraceDumper {
    pub print: bool,
}

impl TraceDumper {
    pub fn analyze<STEP, LAUNCHER, TRACER, ITER, const N: usize>(
        &self,
        // to read files
        launcher: &LAUNCHER,
        // inferred from TRACER
        mut iter: ITER,
        arch: Arch,
    ) -> Analysis<STEP, N>
    where
        STEP: Step<N> + std::fmt::Debug,
        // for inferance
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: std::fmt::Debug,
        TRACER: crate::tracer::Tracer<STEP, N, ITER = ITER>,
        ITER: Iterator<Item = ParsedStep<STEP, N>>,
    {
        let cs = Rc::new(arch.make_capstone().unwrap());
        let dis = Dis { cs, arch };

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
        for (path, pie) in offsets {
            let binary = Binary::from_path(launcher, &PathBuf::from(path.clone())).unwrap();

            let mut table = SymbolTable::from_elf(path.clone(), binary.elf());

            if binary.elf().syms.is_empty() {
                let debug_binary = binary
                    .build_id()
                    .and_then(|id| Binary::try_from_buildid(launcher, &id, arch));

                if let Some(debug_binary) = debug_binary {
                    table = table.extend_with_debug(debug_binary.elf(), 0, pie.1 - pie.0);
                }
            }

            table = table.add_offset(pie.0);

            symbol_tables.push(table);
        }
        // merge into a single table
        let table = symbol_tables
            .into_iter()
            .reduce(|mut accum, item| {
                accum.push_table(item);
                accum
            })
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

        let table = Rc::new(RefCell::new(table));
        // analyzer will insert new symbols into table
        let mut analyzer = RealAnalyzer::new(dis, arch, table.clone(), self.print);

        // we want changes to instantly show up in the UI, but we are also
        // dependent on the next step for some analysis, so we need to first
        // send the raw step, then later send the analyzed step

        let mut instrumentations = Vec::new();
        let mut insns = Vec::new();

        let mut bt = Vec::new();
        let mut bt_lens = Vec::new();

        let mut mem = HistMem::new();

        for (tick, cur_step) in trace.iter().enumerate() {
            let (insn, instrumentation) = analyzer.step(launcher, cur_step);
            instrumentations.push(instrumentation);
            insns.push(insn);
            let prev_instrumentation = instrumentations.iter().rev().nth(1);

            // apply memory operations
            for op in cur_step
                .memory_ops()
                .iter()
                .filter(|m| matches!(m.kind, MemoryOpKind::Write))
            {
                match op.value {
                    MemoryValue::Byte(b) => mem.store8(tick as u32, op.address, b),
                    MemoryValue::Word(w) => mem.store16(tick as u32, op.address, w),
                    MemoryValue::Dword(d) => mem.store32(tick as u32, op.address, d),
                    MemoryValue::Qword(q) => mem.store64(tick as u32, op.address, q),
                }
                .unwrap();
            }

            // do for the PREVIOUS branch
            match prev_instrumentation {
                Some(Instrumentation {
                    branch: Some(prev_branch),
                    disassembly: _,
                }) => match prev_branch {
                    Branching::Call(target, return_address) => {
                        // 1. if we are at target, it's a normal call

                        // 2. if we are at the next address, it means nothing of it was traced

                        // 3. otherwise, i think it's our code -> invisible code -> our code
                        // so we should still do depth += 1 (or actually more?)

                        let is_invisible = cur_step.state().pc() == *return_address;
                        if is_invisible {
                            debug!(">>> INVISIBLE");
                        } else {
                            bt.push(*return_address);

                            let sym_txt = {
                                let syms = analyzer.syms.borrow();
                                let sym = syms.lookup(*target);
                                if let Some(sym) = sym {
                                    format!(" = <{}>", sym)
                                } else {
                                    String::new()
                                }
                            };
                            debug!(">>> {:3} Calling {:x}{}", bt.len(), target, sym_txt);
                        }
                    }
                    Branching::Return => {
                        // find where in the backtrace we are
                        let idx = bt.iter().position(|v| *v == cur_step.state().pc());

                        if let Some(idx) = idx {
                            let removed: Vec<_> = bt.drain(idx..).collect();
                            trace_log!(
                                ">>> {:3} RETURN: removing {} elements",
                                bt.len(),
                                removed.len()
                            );
                        } else {
                            trace_log!(">>> WARNING RETURN: could not find in backtrace!");
                        }
                    }
                },
                _ => {
                    // even if WERE not at a return, we might HAVE actually returned
                    // because the return was not visible due to qemu shit

                    let idx = bt.iter().position(|v| *v == cur_step.state().pc());

                    if let Some(idx) = idx {
                        // TODO also make sure sp changed, as a measure to reduce false positives
                        drop(bt.drain(idx..));
                    }
                }
            }

            bt_lens.push(bt.len());
        }

        // last instruction can be a RET now that we allow tracing only main part of program.
        // assert_eq!(instrumentations.last().and_then(|x| x.branch.clone()), None);

        if !result.status.success() {
            println!("Failed with code: {}", result.status);
        }
        if !result.stdout.is_empty() {
            println!("stdout:\n{}", String::from_utf8(result.stdout).unwrap());
        }
        if !result.stderr.is_empty() {
            println!("stderr:\n{}", String::from_utf8(result.stderr).unwrap());
        }

        assert_eq!(trace.len(), instrumentations.len());
        assert_eq!(trace.len(), bt_lens.len());

        let table = table.borrow();
        let table = table.clone();

        Analysis {
            trace,
            insns,
            instrumentations,
            bt_lens,
            table,
        }
    }
}

fn decompose_syscall(strace: &str) -> Option<(String, Vec<String>, String)> {
    lazy_static! {
        // kinda low effort, will fail if there is a string argument with a comma
        static ref RE: Regex = Regex::new(r"(\w+)\(([^)]*)\) = (\w+)").unwrap();
    }

    let captures = RE.captures(strace);

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
    let arguments = arguments.split(',').map(|x| x.to_string()).collect();

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
    // todo, actually use this
    Munmap {
        #[allow(dead_code)]
        addr: u64,
        #[allow(dead_code)]
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
        debug!("parsed: {}({}) -> {}", name, combined, ret);

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
                    debug!("mmap {} {} {} {}", fd, path, offset, len);

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

struct RealAnalyzer<STEP, const N: usize>
where
    STEP: Step<N>,
{
    hist: Vec<STEP::STATE>,
    dis: Dis,
    syms: Rc<RefCell<SymbolTable>>,
    arch: Arch,
    syscall_state: SyscallState,
    print: bool,
}

impl<STEP, const N: usize> RealAnalyzer<STEP, N>
where
    STEP: Step<N>,
{
    fn new(dis: Dis, arch: Arch, syms: Rc<RefCell<SymbolTable>>, print: bool) -> Self {
        Self {
            hist: Vec::new(),
            dis,
            syms,
            arch,
            syscall_state: SyscallState::new(),
            print,
        }
    }

    fn step<LAUNCHER>(&mut self, launcher: &LAUNCHER, step: &STEP) -> (Instruction, Instrumentation)
    where
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: std::fmt::Debug,
    {
        if let Some(previous) = self.hist.last() {
            let current = step.state();

            if self.print {
                let diff = rstate::diff(previous, current);
                diff.print::<STEP::STATE>(self.arch);
            }
        }

        let address = step.address();
        let code = step.code();

        let insn = self.dis.disassemble_one(code, address).unwrap();
        let op = inst_to_str(&insn, Some(&self.syms.borrow()));

        let syms = self.syms.borrow();
        let symbol = syms.lookup(address);

        let location = if let Some(ref symbol) = symbol {
            let symbol = format!("<{}>", symbol);
            format!("{:>18}", symbol)
        } else {
            format!("0x{:016x}", address)
        };

        drop(syms);

        if self.print {
            println!("{}: {}", location, op);
        }
        // TODO: for some reason the pc is not always the same as the address, especially after cbnz, bl, etc, but also str...
        // EDIT: it seems like it happens when branching to somewhere doing a syscall. it results in two regs| messages, and the last one is the one that "counts"..., i guess where it jump to after the syscall is done or something...?
        assert_eq!(address, step.state().pc());

        if let Some(strace) = step.strace() {
            if self.print {
                println!("syscall: {}", strace);
            }

            let update = self.syscall_state.register(strace);
            match update {
                Ok(Some(StateUpdate::Mmap {
                    path,
                    addr,
                    offset,
                    size,
                })) => {
                    let binary = Binary::from_path(launcher, Path::new(&path));

                    if let Ok(binary) = binary {
                        let mut new_symbol_table = SymbolTable::from_elf(path, binary.elf());

                        if binary.elf().syms.is_empty() {
                            debug!("No symbols, trying to read debug symbols elsewhere. we have {} offsets", new_symbol_table.offsets.len());

                            let buildid = binary.build_id();

                            if let Some(buildid) = buildid {
                                let other_bin =
                                    Binary::try_from_buildid(launcher, &buildid, self.arch);

                                if let Some(other_bin) = other_bin {
                                    new_symbol_table = new_symbol_table.extend_with_debug(
                                        other_bin.elf(),
                                        offset,
                                        offset + size,
                                    );
                                }
                            }
                        }

                        // TODO size
                        new_symbol_table = new_symbol_table.add_offset(addr);

                        self.syms.borrow_mut().push_table(new_symbol_table);
                    }
                }
                Ok(Some(StateUpdate::Munmap { addr: _, size: _ })) => {
                    // TODO remove the symbols
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("Error decoding syscall: {:?}", e);
                }
            }
        }

        let instrum = step.instrument();
        let branch = instrum.recover_branch(&self.dis.cs, &insn);

        if self.print {
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
        }

        self.hist.push(step.state().clone());

        (
            insn,
            Instrumentation {
                branch,
                disassembly: op,
            },
        )
    }
}

fn inst_to_str(insn: &dis::Instruction, table: Option<&SymbolTable>) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(.*)0x([0-9a-fA-F]*)(.*)"#).unwrap();
    }

    let mn = insn.mnemonic.as_ref().unwrap();
    let op = insn.op_str.as_ref().unwrap();

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
