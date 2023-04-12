use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::{check_program_account, processor::Processor};
entrypoint!(process_instruction);
fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(e) = check_program_account(program_id) {
        return Err(e);
    }
    if let Err(e) = Processor::process(accounts, instruction_data) {
        return Err(e.print_into());
    };
    Ok(())
}
