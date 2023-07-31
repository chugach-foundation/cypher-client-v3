use anchor_lang::{
    prelude::*, solana_program::instruction::Instruction, system_program, InstructionData,
};
use anchor_spl::token;

anchor_gen::generate_cpi_interface!(idl_path = "idl.json",);

#[cfg(feature = "devnet")]
declare_id!("2gCkR5aaUiTVRiKDB79EWXm5PAVDWtNTnp9mGuu4ZKdY");
#[cfg(not(feature = "devnet"))]
declare_id!("2gCkR5aaUiTVRiKDB79EWXm5PAVDWtNTnp9mGuu4ZKdY");

const B_FAUCET: &[u8] = b"FAUCET";
const B_MINT_AUTHORITY: &[u8] = b"MINT_AUTHORITY";

pub fn derive_faucet_address(mint: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(&[B_FAUCET, mint.as_ref()], &crate::id());

    (address, bump)
}

pub fn derive_mint_authority_address(mint: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) =
        Pubkey::find_program_address(&[B_MINT_AUTHORITY, mint.as_ref()], &crate::id());

    (address, bump)
}

pub fn init_faucet(
    faucet: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    payer: &Pubkey,
    mint_authority_bump: u8,
) -> Instruction {
    let accounts = crate::accounts::InitFaucet {
        faucet: *faucet,
        mint: *mint,
        mint_authority: *mint_authority,
        payer: *payer,
        system_program: system_program::ID,
    };

    let ix_data = crate::instruction::InitFaucet {
        _mint_authority_bump: mint_authority_bump,
    };

    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn request(
    faucet: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    destination_token_account: &Pubkey,
) -> Instruction {
    let accounts = crate::accounts::Request {
        faucet: *faucet,
        mint: *mint,
        mint_authority: *mint_authority,
        target: *destination_token_account,
        token_program: token::ID,
    };

    let ix_data = crate::instruction::Request {};

    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}
