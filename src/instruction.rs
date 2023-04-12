use solana_program::msg;
use std::convert::TryInto;

use crate::error::EscrowError;
use strum_macros::AsRefStr;
#[derive(AsRefStr)]
pub enum EscrowInstruction {
    /// Starts the trade by creating and populating an escrow account and transferring token of the given mint(Mint A) to the ATA owned by Escrow account
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer, writable]` The account of the person initializing the escrow
    /// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
    /// 1. `[writable]` Temporary token A account  owned by the escrow account
    /// 2. `[writable]` The initializer's A token account for the token they will transfer
    /// 3. `[writable]` The mint of token A.
    /// 3. `[writable]` The mint of token B.
    /// 5. `[]` The token program
    /// 5. `[]` The associated token program
    /// 5. `[]` The system program
    InitEscrow {
        /// amount of A token to trade
        amount_to_trade: u64,
        /// amount of B token to trade
        amount_expected: u64,
        /// random seed
        seed: u64,
    },
    /// Accepts a trade
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person taking the trade
    /// 1. `[writable]` The taker's token account for the token they send
    /// 2. `[writable]` The taker's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The PDA's temp token account to get tokens from and eventually close
    /// 4. `[writable]` The initializer's main account to send their rent fees to
    /// 5. `[writable]` The initializer's token account that will receive tokens
    /// 6. `[writable]` The escrow account holding the escrow info
    /// 7. `[]` The token program
    /// 8. `[]` The PDA account
    Exchange {
        /// the amount the taker expects to be paid in the other token, as a u64 because that's the max possible supply of a token
        amount: u64,
    },
}

impl EscrowInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, EscrowError> {
        let (tag, rest) = match input.split_first() {
            Some(s) => s,
            None => return Err(EscrowError::InvalidInstructionType),
        };

        Ok(match tag {
            0 => {
                let (amount_to_trade, rest) = Self::unpack_u64(rest)?;
                let (amount_expected, rest) = Self::unpack_u64(rest)?;
                let (seed, _) = Self::unpack_u64(rest)?;
                Self::InitEscrow {
                    amount_expected,
                    seed,
                    amount_to_trade,
                }
            }
            1 => Self::Exchange {
                amount: Self::unpack_amount(rest)?,
            },
            _ => return Err(EscrowError::InvalidInstructionType),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, EscrowError> {
        return match input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
        {
            Some(a) => Ok(a),
            None => Err(EscrowError::InvalidInstructionData),
        };
    }
    fn unpack_u64(input: &[u8]) -> Result<(u64, &[u8]), EscrowError> {
        let (amount, rest) = input.split_at(8);
        let amount = amount
            .try_into()
            .ok()
            .map(u64::from_le_bytes)
            .ok_or(EscrowError::InvalidInstructionData)?;
        Ok((amount, rest))
    }
    pub fn print_instruction_name(self) -> EscrowInstruction {
        msg!(self.as_ref());
        self
    }
}
