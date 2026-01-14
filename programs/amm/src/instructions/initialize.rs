use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{Config, CONFIG, LP};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint_x: InterfaceAccount<'info, Mint>, // SOL

    #[account(
        init_if_needed,
        payer = signer,
        mint::authority = signer,
        mint::decimals = 6
    )]
    pub mint_y: InterfaceAccount<'info, Mint>, // Your Token

    #[account(
        init,
        payer = signer,
        mint::authority = config,
        mint::decimals = 6,
        seeds = [LP, config.key().as_ref()],
        bump
    )]
    pub mint_lp_token: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = signer,
        associated_token::authority = config,
        associated_token::mint = mint_x,
    )]
    pub vault_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        associated_token::authority = config,
        associated_token::mint = mint_y,
    )]
    pub vault_y: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        seeds = [CONFIG, mint_lp_token.key().as_ref()],
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
        bump
    )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Initialize<'info> {
    pub fn intialize(&mut self, bumps: InitializeBumps) -> Result<()> {
        self.config.set_inner(Config {
            authority: Some(self.signer.key()),
            mint_x: self.mint_x.key(),
            mint_y: self.mint_y.key(),
            locked: false,
            config_bump: bumps.config,
            lp_bump: bumps.mint_lp_token,
        });
        Ok(())
    }
}
