use core::fmt;
use std::str::FromStr;

use anyhow::Context;
use bitflags::Flags;
use hex::FromHex;
use num_traits::Num;

pub mod aarch64;
pub use aarch64::{Aarch64Flags, Aarch64State, Aarch64Step};
pub mod x64;
pub use x64::{X64Flags, X64State, X64Step};

// This needs to be a trait because different architectures have different instruction sizes
pub trait Step<S: State<N>, const N: usize, FLAGS>: Clone {
    fn code(&self) -> &[u8];
    // this also contains the pc
    fn state(&self) -> &S;
    // sometimes they differ, though, so also keep address
    fn address(&self) -> u64;
}

pub trait State<const N: usize>: Clone {
    type FLAGS: Flags + Clone + Copy + fmt::Debug;
    fn pc(&self) -> u64;
    fn regs(&self) -> &[u64; N];
    fn reg_name(i: usize) -> &'static str;
    fn flags(&self) -> &Self::FLAGS;
}

// nasty shit
// ==========
struct GenericState<TYPE, const N: usize> {
    regs: [TYPE; N],
    pc: TYPE,
    flags: TYPE,
}

impl<TYPE, const N: usize> FromStr for GenericState<TYPE, N>
where
    TYPE: Num + Copy,
    <TYPE as Num>::FromStrRadixErr: fmt::Debug,
{
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<Self> {
        let regs = input
            .split('|')
            .map(|data| data.split_once('='))
            .map(Option::unwrap)
            .map(|(name, value)| (name.trim(), TYPE::from_str_radix(value, 16).unwrap()));

        let mut registers: [Option<TYPE>; N] = [None; N];
        let mut pc = None;
        let mut flags = None;

        for (name, value) in regs {
            match name {
                "pc" => {
                    pc = Some(value);
                }
                "flags" => {
                    flags = Some(value);
                }
                _ => {
                    let index = name.strip_prefix('r').context("missing register prefix")?;
                    let index = usize::from_str_radix(index, 10)?;
                    registers[index] = Some(value);
                }
            }
        }

        let pc = pc.unwrap();
        let flags = flags.unwrap();

        if registers.contains(&None) {
            return Err(anyhow::anyhow!("register not set"));
        }
        let registers = registers.map(Option::unwrap);

        Ok(Self {
            regs: registers,
            pc,
            flags,
        })
    }
}

struct GenericStep<STATE: FromStr> {
    state: STATE,
    code: Vec<u8>,
    address: u64,
}

impl<STATE> FromStr for GenericStep<STATE>
where
    STATE: FromStr<Err = anyhow::Error>,
{
    type Err = anyhow::Error;

    fn from_str(input: &str) -> anyhow::Result<Self> {
        let lines = input.split('\n').filter_map(|x| x.split_once('|'));

        let mut s_state = None;
        let mut s_address = None;
        let mut s_code = None;

        for (what, content) in lines {
            match what {
                "regs" => {
                    s_state = Some(STATE::from_str(content)?);
                }
                "address" => {
                    s_address = Some(
                        u64::from_str_radix(content, 16).map_err(Into::<anyhow::Error>::into)?,
                    );
                }
                "code" => {
                    s_code = Some(Vec::from_hex(content).unwrap());
                }
                _ => panic!("unknown data"),
            }
        }

        let address = s_address.unwrap();
        let code = s_code.unwrap();
        let state = s_state.unwrap();

        Ok(Self {
            state,
            code,
            address,
        })
    }
}
