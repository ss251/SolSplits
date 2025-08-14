use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};
use crate::state::{SplitArrangement, SplitStatus};
use crate::errors::SolSplitsError;

#[derive(Accounts)]
pub struct ExecuteDistribution<'info> {
    #[account(
        mut,
        seeds = [b"split", split_arrangement.creator.as_ref(), &split_arrangement.split_id.to_le_bytes()],
        bump,
        constraint = split_arrangement.status == SplitStatus::Funded @ SolSplitsError::InvalidSplitStatus
    )]
    pub split_arrangement: Account<'info, SplitArrangement>,
    
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = split_arrangement,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = executor.key() == split_arrangement.creator @ crate::errors::SolSplitsError::UnauthorizedUser
    )]
    pub executor: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> ExecuteDistribution<'info> {
    pub fn transfer_to_participant(
        &self,
        amount: u64,
        participant_account: &AccountInfo<'info>,
        seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let cpi_accounts = anchor_spl::token::Transfer {
            from: self.escrow_token_account.to_account_info(),
            to: participant_account.to_account_info(),
            authority: self.split_arrangement.to_account_info(),
        };
        
        let cpi_program = self.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
        
        token::transfer(cpi_ctx, amount)
    }
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteDistribution<'info>>) -> Result<()> {
    // Validate split status
    require!(
        ctx.accounts.split_arrangement.status == SplitStatus::Funded,
        SolSplitsError::InvalidSplitStatus
    );
    
    // Validate escrow balance
    require!(
        ctx.accounts.escrow_token_account.amount >= ctx.accounts.split_arrangement.total_amount,
        SolSplitsError::InsufficientFunds
    );
    
    // Validate remaining accounts
    require!(
        ctx.remaining_accounts.len() == ctx.accounts.split_arrangement.participants.len(),
        SolSplitsError::InvalidParticipant
    );
    
    // Clone necessary data
    let creator_key = ctx.accounts.split_arrangement.creator;
    let split_id = ctx.accounts.split_arrangement.split_id;
    let participants = ctx.accounts.split_arrangement.participants.clone();
    let percentages = ctx.accounts.split_arrangement.percentages.clone();
    let total_amount = ctx.accounts.split_arrangement.total_amount;
    let token_mint = ctx.accounts.split_arrangement.token_mint;
    
    // Process distributions
    for (i, participant) in participants.iter().enumerate() {
        let percentage = percentages[i];
        let amount = (total_amount as u128)
            .checked_mul(percentage as u128)
            .ok_or(SolSplitsError::ArithmeticOverflow)?
            .checked_div(10000)
            .ok_or(SolSplitsError::ArithmeticOverflow)? as u64;
        
        if amount == 0 {
            continue;
        }
        
        let participant_token_account = &ctx.remaining_accounts[i];
        
        // Validate participant token account
        let expected_owner = anchor_spl::associated_token::get_associated_token_address(
            participant,
            &token_mint,
        );
        
        require!(
            participant_token_account.key() == expected_owner,
            SolSplitsError::InvalidParticipant
        );
        
        // Prepare seeds for PDA signing
        let split_id_bytes = split_id.to_le_bytes();
        let seeds = &[
            b"split".as_ref(),
            creator_key.as_ref(),
            &split_id_bytes,
            &[ctx.bumps.split_arrangement],
        ];
        
        // Transfer tokens to participant
        ctx.accounts.transfer_to_participant(
            amount,
            participant_token_account,
            &[&seeds[..]],
        )?;
        
        msg!(
            "Distributed {} tokens to participant {}",
            amount,
            participant
        );
    }
    
    // Update split status
    ctx.accounts.split_arrangement.status = SplitStatus::Completed;
    ctx.accounts.split_arrangement.distributed_at = Some(Clock::get()?.unix_timestamp);
    
    msg!(
        "Split {} distribution completed",
        ctx.accounts.split_arrangement.split_id
    );
    
    Ok(())
}