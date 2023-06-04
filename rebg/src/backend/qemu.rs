use anyhow::Context;

use super::{Backend, ParsedStep};
use crate::{arch::Arch, state::Step};
use std::{
    collections::HashMap,
    fmt,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    thread,
};

pub struct QEMU {}

impl<STEP, const N: usize> Backend<STEP, N> for QEMU
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [String], Error = anyhow::Error>,
{
    fn command(&self, executable: &Path, arch: Arch) -> (String, Vec<String>) {
        let qemu = arch.qemu_user_bin().to_string();

        let guest_path = format!(
            "/container/{}",
            PathBuf::from(&executable)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );

        let options = vec![
            String::from("-one-insn-per-tb"),
            String::from("-d"),
            String::from("in_asm,strace"),
            guest_path,
        ];

        (qemu, options)
    }

    /// Takes output from the process and parses it to steps
    /// TODO: Use an iterator instead of a new thread
    fn parse(&self, mut proc: std::process::Child) -> flume::Receiver<ParsedStep<STEP, N>> {
        let (tx, rx) = flume::unbounded();

        thread::spawn(move || {
            let stderr = proc.stderr.take().unwrap();
            let mut stderr = BufReader::new(stderr);

            let mut lines: Vec<String> = vec![];

            loop {
                let done = lines.last().map(|x| x.as_str()) == Some("----------------");

                if done {
                    lines.pop(); // remove the -- sep

                    if lines[0].starts_with("elflibload") {
                        let e = Self::parse_elflibload(&lines).unwrap();
                        let e = ParsedStep::LibLoad(e);
                        tx.send(e).unwrap();
                    } else {
                        let s = STEP::try_from(&lines).unwrap();
                        let s = ParsedStep::TraceStep(s);
                        tx.send(s).unwrap();
                    }

                    lines.clear();
                }

                let mut stderr_buf = String::new();
                let result = stderr.read_line(&mut stderr_buf).unwrap();
                if result == 0 {
                    // EOF

                    // now make sure it closed gracefully
                    let result = proc.wait_with_output().unwrap();

                    tx.send(ParsedStep::Final(result)).unwrap();

                    return;
                }

                lines.push(
                    stderr_buf
                        .strip_suffix('\n')
                        .map(|x| x.to_string())
                        // last line may not have a newline
                        .unwrap_or(stderr_buf),
                );
            }
        });

        rx
    }
}

impl QEMU {
    fn parse_elflibload(output: &[String]) -> anyhow::Result<HashMap<String, (u64, u64)>> {
        let parts: Vec<_> = output
            .iter()
            .map(|x| x.split_once('|'))
            .collect::<Option<Vec<_>>>()
            .context("invalid header, should only be | separated key|values")?;

        let mut elfs = HashMap::new();
        for (key, value) in parts {
            match key {
                "elflibload" => {
                    let (path, other) = value.split_once('|').unwrap();
                    let (from, to) = other.split_once('|').unwrap();

                    let from = u64::from_str_radix(from, 16).unwrap();
                    let to = u64::from_str_radix(to, 16).unwrap();

                    elfs.insert(path.to_string(), (from, to));
                }
                _ => {
                    return Err(anyhow::anyhow!("unknown header key: {}", key));
                }
            }
        }

        Ok(elfs)
    }
}
