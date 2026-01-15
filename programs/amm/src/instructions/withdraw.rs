use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn_checked, transfer_checked, BurnChecked, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};
use constant_product_curve::ConstantProduct;

use crate::{error::AmmError, Config, CONFIG, LP};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = signer
    )]
    pub user_token_account_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = signer
    )]
    pub user_token_account_y: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config
    )]
    pub vault_x: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config
    )]
    pub vault_y: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_lp_token,
        associated_token::authority = config,
    )]
    pub lp_mint_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_lp_token,
        associated_token::authority = signer
    )]
    pub user_lp_mint_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [LP, config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp_token: InterfaceAccount<'info, Mint>,
    #[account(
        constraint= mint_x.key() == config.mint_x.key() @AmmError::InvalidMintX
    )]
    pub mint_x: InterfaceAccount<'info, Mint>,
    #[account(
        constraint= mint_y.key() == config.mint_y.key() @AmmError::InvalidMintY
    )]
    pub mint_y: InterfaceAccount<'info, Mint>,

    #[account(
        mut @AmmError::InvalidConfig,
        has_one = mint_x,
        has_one = mint_y,
        seeds = [CONFIG, mint_lp_token.key().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, args: WithdrawArgs) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(args.amount != 0, AmmError::InvalidAmount);
        require!(args.min_x != 0 || args.min_y != 0, AmmError::InvalidAmount);

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp_token.supply,
            args.amount,
            6,
        )
        .map_err(|_| AmmError::InvalidAmount)?; // TODO clean this up properly

        require!(
            args.min_x <= amounts.x && args.min_y <= amounts.y,
            AmmError::SlippageExceeded
        );

        Ok(())
    }

    pub fn withdraw_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint, decimal) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_token_account_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_token_account_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from,
            to,
            authority: self.config.to_account_info(),
            mint,
        };

        let mint_lp_token = self.mint_lp_token.key();

        let seeds = &[
            &CONFIG[..],
            &mint_lp_token.as_ref(),
            &[self.config.config_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer_checked(cpi_ctx, amount, decimal)?;

        Ok(())
    }

    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = BurnChecked {
            mint: self.mint_lp_token.to_account_info(),
            from: self.user_lp_mint_account.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        burn_checked(cpi_context, amount, self.mint_lp_token.decimals)?;

        Ok(())
    }
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct WithdrawArgs {
    amount: u64, // amount of LP tokens
    min_x: u64,  // min amount of token X willing to receive
    min_y: u64,  // min amount of token Y willing to receive
}
