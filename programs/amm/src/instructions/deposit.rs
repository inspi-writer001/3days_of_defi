use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
    },
};
use constant_product_curve::ConstantProduct;
use num_integer::Roots;

use crate::{error::AmmError, Config, CONFIG, LP};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_x,
        associated_token::authority = signer
    )]
    pub user_account_x: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_y,
        associated_token::authority = signer
    )]
    pub user_account_y: InterfaceAccount<'info, TokenAccount>,

    #[account(
        constraint= mint_x.key() == config.mint_x.key() @AmmError::InvalidMintX
    )]
    pub mint_x: InterfaceAccount<'info, Mint>,

    #[account(
        constraint= mint_y.key() == config.mint_y.key() @AmmError::InvalidMintY
    )]
    pub mint_y: InterfaceAccount<'info, Mint>,

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
        seeds = [LP, config.key().as_ref()],
        bump = config.lp_bump,
    )]
    pub mint_lp_token: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_lp_token,
        associated_token::authority = signer,
    )]
    pub user_lp_token_account: InterfaceAccount<'info, TokenAccount>,

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

impl<'info> Deposit<'info> {
    /// This deposit works by saying: "I want this specific amount of LP Tokens (`amount`),
    /// and here is the max X (`max_x`) and max Y (`max_y`) I am willing to pay for it."
    ///
    /// 1. CALCULATE LP AMOUNT (How many LP tokens do I get?):
    ///    - If New Pool: `lp_tokens = Math.sqrt(deposit_x * deposit_y)`
    ///    - If Existing Pool: Fetch total_lp_supply from the mint.
    ///      Formula: `lp_tokens = (my_deposit_x / total_vault_x) * total_lp_supply`
    ///
    /// 2. CALCULATE MATCHING Y (How much Y do I need to send?):
    ///    - To maintain the ratio, the math is:
    ///      `required_y = (my_deposit_x / total_vault_x) * total_vault_y`
    ///
    /// 3. INPUTS FOR FUNCTION:
    ///    - amount: The result from Step 1.
    ///    - max_x:  my_deposit_x (plus small slippage buffer)
    ///    - max_y:  required_y (plus small slippage buffer)
    pub fn deposit(&mut self, args: DepositArgs) -> Result<()> {
        require!(self.config.locked == false, AmmError::PoolLocked);
        require!(args.amount != 0, AmmError::InvalidAmount);

        let mut amount_to_mint = args.amount;

        let (x, y) = match self.mint_lp_token.supply == 0
            && self.vault_x.amount == 0
            && self.vault_y.amount == 0
        {
            true => {
                let x = args.max_x;
                let y = args.max_y;

                // Formula: sqrt(x * y) --- Math.sqrt(deposit_x * deposit_y)
                let calculated_initial_lp = (x as u128)
                    .checked_mul(y as u128)
                    .ok_or(AmmError::MathOverflow)?
                    .sqrt();

                amount_to_mint = calculated_initial_lp as u64;

                (x, y)
            }
            false => {
                let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp_token.supply,
                    args.amount,
                    6,
                )
                .unwrap();
                (amounts.x, amounts.y)
            }
        };

        require!(
            x <= args.max_x && y <= args.max_y,
            AmmError::SlippageExceeded
        );

        // Deposit token x
        self.deposit_tokens(true, x)?;
        // deposit token y
        self.deposit_tokens(false, y)?;
        // mint lp tokens
        self.mint_lp_tokens(amount_to_mint)?;

        Ok(())
    }

    pub fn deposit_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint, decimal) = match is_x {
            true => (
                self.user_account_x.to_account_info(),
                self.vault_x.to_account_info(),
                self.mint_x.to_account_info(),
                self.mint_x.decimals,
            ),
            false => (
                self.user_account_y.to_account_info(),
                self.vault_y.to_account_info(),
                self.mint_y.to_account_info(),
                self.mint_y.decimals,
            ),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from,
            to,
            authority: self.signer.to_account_info(),
            mint,
        };

        let ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked(ctx, amount, decimal)
    }

    pub fn mint_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo {
            mint: self.mint_lp_token.to_account_info(),
            to: self.user_lp_token_account.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let mint_lp_token = self.mint_lp_token.key();

        let seeds = &[
            &CONFIG[..],
            &mint_lp_token.as_ref(),
            &[self.config.config_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        mint_to(ctx, amount)
    }
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct DepositArgs {
    amount: u64, // amount of LP tokens
    max_x: u64,  // max amount of token X willing to deposit
    max_y: u64,  // max amount of token Y willing to deposit
}
