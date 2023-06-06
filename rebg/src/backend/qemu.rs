use anyhow::Context;

use super::{Backend, ParsedStep};
use crate::{arch::Arch, state::Step};
use std::{
    collections::HashMap,
    fmt,
    io::{BufRead, BufReader},
    marker::PhantomData,
    mem,
    path::{Path, PathBuf},
    process::{Child, ChildStderr},
};

pub struct QEMU {}

impl<STEP, const N: usize> Backend<STEP, N> for QEMU
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [String], Error = anyhow::Error>,
{
    type ITER = QEMUParser<STEP, N>;

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
    fn parse(&self, proc: std::process::Child) -> Self::ITER {
        QEMUParser::new(proc)
    }
}

impl QEMU {}

// having the bounds here mean the STATE has to be the same type as the STATE type in QEMU, which
// means less room for error and automatic inference of this type

pub struct QEMUParser<STEP, const N: usize> {
    /// None when done
    proc: Option<Child>,

    stderr: BufReader<ChildStderr>,
    _phantom: PhantomData<STEP>,
}

impl<STEP, const N: usize> Iterator for QEMUParser<STEP, N>
where
    STEP: Step<N> + Send + 'static + fmt::Debug,
    STEP: for<'a> TryFrom<&'a [String], Error = anyhow::Error>,
{
    type Item = ParsedStep<STEP, N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.proc.is_none() {
            return None;
        }

        let mut lines: Vec<String> = vec![];

        loop {
            let done = lines.last().map(|x| x.as_str()) == Some("----------------");

            // if done, send the message
            if done {
                lines.pop(); // remove the -- sep

                if lines[0].starts_with("elflibload") {
                    let e = Self::parse_elflibload(&lines).unwrap();
                    let e = ParsedStep::LibLoad(e);
                    lines.clear();
                    break Some(e);
                } else {
                    let s = STEP::try_from(&lines).unwrap();
                    let s = ParsedStep::TraceStep(s);
                    lines.clear();
                    break Some(s);
                }
            }

            // otherwise, read one more line
            let mut stderr_buf = String::new();
            let result = self.stderr.read_line(&mut stderr_buf).unwrap();

            // EOF
            if result == 0 {
                // this sets self.proc = None, so a None is returned next time
                let mut my_proc = None;
                mem::swap(&mut self.proc, &mut my_proc);
                let my_proc = my_proc.unwrap();

                // make sure it closed gracefully
                let result = my_proc.wait_with_output().unwrap();

                break Some(ParsedStep::Final(result));
            }

            lines.push(
                stderr_buf
                    .strip_suffix('\n')
                    .map(|x| x.to_string())
                    // last line may not have a newline
                    .unwrap_or(stderr_buf),
            );
        }
    }
}

impl<STEP, const N: usize> QEMUParser<STEP, N> {
    fn new(mut proc: Child) -> Self {
        let stderr = proc.stderr.take().unwrap();
        let stderr = BufReader::new(stderr);

        Self {
            proc: Some(proc),
            stderr,
            _phantom: PhantomData,
        }
    }

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
