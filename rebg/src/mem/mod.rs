//!

// we have two implementations:

pub mod qword;
// pub use qword::{HistMem, MCell};

pub mod byte;
pub use byte::{HistMem, MCell};
