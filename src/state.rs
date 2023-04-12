use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

use crate::error::EscrowError;
use spl_token::state::{Account, Mint};
#[derive(Default)]
pub struct EscrowState {
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub expected_amount: u64,
    pub bump: u8,
    pub seed: u64,
}

impl Sealed for EscrowState {}

impl IsInitialized for EscrowState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for EscrowState {
    const LEN: usize = 114;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, EscrowState::LEN];
        let (is_initialized, initializer_pubkey, mint_a, mint_b, expected_amount, bump, seed) =
            array_refs![src, 1, 32, 32, 32, 8, 1, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(EscrowError::InvalidEscrowState.print_into()),
        };

        Ok(EscrowState {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            mint_a: Pubkey::new_from_array(*mint_a),
            mint_b: Pubkey::new_from_array(*mint_b),
            expected_amount: u64::from_le_bytes(*expected_amount),
            bump: bump[0],
            seed: u64::from_le_bytes(*seed),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, EscrowState::LEN];
        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            mint_a_dst,
            mint_b_dst,
            expected_amount_dst,
            bump_dst,
            seed_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 8, 1, 8];

        let EscrowState {
            is_initialized,
            initializer_pubkey,
            mint_a,
            mint_b,
            expected_amount,
            bump,
            seed,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        mint_a_dst.copy_from_slice(mint_a.as_ref());
        mint_b_dst.copy_from_slice(mint_b.as_ref());
        *expected_amount_dst = expected_amount.to_le_bytes();
        *bump_dst = bump.to_le_bytes();
        *seed_dst = seed.to_le_bytes()
    }
}

pub struct TokenAccount {
    pub key: Pubkey,
    pub info: Account,
    pub program_id: Pubkey,
    pub is_writable: bool,
}
impl TokenAccount {
    pub fn unpack(info: &AccountInfo) -> Result<TokenAccount, EscrowError> {
        Ok(TokenAccount {
            key: *info.key,
            info: match Account::unpack(&info.try_borrow_data().unwrap()) {
                Ok(s) => s,
                Err(_) => return Err(EscrowError::DeserializeTokenAccountError),
            },
            program_id: *info.owner,
            is_writable: info.is_writable,
        })
    }
}
pub struct EscrowAccount {
    pub key: Pubkey,
    pub info: EscrowState,
    pub program_id: Pubkey,
    pub is_writable: bool,
}
impl EscrowAccount {
    pub fn unpack(info: &AccountInfo) -> Result<EscrowAccount, EscrowError> {
        Ok(EscrowAccount {
            key: *info.key,
            info: match EscrowState::unpack(&info.try_borrow_data().unwrap()) {
                Ok(s) => s,
                Err(_) => return Err(EscrowError::DeserializeMintAccountError),
            },
            program_id: *info.owner,
            is_writable: info.is_writable,
        })
    }
}
pub struct MintAccount {
    pub key: Pubkey,
    pub info: Mint,
    pub program_id: Pubkey,
    pub is_writable: bool,
}
impl MintAccount {
    pub fn unpack(info: &AccountInfo) -> Result<MintAccount, EscrowError> {
        Ok(MintAccount {
            key: *info.key,
            info: match Mint::unpack(&info.try_borrow_data().unwrap()) {
                Ok(s) => s,
                Err(_) => return Err(EscrowError::DeserializeMintAccountError),
            },
            program_id: *info.owner,
            is_writable: info.is_writable,
        })
    }
}
