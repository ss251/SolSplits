use anchor_lang::prelude::*;
use crate::state::{UserRegistry, SplitArrangement, SplitStatus};
use crate::errors::SolSplitsError;

#[derive(Accounts)]
#[instruction(split_id: u64, participants: Vec<Pubkey>, percentages: Vec<u16>)]
pub struct CreateSplitArrangement<'info> {
    #[account(
        init,
        payer = creator,
        space = SplitArrangement::space(participants.len()),
        seeds = [b"split", creator.key().as_ref(), &split_id.to_le_bytes()],
        bump
    )]
    pub split_arrangement: Account<'info, SplitArrangement>,
    
    #[account(
        mut,
        seeds = [b"user_registry", creator.key().as_ref()],
        bump,
        constraint = user_registry.wallet_pubkey == creator.key() @ SolSplitsError::UnauthorizedUser
    )]
    pub user_registry: Account<'info, UserRegistry>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateSplitArrangement>,
    split_id: u64,
    participants: Vec<Pubkey>,
    percentages: Vec<u16>,
) -> Result<()> {
    require!(
        participants.len() == percentages.len(),
        SolSplitsError::InvalidParticipant
    );
    
    require!(
        participants.len() <= SplitArrangement::MAX_PARTICIPANTS,
        SolSplitsError::TooManyParticipants
    );
    
    let total_percentage: u16 = percentages.iter().sum();
    require!(
        total_percentage == 10000,
        SolSplitsError::InvalidPercentages
    );
    
    let split_arrangement = &mut ctx.accounts.split_arrangement;
    let user_registry = &mut ctx.accounts.user_registry;
    
    split_arrangement.creator = ctx.accounts.creator.key();
    split_arrangement.split_id = split_id;
    split_arrangement.participants = participants;
    split_arrangement.percentages = percentages;
    split_arrangement.total_amount = 0;
    split_arrangement.token_mint = Pubkey::default();
    split_arrangement.status = SplitStatus::Created;
    split_arrangement.created_at = Clock::get()?.unix_timestamp;
    split_arrangement.funded_at = None;
    split_arrangement.distributed_at = None;
    
    user_registry.total_splits_created = user_registry
        .total_splits_created
        .checked_add(1)
        .ok_or(SolSplitsError::ArithmeticOverflow)?;
    
    msg!(
        "Split arrangement created with ID: {} for {} participants",
        split_id,
        split_arrangement.participants.len()
    );
    
    Ok(())
}