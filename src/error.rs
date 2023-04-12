use solana_program::msg;
use solana_program::program_error::ProgramError;
use strum_macros::AsRefStr;
#[derive(Debug, Copy, Clone, AsRefStr)]
pub enum EscrowError {
    ExpectedAmountMismatch,
    InvalidInstructionType,
    InvalidInstructionData,
    InvalidEscrowState,
    InvalidEscrowVault,
    InvalidSigner,
    NotEnoughAccountKeys,
    TooMuchAccountKeys,
    DeserializeTokenAccountError,
    DeserializeMintAccountError,
    DeserializeEscrowAccountError,
    MintAMismatch,
    MintBMismatch,
    VaultKeyMismatch,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl EscrowError {
    pub fn print_into(self) -> ProgramError {
        msg!(format!("Error: {}", self.as_ref()).as_str());
        self.into()
    }
}
