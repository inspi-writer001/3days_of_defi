use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Invalid Mint X")]
    InvalidMintX,
    #[msg("Invalid Mint Y")]
    InvalidMintY,
    #[msg("Invalid Config")]
    InvalidConfig,
    #[msg("Invalid Amount")]
    InvalidAmount,
    #[msg("Slippage Exceeded")]
    SlippageExceeded,
    #[msg("Pool Locked")]
    PoolLocked,
    #[msg("Math overflow")]
    MathOverflow,
}
