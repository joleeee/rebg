use hex::FromHex;
use std::fmt::{Debug, LowerHex};

use crate::{CpuState, Step};

pub trait Code: Clone + Debug + hex::FromHex + std::fmt::LowerHex {
    fn be_bytes(&self) -> &[u8];
}

#[derive(Clone, Debug)]
pub struct FourBytes([u8; 4]);
impl Code for FourBytes {
    fn be_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl FromHex for FourBytes {
    type Error = hex::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut bytes = [0; 4];
        hex::decode_to_slice(hex, &mut bytes)?;
        Ok(FourBytes(bytes))
    }
}

impl LowerHex for FourBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

pub type ARM64Step = Step<u64, FourBytes, ARM64State>;
pub type ARM64State = CpuState<u64, 32>;

// TODO use smallvec
#[derive(Clone, Debug)]
pub struct VarBytes(Vec<u8>);
impl Code for VarBytes {
    fn be_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl FromHex for VarBytes {
    type Error = hex::FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        Ok(VarBytes(hex::decode(hex)?))
    }
}

impl LowerHex for VarBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

pub type X64Step = Step<u64, VarBytes, X64State>;
pub type X64State = CpuState<u64, 16>;