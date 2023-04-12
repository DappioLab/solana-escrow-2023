use crate::{
    accounts::ExchangeAccount, accounts::InitEscrowAccount, error::EscrowError, id,
    instruction::EscrowInstruction, state::EscrowState,
};
use solana_program::{
    account_info::AccountInfo,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::{self},
    sysvar::Sysvar,
};
use spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use spl_token::instruction::{close_account, transfer_checked};
pub struct Processor;

impl Processor {
    pub fn process<'a>(
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> Result<(), EscrowError> {
        return match EscrowInstruction::unpack(instruction_data) {
            Ok(s) => match s {
                EscrowInstruction::Exchange { amount } => Self::process_exchange(accounts, amount),
                EscrowInstruction::InitEscrow {
                    amount_to_trade,
                    amount_expected,
                    seed,
                } => Self::process_init_escrow(accounts, amount_to_trade, amount_expected, seed),
            },
            Err(e) => Err(e),
        };
    }
    fn process_init_escrow<'a>(
        accounts: &'a [AccountInfo<'a>],
        amount_to_trade: u64,
        amount_expected: u64,
        seed: u64,
    ) -> Result<(), EscrowError> {
        let ctx = InitEscrowAccount::unpack(accounts)?;

        // derived the key from seed
        let (escrow_key, bump) = Pubkey::find_program_address(
            &[&seed.to_le_bytes(), &ctx.initializer.key.to_bytes()],
            &id(),
        );
        // access rent info
        let rent_info = Rent::get().unwrap();
        // create escrow account
        let create_account_ix = system_instruction::create_account(
            ctx.initializer.key,
            ctx.escrow_state.key,
            rent_info.minimum_balance(EscrowState::LEN),
            EscrowState::LEN.try_into().unwrap(),
            &id(),
        );
        invoke_signed(
            &create_account_ix,
            accounts,
            &[&[
                &seed.to_le_bytes(),
                &ctx.initializer.key.to_bytes(),
                &bump.to_le_bytes(),
            ]],
        )
        .unwrap();

        // create vault account
        let create_ata_ix = create_associated_token_account_idempotent(
            ctx.initializer.key,
            &escrow_key,
            &ctx.token_a_mint.key,
            &spl_token::ID,
        );
        invoke(&create_ata_ix, accounts).unwrap();

        // transfer token A to vault
        let transfer = transfer_checked(
            &spl_token::ID,
            &ctx.token_a_founder.key,
            &ctx.token_a_mint.key,
            &ctx.token_a_vault.key,
            ctx.initializer.key,
            &[],
            amount_to_trade,
            ctx.token_a_mint.info.decimals,
        )
        .unwrap();
        invoke(&transfer, accounts).unwrap();

        // update state back on chain
        EscrowState::pack(
            EscrowState {
                is_initialized: true,
                initializer_pubkey: ctx.initializer.key.clone(),
                mint_a: ctx.token_a_mint.key,
                mint_b: ctx.token_b_mint.key,
                expected_amount: amount_expected,
                bump: bump,
                seed: seed,
            },
            &mut ctx.escrow_state.try_borrow_mut_data().unwrap(),
        )
        .unwrap();
        Ok(())
    }
    fn process_exchange<'a>(
        accounts: &'a [AccountInfo<'a>],
        amount_expected_by_taker: u64,
    ) -> Result<(), EscrowError> {
        let ctx = ExchangeAccount::unpack(accounts, amount_expected_by_taker)?;

        // create A token account owned by the taker
        let create_a_reciever_ata = create_associated_token_account_idempotent(
            &ctx.taker.key,
            &ctx.taker.key,
            &ctx.token_a_mint.key,
            &spl_token::ID,
        );
        invoke(&create_a_reciever_ata, accounts).unwrap();

        // transfer out token A
        let take_transfer = transfer_checked(
            &spl_token::ID,
            &ctx.token_a_vault.key,
            &ctx.token_a_mint.key,
            ctx.token_a_receiver.key,
            &ctx.escrow_account_info.key,
            &[],
            ctx.token_a_vault.info.amount,
            ctx.token_a_mint.info.decimals,
        )
        .unwrap();
        invoke_signed(
            &take_transfer,
            accounts,
            &[&[
                &ctx.escrow_account_info.info.seed.clone().to_le_bytes(),
                &ctx.initializer.key.to_bytes(),
                &ctx.escrow_account_info.info.bump.clone().to_le_bytes(),
            ]],
        )
        .unwrap();

        // create B token account owned by the initializer
        let creata_b_reciever_ata = create_associated_token_account_idempotent(
            &ctx.taker.key,
            &ctx.escrow_account_info.key,
            &ctx.token_b_mint.key,
            &spl_token::ID,
        );
        invoke(&creata_b_reciever_ata, accounts).unwrap();

        // transfer token B from taker to initializer
        let transfer_b = transfer_checked(
            &spl_token::ID,
            &ctx.token_b_founder.key,
            &ctx.token_b_mint.key,
            ctx.token_b_receiver.key,
            &ctx.taker.key,
            &[],
            ctx.escrow_account_info.info.expected_amount,
            ctx.token_b_mint.info.decimals,
        )
        .unwrap();
        invoke(&transfer_b, accounts).unwrap();

        // close vault account return rent back to initializer
        let close_vault = close_account(
            &spl_token::ID,
            &ctx.token_a_vault.key,
            &ctx.escrow_account_info.key,
            &ctx.escrow_account_info.key,
            &[],
        )
        .unwrap();
        invoke_signed(
            &close_vault,
            accounts,
            &[&[
                &ctx.escrow_account_info.info.seed.clone().to_le_bytes(),
                &ctx.initializer.key.to_bytes(),
                &ctx.escrow_account_info.info.bump.clone().to_le_bytes(),
            ]],
        )
        .unwrap();
        // transfer the rent inside escrow account back to initializer
        **ctx.initializer.try_borrow_mut_lamports().unwrap() = ctx
            .initializer
            .lamports()
            .checked_add(ctx.escrow_state.lamports())
            .unwrap();
        **ctx.escrow_state.try_borrow_mut_lamports().unwrap() = 0;
        // clear escrow account
        EscrowState::pack(
            EscrowState::default(),
            &mut ctx.escrow_state.try_borrow_mut_data().unwrap(),
        )
        .unwrap();
        Ok(())
    }
}
