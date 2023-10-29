use crate::analyzer::Analysis;
use crate::dis::regs::Reg;
use crate::{
    arch::Arch,
    state::{State, Step},
};
use itertools::Itertools;
use serde_json::json;
use std::fmt;
use std::net::{TcpListener, TcpStream};
use tungstenite::{accept, WebSocket};

pub fn ws<STEP, const N: usize>(analysis: Analysis<STEP, N>, arch: Arch)
where
    STEP: Step<N> + fmt::Debug + std::marker::Sync,
{
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    std::thread::scope(|s| {
        for stream in server.incoming() {
            if let Ok(stream) = stream {
                if let Ok(ws) = accept(stream) {
                    s.spawn(|| handle(ws, &analysis, arch));
                }
            }
        }
    });
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
                // show current values
                let step = trace.get(idx as usize).unwrap();
                let cur_regs = step.state().regs();

                // with markings based on what happen from the PREV step
                let insn = insns.get(idx as usize);
                let mut modifiers = vec![String::new(); cur_regs.len()];

                for r in insn.map(|i| i.read.iter()).unwrap_or_default() {
                    let idx = if let Some(idx) = r.canonical().idx() {
                        idx
                    } else {
                        continue;
                    };

                    modifiers[idx].push('r');
                }

                for r in insn.map(|i| i.write.iter()).unwrap_or_default() {
                    let idx = if let Some(idx) = r.canonical().idx() {
                        idx
                    } else {
                        continue;
                    };

                    modifiers[idx].push('w');
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

                let serialized =
                    serde_json::to_string(&json!({"registers": {"idx": idx, "registers": pairs}}))
                        .unwrap();

                ws.send(tungstenite::Message::Text(serialized)).unwrap();
            }
        }
    }
}
