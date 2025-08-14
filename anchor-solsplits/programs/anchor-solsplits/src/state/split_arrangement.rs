use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum SplitStatus {
    Created,
    Funded,
    Completed,
}

#[account]
pub struct SplitArrangement {
    pub creator: Pubkey,                 
    pub split_id: u64,                   
    pub participants: Vec<Pubkey>,       
    pub percentages: Vec<u16>,           
    pub total_amount: u64,               
    pub token_mint: Pubkey,              
    pub status: SplitStatus,             
    pub created_at: i64,                 
    pub funded_at: Option<i64>,          
    pub distributed_at: Option<i64>,     
}

impl SplitArrangement {
    pub const MAX_PARTICIPANTS: usize = 10;
    
    pub fn space(num_participants: usize) -> usize {
        8 + // discriminator
        32 + // creator
        8 + // split_id
        4 + (32 * num_participants) + // participants vec
        4 + (2 * num_participants) + // percentages vec
        8 + // total_amount
        32 + // token_mint
        1 + // status
        8 + // created_at
        1 + 8 + // funded_at option
        1 + 8 // distributed_at option
    }
}