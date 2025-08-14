use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer},
};
use crate::state::{SplitArrangement, SplitStatus};
use crate::errors::SolSplitsError;

#[derive(Accounts)]
pub struct FundSplit<'info> {
    #[account(
        mut,
        seeds = [b"split", split_arrangement.creator.as_ref(), &split_arrangement.split_id.to_le_bytes()],
        bump,
        constraint = split_arrangement.status == SplitStatus::Created @ SolSplitsError::InvalidSplitStatus
    )]
    pub split_arrangement: Account<'info, SplitArrangement>,
    
    #[account(
        init_if_needed,
        payer = funder,
        associated_token::mint = token_mint,
        associated_token::authority = split_arrangement,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = funder,
    )]
    pub funder_token_account: Account<'info, TokenAccount>,
    
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = funder.key() == split_arrangement.creator @ crate::errors::SolSplitsError::UnauthorizedUser
    )]
    pub funder: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<FundSplit>, amount: u64) -> Result<()> {
    require!(amount > 0, SolSplitsError::InsufficientFunds);
    
    let split_arrangement = &mut ctx.accounts.split_arrangement;
    
    require!(
        split_arrangement.status == SplitStatus::Created,
        SolSplitsError::AlreadyFunded
    );
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.funder_token_account.to_account_info(),
        to: ctx.accounts.escrow_token_account.to_account_info(),
        authority: ctx.accounts.funder.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );
    
    token::transfer(cpi_ctx, amount)?;
    
    split_arrangement.total_amount = amount;
    split_arrangement.token_mint = ctx.accounts.token_mint.key();
    split_arrangement.status = SplitStatus::Funded;
    split_arrangement.funded_at = Some(Clock::get()?.unix_timestamp);
    
    msg!(
        "Split {} funded with {} tokens",
        split_arrangement.split_id,
        amount
    );
    
    Ok(())
}