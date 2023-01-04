#![allow(dead_code)]
use {
    anchor_lang::{prelude::*, Discriminator, ZeroCopy},
    anchor_spl::{associated_token, dex, token::spl_token},
    arrayref::array_ref,
    bytemuck::{bytes_of, from_bytes},
    fixed::types::I80F48,
};

use crate::{constants::*, ClearingType};

pub fn adjust_decimals(value: I80F48, decimals: u8) -> I80F48 {
    match decimals {
        0 => value,
        1 => value.checked_mul(INV_ONE_DECIMAL_ADJ_FIXED).unwrap(),
        2 => value.checked_mul(INV_TWO_DECIMAL_ADJ_FIXED).unwrap(),
        3 => value.checked_mul(INV_THREE_DECIMAL_ADJ_FIXED).unwrap(),
        4 => value.checked_mul(INV_FOUR_DECIMAL_ADJ_FIXED).unwrap(),
        5 => value.checked_mul(INV_FIVE_DECIMAL_ADJ_FIXED).unwrap(),
        6 => value.checked_mul(INV_SIX_DECIMAL_ADJ_FIXED).unwrap(),
        7 => value.checked_mul(INV_SEVEN_DECIMAL_ADJ_FIXED).unwrap(),
        8 => value.checked_mul(INV_EIGHT_DECIMAL_ADJ_FIXED).unwrap(),
        9 => value.checked_mul(INV_NINE_DECIMAL_ADJ_FIXED).unwrap(),
        _ => unreachable!(),
    }
}

#[inline(always)]
pub fn convert_price_to_lots(
    price: u64,
    base_multiplier: u64,
    coin_decimals_factor: u64,
    quote_multiplier: u64,
) -> u64 {
    (price * base_multiplier) / (coin_decimals_factor * quote_multiplier)
}
#[inline(always)]
pub fn convert_price_to_lots_fixed(
    price: I80F48,
    base_multiplier: u64,
    coin_decimals_factor: u64,
    quote_multiplier: u64,
) -> u64 {
    price
        .checked_mul(I80F48::from(base_multiplier))
        .and_then(|n| n.checked_div(I80F48::from(coin_decimals_factor * quote_multiplier)))
        .unwrap()
        .to_num()
}

#[inline(always)]
pub fn convert_price_to_decimals(
    price: u64,
    coin_lot_size: u64,
    coin_decimals_factor: u64,
    pc_lot_size: u64,
) -> u64 {
    let res =
        price as u128 * pc_lot_size as u128 * coin_decimals_factor as u128 / coin_lot_size as u128;
    res as u64
}

#[inline(always)]
pub fn convert_coin_to_lots(coin: u64, coin_lot_size: u64) -> u64 {
    coin / coin_lot_size
}

#[inline(always)]
pub fn convert_pc_to_lots(pc: u64, pc_lot_size: u64) -> u64 {
    pc / pc_lot_size
}

#[inline(always)]
pub fn convert_coin_to_decimals(coin: u64, coin_lot_size: u64) -> u64 {
    coin * coin_lot_size
}

#[inline(always)]
pub fn convert_pc_to_decimals(pc: u64, pc_lot_size: u64) -> u64 {
    pc * pc_lot_size
}

pub fn fixed_to_ui(number: I80F48, decimals: u8) -> I80F48 {
    number / I80F48::from_num::<u64>(10_u64.checked_pow(decimals as u32).unwrap())
}

pub fn fixed_to_ui_price(number: I80F48, base_decimals: u8, quote_decimals: u8) -> I80F48 {
    number
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

pub fn derive_sub_account_address(master_account: &Pubkey, account_number: u8) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(
        &[
            B_CYPHER_SUB_ACCOUNT,
            master_account.as_ref(),
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

pub fn derive_pool_node_address(pool: &Pubkey, node_number: u8) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(
        &[
            B_POOL_NODE,
            pool.as_ref(),
            node_number.to_le_bytes().as_ref(),
        ],
        &crate::id(),
    );

    (address, bump)
}

pub fn derive_pool_node_vault_address(pool_node: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) =
        Pubkey::find_program_address(&[B_POOL_NODE_VAULT, pool_node.as_ref()], &crate::id());

    (address, bump)
}

pub fn derive_pool_node_vault_signer_address(pool_node: &Pubkey) -> (Pubkey, u8) {
    let (address, bump) = Pubkey::find_program_address(
        &[B_POOL_NODE_VAULT_SIGNER, pool_node.as_ref()],
        &crate::id(),
    );

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
    sub_account: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            B_OPEN_ORDERS,
            dex_market.as_ref(),
            master_account.as_ref(),
            sub_account.as_ref(),
        ],
        &crate::id(),
    )
}

pub fn derive_orders_account_address(market: &Pubkey, master_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[B_ORDERS_ACCOUNT, market.as_ref(), master_account.as_ref()],
        &crate::id(),
    )
}
