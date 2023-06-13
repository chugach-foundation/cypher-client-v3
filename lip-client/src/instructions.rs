use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, sysvar::SysvarId},
    system_program, InstructionData,
};
use anchor_spl::{associated_token, token};

use crate::accounts::{CreateCampaign, CreateDeposit, EndDeposit};

pub fn create_campaign(
    campaign: &Pubkey,
    reward_mint: &Pubkey,
    funding_account: &Pubkey,
    campaign_reward_vault: &Pubkey,
    campaign_reward_vault_authority: &Pubkey,
    asset_mint: &Pubkey,
    pool: &Pubkey,
    pool_node: &Pubkey,
    pool_node_vault: &Pubkey,
    funding_authority: &Pubkey,
    campaign_authority: &Pubkey,
    payer: &Pubkey,
    deposit_reward_ratio: u16,
    lockup_period: u64,
    min_deposit: u64,
    max_deposits: u64,
    max_rewards: u64,
) -> Instruction {
    let accounts = CreateCampaign {
        campaign: *campaign,
        reward_mint: *reward_mint,
        funding_account: *funding_account,
        campaign_reward_vault: *campaign_reward_vault,
        campaign_reward_vault_authority: *campaign_reward_vault_authority,
        asset_mint: *asset_mint,
        pool: *pool,
        pool_node: *pool_node,
        pool_node_vault: *pool_node_vault,
        funding_authority: *funding_authority,
        campaign_authority: *campaign_authority,
        payer: *payer,
        rent: Rent::id(),
        token_program: token::ID,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreateCampaign {
        _deposit_reward_ratio: deposit_reward_ratio,
        _min_deposit: min_deposit,
        _lockup_period: lockup_period,
        _max_deposits: max_deposits,
        _max_rewards: max_rewards,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_deposit(
    deposit: &Pubkey,
    campaign: &Pubkey,
    cache_account: &Pubkey,
    clearing: &Pubkey,
    cypher_account: &Pubkey,
    cypher_sub_account: &Pubkey,
    funding_account: &Pubkey,
    asset_mint: &Pubkey,
    temp_token_account: &Pubkey,
    pool: &Pubkey,
    pool_node: &Pubkey,
    pool_node_vault: &Pubkey,
    deposit_authority: &Pubkey,
    payer: &Pubkey,
    signer: &Pubkey,
    cypher_program: &Pubkey,
    account_bump: u8,
    sub_account_bump: u8,
    amount: u64,
) -> Instruction {
    let accounts = CreateDeposit {
        deposit: *deposit,
        campaign: *campaign,
        cache_account: *cache_account,
        clearing: *clearing,
        cypher_account: *cypher_account,
        cypher_sub_account: *cypher_sub_account,
        funding_account: *funding_account,
        asset_mint: *asset_mint,
        temp_token_account: *temp_token_account,
        pool: *pool,
        pool_node: *pool_node,
        pool_node_vault: *pool_node_vault,
        deposit_authority: *deposit_authority,
        payer: *payer,
        signer: *signer,
        cypher_program: *cypher_program,
        rent: Rent::id(),
        token_program: token::ID,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreateDeposit {
        _account_bump: account_bump,
        _sub_account_bump: sub_account_bump,
        _amount: amount,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn end_deposit(
    deposit: &Pubkey,
    campaign: &Pubkey,
    campaign_reward_vault: &Pubkey,
    campaign_reward_vault_authority: &Pubkey,
    cache_account: &Pubkey,
    clearing: &Pubkey,
    cypher_account: &Pubkey,
    cypher_sub_account: &Pubkey,
    asset_mint: &Pubkey,
    asset_token_account: &Pubkey,
    temp_token_account: &Pubkey,
    pool: &Pubkey,
    pool_node: &Pubkey,
    pool_node_vault: &Pubkey,
    pool_node_vault_signer: &Pubkey,
    reward_mint: &Pubkey,
    reward_token_account: &Pubkey,
    deposit_authority: &Pubkey,
    payer: &Pubkey,
    signer: &Pubkey,
    cypher_program: &Pubkey,
) -> Instruction {
    let accounts = EndDeposit {
        deposit: *deposit,
        campaign: *campaign,
        campaign_reward_vault: *campaign_reward_vault,
        campaign_reward_vault_authority: *campaign_reward_vault_authority,
        cache_account: *cache_account,
        clearing: *clearing,
        cypher_account: *cypher_account,
        cypher_sub_account: *cypher_sub_account,
        asset_mint: *asset_mint,
        asset_token_account: *asset_token_account,
        temp_token_account: *temp_token_account,
        pool: *pool,
        pool_node: *pool_node,
        pool_node_vault: *pool_node_vault,
        pool_node_vault_signer: *pool_node_vault_signer,
        reward_mint: *reward_mint,
        reward_token_account: *reward_token_account,
        deposit_authority: *deposit_authority,
        payer: *payer,
        signer: *signer,
        cypher_program: *cypher_program,
        rent: Rent::id(),
        token_program: token::ID,
        associated_token_program: associated_token::ID,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::EndDeposit {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}
