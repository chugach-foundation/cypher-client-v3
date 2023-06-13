use anchor_lang::prelude::*;

use crate::constants::{CAMPAIGN_AUTH_SEED, CAMPAIGN_SEED, DEPOSIT_AUTH_SIGNER_SEED};

pub fn derive_campaign_reward_vault(campaign: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) =
        Pubkey::find_program_address(&[CAMPAIGN_SEED, campaign.as_ref()], &crate::id());

    (address, bump)
}

pub fn derive_campaign_reward_vault_authority(campaign: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) =
        Pubkey::find_program_address(&[CAMPAIGN_AUTH_SEED, campaign.as_ref()], &crate::id());

    (address, bump)
}

pub fn derive_deposit_authority(deposit: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) =
        Pubkey::find_program_address(&[DEPOSIT_AUTH_SIGNER_SEED, deposit.as_ref()], &crate::id());

    (address, bump)
}
