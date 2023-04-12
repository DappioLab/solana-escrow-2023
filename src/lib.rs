pub mod accounts;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("GGJNxHtBwdQTYaz8yhmjCNy8NU8ayJB5GjYbDLkzSsuF");
/// Checks that the supplied program ID is the correct one for Lottery program
pub fn check_program_account(program_account: &Pubkey) -> ProgramResult {
    if program_account != &id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}
