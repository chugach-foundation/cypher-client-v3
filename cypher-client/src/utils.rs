#![allow(dead_code)]
use {
    anchor_lang::{prelude::*, Discriminator, ZeroCopy},
    anchor_spl::{associated_token, dex, token::spl_token},
    arrayref::array_ref,
    bytemuck::{bytes_of, from_bytes},
    fixed::types::I80F48,
    std::cmp::Ordering,
};

use crate::{constants::*, ClearingType};

pub fn fixed_to_ui(number: I80F48, decimals: u8) -> I80F48 {
    number / I80F48::from_num::<u64>(10_u64.checked_pow(decimals as u32).unwrap())
}

pub fn native_to_ui(number: u64, decimals: u8) -> u64 {
    number / 10_u64.checked_pow(decimals as u32).unwrap()
}

pub fn native_to_ui_price(number: u64, base_decimals: u8, quote_decimals: u8) -> I80F48 {
    I80F48::from_num::<u64>(number)
        .checked_mul(I80F48::from_num::<u64>(
            10_u64.checked_pow(base_decimals as u32).unwrap(),
        ))
        .and_then(|n| {
            n.checked_div(I80F48::from_num::<u64>(
                10_u64.checked_pow(quote_decimals as u32).unwrap(),
            ))
        })
        .unwrap()
}

pub fn get_oracle_products_space(num_products: usize) -> usize {
    let num_weights = match num_products.cmp(&2) {
        Ordering::Greater => num_products,
        Ordering::Equal => 0,
        Ordering::Less => 0,
    };
    8 + 8 + 32 + 32 + 8 + num_products * 32 + 8 + num_weights * 2
}

pub fn get_zero_copy_account<T: ZeroCopy + Owner>(account_data: &[u8]) -> Box<T> {
    let disc_bytes = array_ref![account_data, 0, 8];
    assert_eq!(disc_bytes, &T::discriminator());
    Box::new(*from_bytes::<T>(
        &account_data[8..std::mem::size_of::<T>() + 8],
    ))
}

pub fn get_program_account<
    T: AccountSerialize + AccountDeserialize + Discriminator + Clone + Owner,
>(
    account_data: &mut &[u8],
) -> Box<T> {
    Box::new(<T>::try_deserialize(account_data).unwrap())
}
pub fn gen_dex_vault_signer_key(nonce: u64, dex_market: &Pubkey) -> Result<Pubkey> {
    let seeds = [dex_market.as_ref(), bytes_of(&nonce)];
    Ok(Pubkey::create_program_address(&seeds, &dex::id()).unwrap())
}

pub fn derive_token_address(wallet_address: &Pubkey, token_mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            wallet_address.as_ref(),
            &spl_token::id().to_bytes(),
            token_mint.as_ref(),
        ],
        &associated_token::ID,
    )
    .0
}

pub fn derive_public_clearing_address() -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(
        &[
            B_CLEARING,
            ClearingType::Public.try_to_vec().unwrap().as_ref(),
        ],
        &crate::id(),
    );

    (address, bump)
}

pub fn derive_private_clearing_address(clearing_number: u8) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(
        &[
            B_CLEARING,
            ClearingType::Private.try_to_vec().unwrap().as_ref(),
            clearing_number.to_le_bytes().as_ref(),
        ],
        &crate::id(),
    );

    (address, bump)
}

pub fn derive_oracle_products_address(symbol: &[u8]) -> (Pubkey, u8) {
    let (address, bump) =
        Pubkey::find_program_address(&[B_ORACLE_PRODUCTS, symbol.as_ref()], &crate::id());

    (address, bump)
}

pub fn derive_account_address(authority: &Pubkey, account_number: u8) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(
        &[
            B_CYPHER_ACCOUNT,
            authority.as_ref(),
            account_number.to_le_bytes().as_ref(),
        ],
        &crate::id(),
    );

    (address, bump)
}

pub fn derive_sub_account_address(authority: &Pubkey, account_number: u8) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(
        &[
            B_CYPHER_SUB_ACCOUNT,
            authority.as_ref(),
            account_number.to_le_bytes().as_ref(),
        ],
        &crate::id(),
    );

    (address, bump)
}

pub fn derive_pool_address(pool_name: &[u8]) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(&[B_POOL, pool_name], &crate::id());

    (address, bump)
}

pub fn derive_pool_vault_address(pool: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) =
        Pubkey::find_program_address(&[B_POOL_VAULT, pool.as_ref()], &crate::id());

    (address, bump)
}

pub fn derive_market_address(market_name: &[u8]) -> (Pubkey, u8) {
    let (address, bump) =
        Pubkey::find_program_address(&[B_CYPHER_MARKET, market_name], &crate::id());
    (address, bump)
}

pub fn derive_whitelist_address(account_owner: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) =
        Pubkey::find_program_address(&[B_WHITELIST, account_owner.as_ref()], &crate::id());
    (address, bump)
}

pub fn derive_spot_open_orders_address(
    dex_market: &Pubkey,
    master_account: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[B_OPEN_ORDERS, dex_market.as_ref(), master_account.as_ref()],
        &crate::id(),
    )
}

pub fn derive_orders_account_address(market: &Pubkey, master_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[B_ORDERS_ACCOUNT, market.as_ref(), master_account.as_ref()],
        &crate::id(),
    )
}
