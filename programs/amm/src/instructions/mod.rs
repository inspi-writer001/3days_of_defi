pub mod deposit;
pub mod initialize;

pub use deposit::*;
pub use initialize::*;

pub const CONFIG: &[u8] = b"config";
pub const LP: &[u8] = b"lp";
