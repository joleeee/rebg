use crate::analyzer::Analysis;
use crate::dis::regs::Reg;
use crate::state::MemoryOpKind;
use crate::{
    arch::Arch,
    state::{State, Step},
};
use itertools::Itertools;
use serde_json::json;
use std::fmt;
use std::net::{TcpListener, TcpStream};
use tracing::info;
use tungstenite::{accept, WebSocket};

pub fn ws<STEP, const N: usize>(analysis: Analysis<STEP, N>, arch: Arch)
where
    STEP: Step<N> + fmt::Debug + std::marker::Sync,
{
    info!("Execution done, starting WS server.");
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    std::thread::scope(|s| {
        for stream in server.incoming() {
            match stream.map(accept) {
                Ok(Ok(ws)) => {
                    s.spawn(|| handle(ws, &analysis, arch));
                }
                e => {
                    info!("WS failed: {:?}", e);
                }
            }
        }
    });
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum RebgRequest {
    Registers(u64),
}

fn handle<STEP, const N: usize>(
    mut ws: WebSocket<TcpStream>,
    analysis: &Analysis<STEP, N>,
    arch: Arch,
) where
    STEP: Step<N> + fmt::Debug,
{
    let Analysis {
        trace,
        insns,
        instrumentations,
        bt_lens,
        table,
    } = analysis;

    // first send all addresses etc
    let iter = trace
        .iter()
        .enumerate()
        .zip(instrumentations.iter())
        .zip(bt_lens);

    let chunked = iter.chunks(100);

    for chunk in &chunked {
        let mut parts = Vec::new();

        for (((i, step), instru), bt_len) in chunk {
            let symbolized = table
                .lookup(step.state().pc())
                .map(|sy| sy.to_string())
                .unwrap_or("".to_string());

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
            tungstenite::Message::Ping(data) => {
                ws.send(tungstenite::Message::Pong(data)).unwrap();
                continue;
            }
            tungstenite::Message::Close(frame) => {
                info!("Closing: {:?}", frame);
                break;
            }
            _ => continue,
        };

        let msg: RebgRequest = serde_json::from_str(&msg).unwrap();
        match msg {
            RebgRequest::Registers(idx) => {
                // show current values
                let step = trace.get(idx as usize).unwrap();
                let cur_regs = step.state().regs();

                // with markings based on what happen from the PREV step
                let insn = insns.get(idx as usize);
                let mut modifiers = vec![String::new(); cur_regs.len()];

                if let Some(insn) = insn {
                    for idx in insn.read.iter().flat_map(|r| r.canonical().idx()) {
                        modifiers[idx].push('r');
                    }

                    for idx in insn.write.iter().flat_map(|r| r.canonical().idx()) {
                        modifiers[idx].push('w');
                    }
                }

                let pairs: Vec<_> = cur_regs
                    .iter()
                    .zip(modifiers)
                    .enumerate()
                    .map(|(idx, (value, modifier))| {
                        let name = Reg::from_idx(arch, idx).unwrap().as_str();
                        (name, value, modifier)
                    })
                    .collect();

                let (mem_reads, mem_writes) = {
                    let mut reads = Vec::new();
                    let mut writes = Vec::new();

                    for op in step.memory_ops() {
                        let deserialized = op.value.as_u64();

                        match op.kind {
                            MemoryOpKind::Read => reads.push((op.address, deserialized)),
                            MemoryOpKind::Write => writes.push((op.address, deserialized)),
                        }
                    }

                    (reads, writes)
                };

                let serialized =
                    serde_json::to_string(&json!({"registers": {"idx": idx, "registers": pairs}, "mem_ops": {"r": mem_reads, "w": mem_writes}}))
                        .unwrap();

                ws.send(tungstenite::Message::Text(serialized)).unwrap();
            }
        }
    }
}
