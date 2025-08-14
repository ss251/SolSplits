use anchor_lang::prelude::*;

#[error_code]
pub enum SolSplitsError {
    #[msg("User is not authorized to perform this action")]
    UnauthorizedUser,
    
    #[msg("Invalid split status for this operation")]
    InvalidSplitStatus,
    
    #[msg("Invalid percentages - must sum to 10000 (100%)")]
    InvalidPercentages,
    
    #[msg("Too many participants in split arrangement")]
    TooManyParticipants,
    
    #[msg("Arithmetic overflow occurred")]
    ArithmeticOverflow,
    
    #[msg("Insufficient funds in escrow")]
    InsufficientFunds,
    
    #[msg("Invalid participant account")]
    InvalidParticipant,
    
    #[msg("Split has already been funded")]
    AlreadyFunded,
    
    #[msg("Split has already been distributed")]
    AlreadyDistributed,
}