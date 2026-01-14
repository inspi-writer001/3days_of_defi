use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct Config {
    pub authority: Option<Pubkey>,
    // pub seed: u8,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub locked: bool,
    pub config_bump: u8,
    pub lp_bump: u8,
}
