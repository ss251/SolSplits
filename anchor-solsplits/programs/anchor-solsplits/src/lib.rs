use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("4FEbQAQQiqoU6Xe3Daz54Dj6QDkFG6Yy6MioSziQvTCZ");

#[program]
pub mod anchor_solsplits {
    use super::*;

    pub fn initialize_user(
        ctx: Context<InitializeUser>,
        social_handle_hash: [u8; 32],
    ) -> Result<()> {
        instructions::initialize_user::handler(ctx, social_handle_hash)
    }

    pub fn create_split_arrangement(
        ctx: Context<CreateSplitArrangement>,
        split_id: u64,
        participants: Vec<Pubkey>,
        percentages: Vec<u16>,
    ) -> Result<()> {
        instructions::create_split_arrangement::handler(ctx, split_id, participants, percentages)
    }

    pub fn fund_split(ctx: Context<FundSplit>, amount: u64) -> Result<()> {
        instructions::fund_split::handler(ctx, amount)
    }

    pub fn execute_distribution<'info>(ctx: Context<'_, '_, '_, 'info, ExecuteDistribution<'info>>) -> Result<()> {
        instructions::execute_distribution::handler(ctx)
    }
}