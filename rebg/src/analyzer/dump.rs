use super::Analyzer;
use crate::state::{self, Branching, Instrument};
use crate::{
    arch::Arch,
    host::Host,
    rstate,
    state::{Instrumentation, MemoryOp, MemoryOpKind, State, Step},
    syms::SymbolTable,
    tracer::ParsedStep,
};
use capstone::Capstone;
use goblin::elf::Elf;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::json;
use std::net::TcpListener;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};
use tungstenite::accept;

/// Dumps the log
pub struct TraceDumper {}

impl Analyzer for TraceDumper {
    fn analyze<STEP, LAUNCHER, TRACER, ITER, const N: usize>(
        // to read files
        launcher: &LAUNCHER,
        // inferred from TRACER
        mut iter: ITER,
        arch: Arch,
    ) where
        STEP: Step<N> + std::fmt::Debug,
        // for inferance
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: std::fmt::Debug,
        TRACER: crate::tracer::Tracer<STEP, N, ITER = ITER>,
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

        let mut analyzer = RealAnalyzer::new(Rc::new(cs), arch, table.clone());

        // we want changes to instantly show up in the UI, but we are also
        // dependent on the next step for some analysis, so we need to first
        // send the raw step, then later send the analyzed step

        let mut instrumentations = Vec::new();

        let mut bt = Vec::new();
        let mut bt_lens = Vec::new();

        for cur_step in &trace {
            let instrumentation = analyzer.step(launcher, cur_step);
            instrumentations.push(instrumentation);
            let prev_instrumentation = instrumentations.iter().rev().skip(1).next();

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
                            println!(">>> INVISIBLE");
                        } else {
                            bt.push(*return_address);

                            let sym_txt = {
                                let sym = analyzer.syms.lookup(*target);
                                if let Some(sym) = sym {
                                    format!(" = <{}>", sym)
                                } else {
                                    String::new()
                                }
                            };
                            println!(">>> {:3} Calling {:x}{}", bt.len(), target, sym_txt);
                        }
                    }
                    Branching::Return => {
                        // find where in the backtrace we are
                        let idx = bt.iter().position(|v| *v == cur_step.state().pc());

                        if let Some(idx) = idx {
                            let removed: Vec<_> = bt.drain(idx..).collect();
                            println!(
                                ">>> {:3} RETURN: removing {} elements",
                                bt.len(),
                                removed.len()
                            );
                        } else {
                            println!(">>> WARNING RETURN: could not find in backtrace!");
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

        // should never be the case, but we COULD end up with an unprocessed instrumentation here if the last step added a instrumentation
        assert_eq!(instrumentations.last().and_then(|x| x.branch.clone()), None);

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

        let server = TcpListener::bind("127.0.0.1:9001").unwrap();
        for stream in server.incoming() {
            let trace = trace.clone();
            let instrumentations = instrumentations.clone();
            let bt_lens = bt_lens.clone();
            let table = table.clone();
            std::thread::spawn(move || {
                // first send all addresses etc
                let mut ws = accept(stream.unwrap()).unwrap();

                let iter = trace
                    .iter()
                    .enumerate()
                    .zip(instrumentations.into_iter())
                    .zip(bt_lens.into_iter());
                // .filter(|(((_, tr), _), _)| tr.state().pc() < 0x5500000000);

                let chunked = iter.chunks(100);

                for chunk in &chunked {
                    let mut parts = Vec::new();

                    for (((i, step), instru), bt_len) in chunk {
                        let symbolized = if let Some(s) = table.lookup(step.state().pc()) {
                            format!("{}", s)
                        } else {
                            "".to_string()
                        };
                        parts.push(json!({"i": i, "a": step.state().pc(), "c": instru.disassembly, "d": bt_len, "s": symbolized}));
                    }

                    let json = serde_json::to_string(&json!({"steps": parts})).unwrap();
                    ws.send(tungstenite::Message::Text(json)).unwrap();
                }

                // then send register values on request
                loop {
                    let msg = ws.read().unwrap();

                    let msg = match msg {
                        tungstenite::Message::Text(text) => text,
                        tungstenite::Message::Ping(_p) => {
                            ws.send(tungstenite::Message::Pong(_p)).unwrap();
                            continue;
                        }
                        tungstenite::Message::Close(_c) => {
                            println!("Closing: {:?}", _c);
                            break;
                        }
                        _ => continue,
                    };

                    #[derive(serde::Deserialize, serde::Serialize)]
                    #[serde(rename_all = "snake_case")]
                    enum RebgRequest {
                        Registers(u64),
                    }

                    let msg: RebgRequest = serde_json::from_str(&msg).unwrap();
                    match msg {
                        RebgRequest::Registers(idx) => {
                            let cur_regs = {
                                let step = trace.get(idx as usize).unwrap();
                                step.state().regs()
                            };
                            let prev_regs = {
                                if idx > 0 {
                                    let step = trace.get((idx - 1) as usize).unwrap();
                                    Some(step.state().regs())
                                } else {
                                    None
                                }
                            };

                            let prev_regs = prev_regs.unwrap_or(cur_regs);

                            let regs: Vec<_> = cur_regs
                                .iter()
                                .zip(prev_regs)
                                .map(|(cur, prev)| (cur, if cur == prev { "" } else { "w" }))
                                .collect();

                            let pairs = regs.iter().enumerate().map(|(idx, (value, modifier))| {
                                let name = <STEP as state::Step<N>>::STATE::reg_name(idx as usize);
                                (name, value, modifier)
                            });
                            let pairs: Vec<_> = pairs.collect();

                            let serialized = serde_json::to_string(
                                &json!({"registers": {"idx": idx, "registers": pairs}}),
                            )
                            .unwrap();

                            ws.send(tungstenite::Message::Text(serialized)).unwrap();
                        }
                    }
                }
            });
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

struct RealAnalyzer<STEP, const N: usize>
where
    STEP: Step<N>,
{
    hist: Vec<STEP::STATE>,
    cs: Rc<Capstone>,
    syms: SymbolTable,
    arch: Arch,
    syscall_state: SyscallState,
}

impl<STEP, const N: usize> RealAnalyzer<STEP, N>
where
    STEP: Step<N>,
{
    fn new(cs: Rc<Capstone>, arch: Arch, syms: SymbolTable) -> Self {
        Self {
            hist: Vec::new(),
            cs,
            syms,
            arch,
            syscall_state: SyscallState::new(),
        }
    }

    fn step<LAUNCHER>(&mut self, launcher: &LAUNCHER, step: &STEP) -> Instrumentation
    where
        LAUNCHER: Host,
        <LAUNCHER as Host>::Error: std::fmt::Debug,
    {
        if let Some(previous) = self.hist.last() {
            let current = step.state();

            let diff = rstate::diff(previous, current);
            diff.print::<STEP::STATE>();
            println!();
        }

        let address = step.address();
        let code = step.code();

        let disasm = self.cs.disasm_all(code, address).unwrap();
        assert_eq!(disasm.len(), 1);
        let insn = &disasm[0];
        let op = inst_to_str(&insn, Some(&self.syms));

        let detail = self.cs.insn_detail(insn).expect("no detail");

        let symbol = self.syms.lookup(address);

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

            let update = self.syscall_state.register(strace);
            match update {
                Ok(Some(StateUpdate::Mmap {
                    path,
                    addr,
                    offset,
                    size,
                })) => {
                    let contents = launcher.read_file(Path::new(&path)).unwrap();

                    let elf = Elf::parse(&contents);

                    println!("MEMMMM");

                    if let Ok(elf) = elf {
                        let mut new_symbol_table = SymbolTable::from_elf(&elf);

                        if elf.syms.is_empty() {
                            eprintln!("No symbols, trying to read debug symbols elsewhere. we have {} offsets", new_symbol_table.offsets.len());

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

                            let bin = find_debug_elf(launcher, &buildid, self.arch);
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

                        self.syms = self.syms.clone().join(new_symbol_table);
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

        let instrum = step.instrument();
        let branch = instrum.recover_branch(&self.cs, &insn, &detail);

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

        self.hist.push(step.state().clone());

        Instrumentation {
            branch,
            disassembly: op,
        }
    }
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
