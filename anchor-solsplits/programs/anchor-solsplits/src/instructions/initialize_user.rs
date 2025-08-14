use anchor_lang::prelude::*;
use crate::state::UserRegistry;

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + UserRegistry::INIT_SPACE,
        seeds = [b"user_registry", user.key().as_ref()],
        bump
    )]
    pub user_registry: Account<'info, UserRegistry>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeUser>, social_handle_hash: [u8; 32]) -> Result<()> {
    let user_registry = &mut ctx.accounts.user_registry;
    
    user_registry.social_handle_hash = social_handle_hash;
    user_registry.wallet_pubkey = ctx.accounts.user.key();
    user_registry.verification_status = false;
    user_registry.created_at = Clock::get()?.unix_timestamp;
    user_registry.total_splits_created = 0;
    user_registry.reputation_score = 0;
    
    msg!("User registry initialized for wallet: {}", ctx.accounts.user.key());
    
    Ok(())
}