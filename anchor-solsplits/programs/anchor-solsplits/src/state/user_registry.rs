use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserRegistry {
    pub social_handle_hash: [u8; 32],    
    pub wallet_pubkey: Pubkey,           
    pub verification_status: bool,        
    pub created_at: i64,                 
    pub total_splits_created: u32,       
    pub reputation_score: u16,           
}