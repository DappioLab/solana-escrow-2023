use crate::{
    error::EscrowError,
    state::{EscrowAccount, MintAccount, TokenAccount},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
};
use spl_associated_token_account::get_associated_token_address;

pub struct InitEscrowAccount<'a> {
    pub initializer: &'a AccountInfo<'a>,
    pub escrow_state: &'a AccountInfo<'a>,
    pub token_a_vault: &'a AccountInfo<'a>,
    pub token_a_founder: TokenAccount,
    pub token_a_mint: MintAccount,
    pub token_b_mint: MintAccount,
}
impl<'a> InitEscrowAccount<'a> {
    pub fn unpack(accounts: &'a [AccountInfo<'a>]) -> Result<InitEscrowAccount<'a>, EscrowError> {
        let account_info_iter = &mut accounts.iter();
        let initializer = unwrap_iter(next_account_info(account_info_iter))?;
        let escrow_state = unwrap_iter(next_account_info(account_info_iter))?;
        let token_a_vault = unwrap_iter(next_account_info(account_info_iter))?;
        let token_a_founder =
            TokenAccount::unpack(unwrap_iter(next_account_info(account_info_iter))?)?;
        let token_a_mint = MintAccount::unpack(unwrap_iter(next_account_info(account_info_iter))?)?;
        let token_b_mint = MintAccount::unpack(unwrap_iter(next_account_info(account_info_iter))?)?;
        let _spl_token_program = unwrap_iter(next_account_info(account_info_iter))?;
        let _ata_program = unwrap_iter(next_account_info(account_info_iter))?;
        let _system_program = unwrap_iter(next_account_info(account_info_iter))?;
        // validate accounts
        if !initializer.is_signer {
            return Err(EscrowError::InvalidSigner);
        }
        if !get_associated_token_address(escrow_state.key, &token_a_mint.key).eq(&token_a_vault.key)
        {
            return Err(EscrowError::VaultKeyMismatch);
        };
        match account_info_iter.next() {
            Some(_) => return Err(EscrowError::NotEnoughAccountKeys),
            None => Ok(InitEscrowAccount {
                initializer,
                token_a_vault,
                token_a_founder,
                escrow_state,
                token_a_mint,
                token_b_mint,
            }),
        }
    }
}

pub struct ExchangeAccount<'a> {
    pub taker: &'a AccountInfo<'a>,
    pub initializer: &'a AccountInfo<'a>,
    pub escrow_state: &'a AccountInfo<'a>,
    pub escrow_account_info: EscrowAccount,
    pub token_a_vault: TokenAccount,
    pub token_a_receiver: &'a AccountInfo<'a>,
    pub token_b_receiver: &'a AccountInfo<'a>,
    pub token_b_founder: TokenAccount,
    pub token_a_mint: MintAccount,
    pub token_b_mint: MintAccount,
}
impl<'a> ExchangeAccount<'a> {
    pub fn unpack(
        accounts: &'a [AccountInfo<'a>],
        amount_expected_by_taker: u64,
    ) -> Result<ExchangeAccount<'a>, EscrowError> {
        let account_info_iter = &mut accounts.iter();
        let taker = unwrap_iter(next_account_info(account_info_iter))?;
        let initializer = unwrap_iter(next_account_info(account_info_iter))?;
        let escrow_state = unwrap_iter(next_account_info(account_info_iter))?;
        let escrow_account_info = EscrowAccount::unpack(escrow_state)?;
        let token_a_vault =
            TokenAccount::unpack(unwrap_iter(next_account_info(account_info_iter))?)?;
        let token_a_receiver = unwrap_iter(next_account_info(account_info_iter))?;
        let token_b_receiver = unwrap_iter(next_account_info(account_info_iter))?;
        let token_b_founder =
            TokenAccount::unpack(unwrap_iter(next_account_info(account_info_iter))?)?;

        let token_a_mint = MintAccount::unpack(unwrap_iter(next_account_info(account_info_iter))?)?;
        let token_b_mint = MintAccount::unpack(unwrap_iter(next_account_info(account_info_iter))?)?;
        let _spl_token_program = unwrap_iter(next_account_info(account_info_iter))?;
        let _ata_program = unwrap_iter(next_account_info(account_info_iter))?;
        let _system_program = unwrap_iter(next_account_info(account_info_iter))?;

        // validate accounts
        if !taker.is_signer {
            return Err(EscrowError::InvalidSigner);
        }
        if !get_associated_token_address(escrow_state.key, &token_a_mint.key).eq(&token_a_vault.key)
        {
            return Err(EscrowError::VaultKeyMismatch);
        };
        if !amount_expected_by_taker.eq(&escrow_account_info.info.expected_amount) {
            return Err(EscrowError::ExpectedAmountMismatch);
        }
        if !token_a_mint.key.eq(&escrow_account_info.info.mint_a) {
            return Err(EscrowError::MintAMismatch);
        }
        if !token_b_mint.key.eq(&escrow_account_info.info.mint_b) {
            return Err(EscrowError::MintBMismatch);
        }

        match account_info_iter.next() {
            Some(_) => return Err(EscrowError::NotEnoughAccountKeys),
            None => Ok(ExchangeAccount {
                taker,
                initializer,
                token_a_receiver,
                token_b_founder,
                token_b_receiver,
                token_a_vault,
                escrow_state,
                escrow_account_info,
                token_a_mint,
                token_b_mint,
            }),
        }
    }
}
fn unwrap_iter<'a>(
    i: Result<&'a AccountInfo<'a>, ProgramError>,
) -> Result<&'a AccountInfo<'a>, EscrowError> {
    match i {
        Ok(s) => Ok(s),
        Err(_) => Err(EscrowError::NotEnoughAccountKeys),
    }
}
