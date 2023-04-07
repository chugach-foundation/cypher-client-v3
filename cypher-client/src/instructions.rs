#![allow(clippy::too_many_arguments)]
use anchor_lang::{
    prelude::{AccountMeta, ProgramError, Pubkey, Rent},
    solana_program::{instruction::Instruction, sysvar::SysvarId},
    system_program, InstructionData, ToAccountMetas,
};
use anchor_spl::token;

use crate::{
    accounts::{
        CacheOraclePrices, CancelFuturesOrder, CancelFuturesOrders, CancelPerpOrder,
        CancelPerpOrders, CancelSpotOrder, CancelSpotOrderDex, ClaimIdoProceeds, CloseAccount,
        CloseCacheAccount, CloseClearing, CloseFuturesMarket, CloseOracleProducts, ClosePerpMarket,
        ClosePool, ClosePoolNode, CloseSpotOpenOrders, CloseSubAccount, ConsumeFuturesEvents,
        ConsumePerpEvents, CreateAccount, CreateFuturesMarket, CreateOracleProducts,
        CreateOracleStub, CreateOrdersAccount, CreatePerpMarket, CreatePool, CreatePoolNode,
        CreatePrivateClearing, CreatePublicClearing, CreateSubAccount, CreateWhitelist,
        CreateWhitelistedAccount, DepositDeliverable, DepositFunds, InitCacheAccount,
        InitSpotOpenOrders, LiquidateFuturesPosition, LiquidatePerpPosition, LiquidateSpotPosition,
        MultipleNewFuturesOrders, MultipleNewPerpOrders, NewFuturesOrder, NewPerpOrder,
        NewSpotOrder, NewSpotOrderDex, RollMarketExpiry, SetAccountDelegate, SetCacheAuthority,
        SetClearingAuthority, SetClearingFeeMint, SetClearingFeeTiers, SetFuturesMarketAuthority,
        SetFuturesMarketLiquidityMiningInfo, SetFuturesMarketParams, SetFuturesMarketStatus,
        SetOracleProducts, SetOracleProductsV2, SetOracleStubPrice, SetPerpetualMarketAuthority,
        SetPerpetualMarketLiquidityMiningInfo, SetPerpetualMarketParams, SetPerpetualMarketStatus,
        SetPoolAuthority, SetPoolDexMarket, SetPoolNodeAuthority, SetPoolNodeStatus, SetPoolParams,
        SetPoolStatus, SetSubAccountDelegate, SettleFunding, SettleFuturesFunds, SettlePerpFunds,
        SettlePosition, SettlePositionWithDelivery, SettleSpotFunds, SettleSpotFundsDex,
        SweepMarketFees, SweepPoolFees, TransferBetweenSubAccounts, UpdateAccountMargin,
        UpdateFundingRate, UpdateMarketExpiration, UpdateTokenIndex, UpgradeOracleProducts,
        WithdrawFunds,
    },
    constants::SUB_ACCOUNT_ALIAS_LEN,
    dex, quote_mint, CancelOrderArgs, CreateClearingArgs, CreateFuturesMarketArgs,
    CreateOracleProductsArgs, CreatePerpetualMarketArgs, CreatePoolArgs, FeeTierArgs,
    LiquidityMiningArgs, NewDerivativeOrderArgs, NewSpotOrderArgs, OperatingStatus, ProductsType,
};

pub fn create_public_clearing(
    clearing: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    args: CreateClearingArgs,
) -> Instruction {
    let accounts = CreatePublicClearing {
        clearing: *clearing,
        authority: *authority,
        payer: *payer,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreatePublicClearing { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}
pub fn create_private_clearing(
    public_clearing: &Pubkey,
    private_clearing: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    args: CreateClearingArgs,
) -> Instruction {
    let accounts = CreatePrivateClearing {
        clearing: *public_clearing,
        private_clearing: *private_clearing,
        authority: *authority,
        payer: *payer,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreatePrivateClearing { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_account(
    clearing: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    account: &Pubkey,
    account_bump: u8,
    account_number: u8,
) -> Instruction {
    let accounts = CreateAccount {
        clearing: *clearing,
        master_account: *account,
        payer: *payer,
        authority: *authority,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreateAccount {
        _account_bump: account_bump,
        _account_number: account_number,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_whitelisted_account(
    clearing: &Pubkey,
    whitelist: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    account: &Pubkey,
    account_bump: u8,
    account_number: u8,
) -> Instruction {
    let accounts = CreateWhitelistedAccount {
        clearing: *clearing,
        master_account: *account,
        whitelist: *whitelist,
        payer: *payer,
        authority: *authority,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreateWhitelistedAccount {
        _account_bump: account_bump,
        _account_number: account_number,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_sub_account(
    authority: &Pubkey,
    payer: &Pubkey,
    account: &Pubkey,
    sub_account: &Pubkey,
    sub_account_bump: u8,
    sub_account_number: u8,
    sub_account_alias: [u8; SUB_ACCOUNT_ALIAS_LEN],
) -> Instruction {
    let accounts = CreateSubAccount {
        master_account: *account,
        sub_account: *sub_account,
        payer: *payer,
        authority: *authority,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreateSubAccount {
        _sub_account_bump: sub_account_bump,
        _sub_account_number: sub_account_number,
        _sub_account_alias: sub_account_alias,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_futures_market(
    clearing: &Pubkey,
    cache: &Pubkey,
    market: &Pubkey,
    price_history: &Pubkey,
    oracle_products: &Pubkey,
    quote_pool: &Pubkey,
    orderbook: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    event_queue: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    args: CreateFuturesMarketArgs,
) -> Instruction {
    let accounts = CreateFuturesMarket {
        clearing: *clearing,
        cache_account: *cache,
        market: *market,
        price_history: *price_history,
        oracle_products: *oracle_products,
        quote_pool: *quote_pool,
        orderbook: *orderbook,
        bids: *bids,
        asks: *asks,
        event_queue: *event_queue,
        authority: *authority,
        payer: *payer,
        system_program: system_program::ID,
        rent: Rent::id(),
    };
    let ix_data = crate::instruction::CreateFuturesMarket { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_perp_market(
    clearing: &Pubkey,
    cache: &Pubkey,
    market: &Pubkey,
    oracle_products: &Pubkey,
    quote_pool: &Pubkey,
    orderbook: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    event_queue: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    args: CreatePerpetualMarketArgs,
) -> Instruction {
    let accounts = CreatePerpMarket {
        clearing: *clearing,
        cache_account: *cache,
        market: *market,
        oracle_products: *oracle_products,
        quote_pool: *quote_pool,
        orderbook: *orderbook,
        bids: *bids,
        asks: *asks,
        event_queue: *event_queue,
        authority: *authority,
        payer: *payer,
        system_program: system_program::ID,
        rent: Rent::id(),
    };
    let ix_data = crate::instruction::CreatePerpMarket { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_pool(
    clearing: &Pubkey,
    cache: &Pubkey,
    pool: &Pubkey,
    pool_node: &Pubkey,
    token_mint: &Pubkey,
    token_vault: &Pubkey,
    vault_signer: &Pubkey,
    oracle_products: &Pubkey,
    dex_market: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    args: CreatePoolArgs,
) -> Instruction {
    let accounts = CreatePool {
        clearing: *clearing,
        cache_account: *cache,
        pool: *pool,
        pool_node: *pool_node,
        token_mint: *token_mint,
        token_vault: *token_vault,
        vault_signer: *vault_signer,
        oracle_products: *oracle_products,
        dex_market: *dex_market,
        authority: *authority,
        payer: *payer,
        system_program: system_program::ID,
        token_program: token::ID,
        rent: Rent::id(),
    };
    let ix_data = crate::instruction::CreatePool { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_pool_node(
    clearing: &Pubkey,
    pool: &Pubkey,
    pool_node: &Pubkey,
    token_mint: &Pubkey,
    token_vault: &Pubkey,
    vault_signer: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
    vault_signer_bump: u8,
    node_number: u8,
) -> Instruction {
    let accounts = CreatePoolNode {
        clearing: *clearing,
        pool: *pool,
        pool_node: *pool_node,
        token_mint: *token_mint,
        token_vault: *token_vault,
        vault_signer: *vault_signer,
        authority: *authority,
        payer: *payer,
        system_program: system_program::ID,
        token_program: token::ID,
        rent: Rent::id(),
    };
    let ix_data = crate::instruction::CreatePoolNode {
        _vault_signer_bump: vault_signer_bump,
        _node_number: node_number,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_whitelist(
    clearing: &Pubkey,
    whitelist: &Pubkey,
    account_owner: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = CreateWhitelist {
        clearing: *clearing,
        whitelist: *whitelist,
        account_owner: *account_owner,
        payer: *payer,
        authority: *authority,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreateWhitelist {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_oracle_products(
    cache_account: &Pubkey,
    oracle_products: &Pubkey,
    payer: &Pubkey,
    authority: &Pubkey,
    product_accounts: Option<&[Pubkey]>,
    args: CreateOracleProductsArgs,
) -> Instruction {
    let mut accounts = CreateOracleProducts {
        cache_account: *cache_account,
        oracle_products: *oracle_products,
        authority: *authority,
        payer: *payer,
        system_program: system_program::ID,
    }
    .to_account_metas(Some(false));

    if product_accounts.is_some() {
        accounts.extend(
            product_accounts
                .as_ref()
                .unwrap()
                .iter()
                .map(|p| AccountMeta::new_readonly(*p, false))
                .collect::<Vec<AccountMeta>>(),
        )
    }

    let ix_data = crate::instruction::CreateOracleProducts { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts,
        data: ix_data.data(),
    }
}

pub fn create_oracle_stub(oracle_stub: &Pubkey, payer: &Pubkey, symbol: [u8; 32]) -> Instruction {
    let accounts = CreateOracleStub {
        oracle_stub: *oracle_stub,
        payer: *payer,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::CreateOracleStub { _symbol: symbol };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_oracle_stub_price(oracle_stub: &Pubkey, price: i128) -> Instruction {
    let accounts = SetOracleStubPrice {
        oracle_stub: *oracle_stub,
    };
    let ix_data = crate::instruction::SetOracleStubPrice { _price: price };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn init_cache_account(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = InitCacheAccount {
        clearing: *clearing,
        cache_account: *cache_account,
        authority: *authority,
    }
    .to_account_metas(Some(false));
    let ix_data = crate::instruction::InitCacheAccount {};
    Instruction {
        program_id: crate::id(),
        accounts,
        data: ix_data.data(),
    }
}

pub fn set_oracle_products(
    clearing: &Pubkey,
    oracle_products: &Pubkey,
    authority: &Pubkey,
    product_accounts: &[Pubkey],
) -> Instruction {
    let mut accounts = SetOracleProducts {
        clearing: *clearing,
        oracle_products: *oracle_products,
        authority: *authority,
    }
    .to_account_metas(Some(false));

    accounts.extend(
        product_accounts
            .iter()
            .map(|p| AccountMeta::new_readonly(*p, false))
            .collect::<Vec<AccountMeta>>(),
    );

    let ix_data = crate::instruction::SetOracleProducts {};
    Instruction {
        program_id: crate::id(),
        accounts,
        data: ix_data.data(),
    }
}

pub fn set_oracle_products_v2(
    clearing: &Pubkey,
    oracle_products: &Pubkey,
    authority: &Pubkey,
    feed_accounts: &[Pubkey],
    products_type: ProductsType,
    time_to_live: u16,
    max_confidence_intervals: Vec<f64>,
    weights: Vec<u16>,
    pyth_weight: Option<u16>,
    switchboard_weight: Option<u16>,
    chainlink_weight: Option<u16>,
) -> Instruction {
    let mut accounts = SetOracleProductsV2 {
        clearing: *clearing,
        oracle_products: *oracle_products,
        authority: *authority,
    }
    .to_account_metas(Some(false));

    accounts.extend(
        feed_accounts
            .iter()
            .map(|p| AccountMeta::new_readonly(*p, false))
            .collect::<Vec<AccountMeta>>(),
    );

    let ix_data = crate::instruction::SetOracleProductsV2 {
        _products_type: products_type,
        _max_confidence_intervals: max_confidence_intervals,
        _weights: weights,
        _time_to_live: time_to_live,
        _pyth_weight: pyth_weight,
        _switchboard_weight: switchboard_weight,
        _chainlink_weight: chainlink_weight,
    };
    Instruction {
        program_id: crate::id(),
        accounts,
        data: ix_data.data(),
    }
}

pub fn upgrade_oracle_products(
    cache_account: &Pubkey,
    price_history: &Pubkey,
    oracle_products: &Pubkey,
    payer: &Pubkey,
    authority: &Pubkey,
    twap_longer_time_horizon: u64,
    twap_shorter_time_horizon: u64,
    twap_price_collection_tick: u64,
    bands_duration: u64,
    bands_threshold: u16,
) -> Instruction {
    let accounts = UpgradeOracleProducts {
        cache: *cache_account,
        price_history: *price_history,
        oracle_products: *oracle_products,
        payer: *payer,
        authority: *authority,
        system_program: system_program::ID,
    };
    let ix_data = crate::instruction::UpgradeOracleProducts {
        _twap_longer_time_horizon: twap_longer_time_horizon,
        _twap_shorter_time_horizon: twap_shorter_time_horizon,
        _twap_price_collection_tick: twap_price_collection_tick,
        _bands_duration: bands_duration,
        _bands_threshold: bands_threshold,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn create_orders_account(
    master_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = CreateOrdersAccount {
        master_account: *master_account,
        market: *market,
        open_orders: *open_orders,
        authority: *authority,
        payer: *payer,
        system_program: system_program::ID,
        rent: Rent::id(),
    };
    let ix_data = crate::instruction::CreateOrdersAccount {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn init_spot_open_orders(
    master_account: &Pubkey,
    sub_account: &Pubkey,
    pool: &Pubkey,
    token_mint: &Pubkey,
    dex_market: &Pubkey,
    open_orders: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = InitSpotOpenOrders {
        master_account: *master_account,
        sub_account: *sub_account,
        pool: *pool,
        token_mint: *token_mint,
        dex_market: *dex_market,
        open_orders: *open_orders,
        authority: *authority,
        payer: *payer,
        dex_program: dex::ID,
        system_program: system_program::ID,
        rent: Rent::id(),
    };
    let ix_data = crate::instruction::InitSpotOpenOrders {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

/// This instruction should only receive either a pool pubkey or a futures market pubkey.
/// It should never receive both pubkeys.
pub fn cache_oracle_prices(
    cache_account: &Pubkey,
    oracle_products: &Pubkey,
    price_accounts: &[Pubkey],
    cache_index: u64,
    pool: &Option<Pubkey>,
    futures_market: &Option<Pubkey>,
) -> Result<Instruction, ProgramError> {
    let mut accounts = CacheOraclePrices {
        cache_account: *cache_account,
        oracle_products: *oracle_products,
    }
    .to_account_metas(Some(false));
    accounts.extend(
        price_accounts
            .iter()
            .map(|p| AccountMeta::new_readonly(*p, false)),
    );
    if pool.is_some() && futures_market.is_some() {
        return Err(ProgramError::InvalidArgument);
    }
    if pool.is_some() {
        accounts.push(AccountMeta::new_readonly(pool.unwrap(), false));
    }
    if futures_market.is_some() {
        accounts.push(AccountMeta::new_readonly(futures_market.unwrap(), false));
    }
    let ix_data = crate::instruction::CacheOraclePrices {
        _cache_index: cache_index,
    };
    Ok(Instruction {
        program_id: crate::id(),
        accounts,
        data: ix_data.data(),
    })
}

/// This instruction should only receive a futures market pubkey.
/// It is the same instruction as in `cache_oracle_prices` but adapted for upgraded oracles.
/// in this specific case we expect a bunch more accounts, i.e
/// 0..n - switchboard feeds
/// n..2n - pyth price accounts
/// 2n+1 - chainlink program
/// 2n+1..3n+1 - chainlink
/// 3n+2 - price history
/// 3n+3 - futures market (opt)
pub fn cache_oracle_prices_v1(
    cache_account: &Pubkey,
    oracle_products: &Pubkey,
    price_history: &Pubkey,
    switchboard_aggregator_accounts: &[Pubkey],
    pyth_price_accounts: &[Pubkey],
    chainlink_program_id: &Pubkey,
    chainlink_store_accounts: &[Pubkey],
    cache_index: u64,
    futures_market: &Option<Pubkey>,
) -> Instruction {
    let mut accounts = CacheOraclePrices {
        cache_account: *cache_account,
        oracle_products: *oracle_products,
    }
    .to_account_metas(Some(false));

    accounts.extend(
        switchboard_aggregator_accounts
            .iter()
            .map(|p| AccountMeta::new_readonly(*p, false)),
    );
    accounts.extend(
        pyth_price_accounts
            .iter()
            .map(|p| AccountMeta::new_readonly(*p, false)),
    );
    if !chainlink_store_accounts.is_empty() {
        accounts.push(AccountMeta::new_readonly(*chainlink_program_id, false));
    }
    accounts.extend(
        chainlink_store_accounts
            .iter()
            .map(|p| AccountMeta::new_readonly(*p, false)),
    );
    accounts.push(AccountMeta::new(*price_history, false));
    if futures_market.is_some() {
        accounts.push(AccountMeta::new_readonly(futures_market.unwrap(), false));
    }
    let ix_data = crate::instruction::CacheOraclePrices {
        _cache_index: cache_index,
    };
    Instruction {
        program_id: crate::id(),
        accounts,
        data: ix_data.data(),
    }
}

pub fn close_spot_open_orders(
    master_account: &Pubkey,
    sub_account: &Pubkey,
    asset_pool: &Pubkey,
    token_mint: &Pubkey,
    dex_market: &Pubkey,
    open_orders: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = CloseSpotOpenOrders {
        master_account: *master_account,
        sub_account: *sub_account,
        asset_pool: *asset_pool,
        token_mint: *token_mint,
        dex_market: *dex_market,
        open_orders: *open_orders,
        authority: *authority,
        dex_program: dex::ID,
    };
    let ix_data = crate::instruction::CloseSpotOpenOrders {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_account_delegate(
    master_account: &Pubkey,
    delegate: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = SetAccountDelegate {
        master_account: *master_account,
        delegate: *delegate,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetAccountDelegate {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_sub_account_delegate(
    sub_account: &Pubkey,
    delegate: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = SetSubAccountDelegate {
        sub_account: *sub_account,
        delegate: *delegate,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetSubAccountDelegate {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn deposit_funds(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    pool: &Pubkey,
    pool_node: &Pubkey,
    source_token_account: &Pubkey,
    token_vault: &Pubkey,
    token_mint: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = DepositFunds {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        pool: *pool,
        pool_node: *pool_node,
        source_token_account: *source_token_account,
        token_vault: *token_vault,
        token_mint: *token_mint,
        authority: *authority,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::DepositFunds { _amount: amount };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn withdraw_funds(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    pool: &Pubkey,
    pool_node: &Pubkey,
    destination_token_account: &Pubkey,
    token_vault: &Pubkey,
    vault_signer: &Pubkey,
    token_mint: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = WithdrawFunds {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        pool: *pool,
        pool_node: *pool_node,
        token_vault: *token_vault,
        vault_signer: *vault_signer,
        destination_token_account: *destination_token_account,
        token_mint: *token_mint,
        authority: *authority,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::WithdrawFunds { _amount: amount };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn new_futures_order(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    price_history: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
    args: NewDerivativeOrderArgs,
) -> Instruction {
    let accounts = NewFuturesOrder {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        price_history: *price_history,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        quote_pool_node: *quote_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::NewFuturesOrder { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn multiple_new_futures_orders(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    price_history: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
    args: Vec<NewDerivativeOrderArgs>,
) -> Instruction {
    let accounts = MultipleNewFuturesOrders {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        price_history: *price_history,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        quote_pool_node: *quote_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::MultipleNewFuturesOrders { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn cancel_futures_order(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
    args: CancelOrderArgs,
) -> Instruction {
    let accounts = CancelFuturesOrder {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        quote_pool_node: *quote_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::CancelFuturesOrder { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn cancel_futures_orders(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
    args: Vec<CancelOrderArgs>,
) -> Instruction {
    let accounts = CancelFuturesOrders {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        quote_pool_node: *quote_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::CancelFuturesOrders { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn settle_futures_funds(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = SettleFuturesFunds {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        quote_pool_node: *quote_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::SettleFuturesFunds {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn new_perp_order(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
    args: NewDerivativeOrderArgs,
) -> Instruction {
    let accounts = NewPerpOrder {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        quote_pool_node: *quote_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::NewPerpOrder { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn multiple_new_perp_orders(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
    args: Vec<NewDerivativeOrderArgs>,
) -> Instruction {
    let accounts = MultipleNewPerpOrders {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        quote_pool_node: *quote_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::MultipleNewPerpOrders { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn cancel_perp_order(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
    args: CancelOrderArgs,
) -> Instruction {
    let accounts = CancelPerpOrder {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        quote_pool_node: *quote_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::CancelPerpOrder { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn cancel_perp_orders(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
    args: Vec<CancelOrderArgs>,
) -> Instruction {
    let accounts = CancelPerpOrders {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        quote_pool_node: *quote_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::CancelPerpOrders { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn settle_perp_funds(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    quote_pool_node: &Pubkey,
) -> Instruction {
    let accounts = SettlePerpFunds {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        quote_pool_node: *quote_pool_node,
    };
    let ix_data = crate::instruction::SettlePerpFunds {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn settle_funding(
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    open_orders: &Pubkey,
    quote_pool_node: &Pubkey,
) -> Instruction {
    let accounts = SettleFunding {
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        quote_pool_node: *quote_pool_node,
    };
    let ix_data = crate::instruction::SettleFunding {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn new_spot_order(
    // cypher accounts
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    asset_pool_node: &Pubkey,
    quote_pool_node: &Pubkey,
    asset_mint: &Pubkey,
    asset_vault: &Pubkey,
    quote_vault: &Pubkey,
    vault_signer: &Pubkey,
    authority: &Pubkey,
    // dex accounts
    market: &Pubkey,
    open_orders: &Pubkey,
    event_queue: &Pubkey,
    request_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    coin_vault: &Pubkey,
    pc_vault: &Pubkey,
    dex_vault_signer: &Pubkey,
    args: NewSpotOrderArgs,
) -> Instruction {
    let accounts = NewSpotOrder {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        asset_pool_node: *asset_pool_node,
        quote_pool_node: *quote_pool_node,
        asset_mint: *asset_mint,
        asset_vault: *asset_vault,
        quote_vault: *quote_vault,
        vault_signer: *vault_signer,
        authority: *authority,
        NewSpotOrderdex: NewSpotOrderDex {
            market: *market,
            open_orders: *open_orders,
            event_queue: *event_queue,
            request_queue: *request_queue,
            bids: *bids,
            asks: *asks,
            coin_vault: *coin_vault,
            pc_vault: *pc_vault,
            vault_signer: *dex_vault_signer,
            rent: Rent::id(),
            token_program: token::ID,
            dex_program: dex::ID,
        },
    };
    let ix_data = crate::instruction::NewSpotOrder { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn cancel_spot_order(
    // cypher accounts
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    asset_pool_node: &Pubkey,
    quote_pool_node: &Pubkey,
    asset_mint: &Pubkey,
    asset_vault: &Pubkey,
    quote_vault: &Pubkey,
    authority: &Pubkey,
    // dex accounts
    market: &Pubkey,
    open_orders: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    coin_vault: &Pubkey,
    pc_vault: &Pubkey,
    dex_vault_signer: &Pubkey,
    args: CancelOrderArgs,
) -> Instruction {
    let accounts = CancelSpotOrder {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        asset_pool_node: *asset_pool_node,
        quote_pool_node: *quote_pool_node,
        asset_mint: *asset_mint,
        asset_vault: *asset_vault,
        quote_vault: *quote_vault,
        authority: *authority,
        CancelSpotOrderdex: CancelSpotOrderDex {
            market: *market,
            open_orders: *open_orders,
            event_queue: *event_queue,
            bids: *bids,
            asks: *asks,
            coin_vault: *coin_vault,
            pc_vault: *pc_vault,
            vault_signer: *dex_vault_signer,
            token_program: token::ID,
            dex_program: dex::ID,
        },
    };
    let ix_data = crate::instruction::CancelSpotOrder { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn settle_spot_funds(
    // cypher accounts
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    asset_pool_node: &Pubkey,
    quote_pool_node: &Pubkey,
    asset_mint: &Pubkey,
    asset_vault: &Pubkey,
    quote_vault: &Pubkey,
    authority: &Pubkey,
    // dex accounts
    market: &Pubkey,
    open_orders: &Pubkey,
    coin_vault: &Pubkey,
    pc_vault: &Pubkey,
    dex_vault_signer: &Pubkey,
) -> Instruction {
    let accounts = SettleSpotFunds {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        asset_pool_node: *asset_pool_node,
        quote_pool_node: *quote_pool_node,
        asset_mint: *asset_mint,
        asset_vault: *asset_vault,
        quote_vault: *quote_vault,
        authority: *authority,
        SettleSpotFundsdex: SettleSpotFundsDex {
            market: *market,
            open_orders: *open_orders,
            coin_vault: *coin_vault,
            pc_vault: *pc_vault,
            vault_signer: *dex_vault_signer,
            token_program: token::ID,
            dex_program: dex::ID,
        },
    };
    let ix_data = crate::instruction::SettleSpotFunds {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn update_token_index(
    cache_account: &Pubkey,
    pool: &Pubkey,
    pool_nodes: &[Pubkey],
) -> Instruction {
    let mut accounts = UpdateTokenIndex {
        cache_account: *cache_account,
        pool: *pool,
    }
    .to_account_metas(Some(false));
    accounts.extend(
        pool_nodes
            .iter()
            .map(|n| AccountMeta::new_readonly(*n, false)),
    );
    let ix_data = crate::instruction::UpdateTokenIndex {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn update_funding_rate(
    cache_account: &Pubkey,
    market: &Pubkey,
    orderbook: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
) -> Instruction {
    let accounts = UpdateFundingRate {
        cache_account: *cache_account,
        market: *market,
        orderbook: *orderbook,
        bids: *bids,
        asks: *asks,
    };
    let ix_data = crate::instruction::UpdateFundingRate {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn consume_futures_events(
    clearing: &Pubkey,
    market: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    open_orders: &[Pubkey],
    limit: u16,
) -> Instruction {
    let mut accounts = ConsumeFuturesEvents {
        clearing: *clearing,
        market: *market,
        orderbook: *orderbook,
        event_queue: *event_queue,
    }
    .to_account_metas(Some(false));

    accounts.extend(open_orders.iter().map(|pk| AccountMeta::new(*pk, false)));

    let ix_data = crate::instruction::ConsumeFuturesEvents { _limit: limit };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn consume_perp_events(
    clearing: &Pubkey,
    market: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    open_orders: &[Pubkey],
    limit: u16,
) -> Instruction {
    let mut accounts = ConsumePerpEvents {
        clearing: *clearing,
        market: *market,
        orderbook: *orderbook,
        event_queue: *event_queue,
    }
    .to_account_metas(Some(false));

    accounts.extend(open_orders.iter().map(|pk| AccountMeta::new(*pk, false)));

    let ix_data = crate::instruction::ConsumePerpEvents { _limit: limit };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn update_account_margin(
    cache_account: &Pubkey,
    master_account: &Pubkey,
    signer: &Pubkey,
    sub_accounts: &[Pubkey],
) -> Instruction {
    let mut accounts = UpdateAccountMargin {
        cache_account: *cache_account,
        master_account: *master_account,
        signer: *signer,
    }
    .to_account_metas(Some(false));

    accounts.extend(sub_accounts.iter().map(|pk| AccountMeta::new(*pk, false)));

    let ix_data = crate::instruction::UpdateAccountMargin {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn transfer_between_sub_accounts(
    clearing: &Pubkey,
    cache_account: &Pubkey,
    master_account: &Pubkey,
    from_sub_account: &Pubkey,
    to_sub_account: &Pubkey,
    asset_mint: &Pubkey,
    asset_pool_node: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = TransferBetweenSubAccounts {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        from_sub_account: *from_sub_account,
        to_sub_account: *to_sub_account,
        asset_mint: *asset_mint,
        asset_pool_node: *asset_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::TransferBetweenSubAccounts { _amount: amount };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn deposit_deliverable(
    market: &Pubkey,
    pool: &Pubkey,
    pool_node: &Pubkey,
    token_mint: &Pubkey,
    token_vault: &Pubkey,
    source_token_account: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = DepositDeliverable {
        market: *market,
        pool: *pool,
        pool_node: *pool_node,
        token_mint: *token_mint,
        token_vault: *token_vault,
        source_token_account: *source_token_account,
        authority: *authority,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::DepositDeliverable { _amount: amount };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn settle_position(
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    quote_pool_node: &Pubkey,
) -> Instruction {
    let accounts = SettlePosition {
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        quote_pool_node: *quote_pool_node,
    };
    let ix_data = crate::instruction::SettlePosition {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn settle_position_with_delivery(
    cache_account: &Pubkey,
    master_account: &Pubkey,
    sub_account: &Pubkey,
    market: &Pubkey,
    quote_pool_node: &Pubkey,
    underlying_pool_node: &Pubkey,
) -> Instruction {
    let accounts = SettlePositionWithDelivery {
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        quote_pool_node: *quote_pool_node,
        underlying_pool_node: *underlying_pool_node,
    };
    let ix_data = crate::instruction::SettlePositionWithDelivery {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn claim_ido_proceeds(
    market: &Pubkey,
    quote_pool_node: &Pubkey,
    quote_vault: &Pubkey,
    vault_signer: &Pubkey,
    ido_authority: &Pubkey,
    destination_token_account: &Pubkey,
) -> Instruction {
    let accounts = ClaimIdoProceeds {
        market: *market,
        quote_pool_node: *quote_pool_node,
        quote_vault: *quote_vault,
        vault_signer: *vault_signer,
        destination_token_account: *destination_token_account,
        ido_authority: *ido_authority,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::ClaimIdoProceeds {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn roll_market_expiry(
    cache_account: &Pubkey,
    clearing: &Pubkey,
    market: &Pubkey,
    authority: &Pubkey,
    new_expiration: u64,
) -> Instruction {
    let accounts = RollMarketExpiry {
        cache_account: *cache_account,
        clearing: *clearing,
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::RollMarketExpiry {
        _expiration_ts: new_expiration,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn close_pool(
    pool: &Pubkey,
    oracle_products: &Pubkey,
    rent_destination: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = ClosePool {
        pool: *pool,
        oracle_products: *oracle_products,
        rent_destination: *rent_destination,
        authority: *authority,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::ClosePool {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn close_pool_node(
    pool: &Pubkey,
    pool_node: &Pubkey,
    token_mint: &Pubkey,
    token_vault: &Pubkey,
    vault_signer: &Pubkey,
    rent_destination: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = ClosePoolNode {
        pool: *pool,
        pool_node: *pool_node,
        token_mint: *token_mint,
        token_vault: *token_vault,
        vault_signer: *vault_signer,
        rent_destination: *rent_destination,
        authority: *authority,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::ClosePoolNode {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn close_clearing(
    clearing: &Pubkey,
    rent_destination: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = CloseClearing {
        clearing: *clearing,
        rent_destination: *rent_destination,
        authority: *authority,
    };
    let ix_data = crate::instruction::CloseClearing {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn close_futures_market(
    market: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    oracle_products: &Pubkey,
    price_history: &Pubkey,
    rent_destination: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = CloseFuturesMarket {
        market: *market,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        oracle_products: *oracle_products,
        price_history: *price_history,
        rent_destination: *rent_destination,
        authority: *authority,
    };
    let ix_data = crate::instruction::CloseFuturesMarket {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn close_perp_market(
    market: &Pubkey,
    orderbook: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    oracle_products: &Pubkey,
    rent_destination: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = ClosePerpMarket {
        market: *market,
        orderbook: *orderbook,
        event_queue: *event_queue,
        bids: *bids,
        asks: *asks,
        oracle_products: *oracle_products,
        rent_destination: *rent_destination,
        authority: *authority,
    };
    let ix_data = crate::instruction::ClosePerpMarket {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn update_market_expiration(
    clearing: &Pubkey,
    market: &Pubkey,
    authority: &Pubkey,
    new_expiry: u64,
) -> Instruction {
    let accounts = UpdateMarketExpiration {
        clearing: *clearing,
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::UpdateMarketExpiration {
        _expiration_ts: new_expiry,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn close_cache_account(
    cache_account: &Pubkey,
    authority: &Pubkey,
    rent_destination: &Pubkey,
) -> Instruction {
    let accounts = CloseCacheAccount {
        cache_account: *cache_account,
        authority: *authority,
        rent_destination: *rent_destination,
    };
    let ix_data = crate::instruction::CloseCacheAccount {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn close_oracle_products(
    cache_account: &Pubkey,
    oracle_products: &Pubkey,
    authority: &Pubkey,
    rent_destination: &Pubkey,
) -> Instruction {
    let accounts = CloseOracleProducts {
        cache_account: *cache_account,
        oracle_products: *oracle_products,
        authority: *authority,
        rent_destination: *rent_destination,
    };
    let ix_data = crate::instruction::CloseOracleProducts {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn close_account(
    account: &Pubkey,
    authority: &Pubkey,
    rent_destination: &Pubkey,
) -> Instruction {
    let accounts = CloseAccount {
        account: *account,
        authority: *authority,
        rent_destination: *rent_destination,
    };
    let ix_data = crate::instruction::CloseAccount {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn close_sub_account(
    account: &Pubkey,
    sub_account: &Pubkey,
    authority: &Pubkey,
    rent_destination: &Pubkey,
) -> Instruction {
    let accounts = CloseSubAccount {
        account: *account,
        sub_account: *sub_account,
        authority: *authority,
        rent_destination: *rent_destination,
    };
    let ix_data = crate::instruction::CloseSubAccount {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_pool_status(pool: &Pubkey, authority: &Pubkey, status: OperatingStatus) -> Instruction {
    let accounts = SetPoolStatus {
        pool: *pool,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPoolStatus { _status: status };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_pool_node_status(
    pool_node: &Pubkey,
    authority: &Pubkey,
    status: OperatingStatus,
) -> Instruction {
    let accounts = SetPoolNodeStatus {
        pool_node: *pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPoolNodeStatus { _status: status };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_perpetual_market_status(
    market: &Pubkey,
    authority: &Pubkey,
    status: OperatingStatus,
) -> Instruction {
    let accounts = SetPerpetualMarketStatus {
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPerpetualMarketStatus { _status: status };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_futures_market_status(
    market: &Pubkey,
    authority: &Pubkey,
    status: OperatingStatus,
) -> Instruction {
    let accounts = SetFuturesMarketStatus {
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetFuturesMarketStatus { _status: status };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_clearing_fee_tiers(
    clearing: &Pubkey,
    authority: &Pubkey,
    fee_tiers: &[FeeTierArgs],
) -> Instruction {
    let accounts = SetClearingFeeTiers {
        clearing: *clearing,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetClearingFeeTiers {
        _fee_tiers: fee_tiers.to_vec(),
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_clearing_fee_mint(
    clearing: &Pubkey,
    authority: &Pubkey,
    fee_mint: &Pubkey,
) -> Instruction {
    let accounts = SetClearingFeeMint {
        clearing: *clearing,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetClearingFeeMint {
        _fee_mint: *fee_mint,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_clearing_authority(
    clearing: &Pubkey,
    authority: &Pubkey,
    new_authority: &Pubkey,
) -> Instruction {
    let accounts = SetClearingAuthority {
        clearing: *clearing,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetClearingAuthority {
        _new_authority: *new_authority,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_perpetual_market_authority(
    market: &Pubkey,
    authority: &Pubkey,
    new_authority: &Pubkey,
) -> Instruction {
    let accounts = SetPerpetualMarketAuthority {
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPerpetualMarketAuthority {
        _new_authority: *new_authority,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_perpetual_market_params(
    cache: &Pubkey,
    market: &Pubkey,
    authority: &Pubkey,
    impact_quantity: Option<u64>,
    max_base_order_size: Option<u64>,
    max_quote_order_size: Option<u64>,
    init_asset_weight: Option<u8>,
    maint_asset_weight: Option<u8>,
    init_liab_weight: Option<u8>,
    maint_liab_weight: Option<u8>,
) -> Instruction {
    let accounts = SetPerpetualMarketParams {
        cache: *cache,
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPerpetualMarketParams {
        _impact_quantity: impact_quantity,
        _max_base_order_size: max_base_order_size,
        _max_quote_order_size: max_quote_order_size,
        _init_asset_weight: init_asset_weight,
        _maint_asset_weight: maint_asset_weight,
        _init_liab_weight: init_liab_weight,
        _maint_liab_weight: maint_liab_weight,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_perpetual_market_liquidity_mining_info(
    market: &Pubkey,
    authority: &Pubkey,
    args: LiquidityMiningArgs,
) -> Instruction {
    let accounts = SetPerpetualMarketLiquidityMiningInfo {
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPerpetualMarketLiquidityMiningInfo { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_futures_market_authority(
    market: &Pubkey,
    authority: &Pubkey,
    new_authority: &Pubkey,
) -> Instruction {
    let accounts = SetFuturesMarketAuthority {
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetFuturesMarketAuthority {
        _new_authority: *new_authority,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_futures_market_params(
    cache: &Pubkey,
    market: &Pubkey,
    authority: &Pubkey,
    max_base_order_size: Option<u64>,
    max_quote_order_size: Option<u64>,
    init_asset_weight: Option<u8>,
    maint_asset_weight: Option<u8>,
    init_liab_weight: Option<u8>,
    maint_liab_weight: Option<u8>,
) -> Instruction {
    let accounts = SetFuturesMarketParams {
        cache: *cache,
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetFuturesMarketParams {
        _max_base_order_size: max_base_order_size,
        _max_quote_order_size: max_quote_order_size,
        _init_asset_weight: init_asset_weight,
        _maint_asset_weight: maint_asset_weight,
        _init_liab_weight: init_liab_weight,
        _maint_liab_weight: maint_liab_weight,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_futures_market_liquidity_mining_info(
    market: &Pubkey,
    authority: &Pubkey,
    args: LiquidityMiningArgs,
) -> Instruction {
    let accounts = SetFuturesMarketLiquidityMiningInfo {
        market: *market,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetFuturesMarketLiquidityMiningInfo { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_pool_authority(
    pool: &Pubkey,
    authority: &Pubkey,
    new_authority: &Pubkey,
) -> Instruction {
    let accounts = SetPoolAuthority {
        pool: *pool,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPoolAuthority {
        _new_authority: *new_authority,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_pool_node_authority(
    pool_node: &Pubkey,
    authority: &Pubkey,
    new_authority: &Pubkey,
) -> Instruction {
    let accounts = SetPoolNodeAuthority {
        pool_node: *pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPoolNodeAuthority {
        _new_authority: *new_authority,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_cache_authority(
    cache_account: &Pubkey,
    authority: &Pubkey,
    new_authority: &Pubkey,
) -> Instruction {
    let accounts = SetCacheAuthority {
        cache_account: *cache_account,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetCacheAuthority {
        _new_authority: *new_authority,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_pool_dex_market(
    clearing: &Pubkey,
    pool: &Pubkey,
    dex_market: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = SetPoolDexMarket {
        clearing: *clearing,
        pool: *pool,
        dex_market: *dex_market,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPoolDexMarket {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn set_pool_params(
    cache: &Pubkey,
    pool: &Pubkey,
    authority: &Pubkey,
    init_asset_weight: Option<u8>,
    maint_asset_weight: Option<u8>,
    init_liab_weight: Option<u8>,
    maint_liab_weight: Option<u8>,
    optimal_apr: Option<u16>,
    optimal_util: Option<u16>,
    max_apr: Option<u16>,
) -> Instruction {
    let accounts = SetPoolParams {
        cache: *cache,
        pool: *pool,
        authority: *authority,
    };
    let ix_data = crate::instruction::SetPoolParams {
        _init_asset_weight: init_asset_weight,
        _maint_asset_weight: maint_asset_weight,
        _init_liab_weight: init_liab_weight,
        _maint_liab_weight: maint_liab_weight,
        _optimal_apr: optimal_apr,
        _optimal_util: optimal_util,
        _max_apr: max_apr,
    };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn sweep_market_fees(
    clearing: &Pubkey,
    market: &Pubkey,
    quote_pool_node: &Pubkey,
    quote_vault: &Pubkey,
    destination_token_account: &Pubkey,
    vault_signer: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = SweepMarketFees {
        clearing: *clearing,
        market: *market,
        quote_pool_node: *quote_pool_node,
        quote_vault: *quote_vault,
        destination_token_account: *destination_token_account,
        vault_signer: *vault_signer,
        authority: *authority,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::SweepMarketFees {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn sweep_pool_fees(
    pool: &Pubkey,
    quote_pool_node: &Pubkey,
    quote_vault: &Pubkey,
    destination_token_account: &Pubkey,
    vault_signer: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = SweepPoolFees {
        token_pool: *pool,
        quote_pool_node: *quote_pool_node,
        quote_vault: *quote_vault,
        destination_token_account: *destination_token_account,
        vault_signer: *vault_signer,
        authority: *authority,
        token_program: token::ID,
    };
    let ix_data = crate::instruction::SweepPoolFees {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn liquidate_spot_position(
    cache_account: &Pubkey,
    liqor_clearing: &Pubkey,
    liqor_account: &Pubkey,
    liqor_sub_account: &Pubkey,
    liqee_clearing: &Pubkey,
    liqee_account: &Pubkey,
    liqee_sub_account: &Pubkey,
    asset_mint: &Pubkey,
    asset_pool_node: &Pubkey,
    liability_mint: &Pubkey,
    liability_pool: &Pubkey,
    liability_pool_node: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = LiquidateSpotPosition {
        cache_account: *cache_account,
        liqor_clearing: *liqor_clearing,
        liqor_account: *liqor_account,
        liqor_sub_account: *liqor_sub_account,
        liqee_clearing: *liqee_clearing,
        liqee_account: *liqee_account,
        liqee_sub_account: *liqee_sub_account,
        asset_mint: *asset_mint,
        asset_pool_node: *asset_pool_node,
        liability_mint: *liability_mint,
        liability_pool: *liability_pool,
        liability_pool_node: *liability_pool_node,
        authority: *authority,
    };
    let ix_data = crate::instruction::LiquidateSpotPosition {};
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn liquidate_perp_position(
    cache_account: &Pubkey,
    liqor_clearing: &Pubkey,
    liqor_account: &Pubkey,
    liqor_sub_account: &Pubkey,
    liqee_clearing: &Pubkey,
    liqee_account: &Pubkey,
    liqee_sub_account: &Pubkey,
    asset: &Pubkey,
    asset_market: &Pubkey,
    liability: &Pubkey,
    liability_market: &Pubkey,
    quote_pool: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = LiquidatePerpPosition {
        cache_account: *cache_account,
        liqor_clearing: *liqor_clearing,
        liqor_account: *liqor_account,
        liqor_sub_account: *liqor_sub_account,
        liqee_clearing: *liqee_clearing,
        liqee_account: *liqee_account,
        liqee_sub_account: *liqee_sub_account,
        authority: *authority,
    };
    let mut accounts = accounts.to_account_metas(Some(false));

    let ix_data = crate::instruction::LiquidatePerpPosition {
        _asset: *asset,
        _liability: *liability,
    };

    extend_derivative_liquidation_accounts(
        &mut accounts,
        asset,
        asset_market,
        liability,
        liability_market,
        quote_pool,
        quote_pool_node,
    )?;

    Ok(Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    })
}

pub fn liquidate_futures_position(
    cache_account: &Pubkey,
    liqor_clearing: &Pubkey,
    liqor_account: &Pubkey,
    liqor_sub_account: &Pubkey,
    liqee_clearing: &Pubkey,
    liqee_account: &Pubkey,
    liqee_sub_account: &Pubkey,
    asset: &Pubkey,
    asset_market: &Pubkey,
    liability: &Pubkey,
    liability_market: &Pubkey,
    quote_pool: &Pubkey,
    quote_pool_node: &Pubkey,
    authority: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = LiquidateFuturesPosition {
        cache_account: *cache_account,
        liqor_clearing: *liqor_clearing,
        liqor_account: *liqor_account,
        liqor_sub_account: *liqor_sub_account,
        liqee_clearing: *liqee_clearing,
        liqee_account: *liqee_account,
        liqee_sub_account: *liqee_sub_account,
        authority: *authority,
    };
    let mut accounts = accounts.to_account_metas(Some(false));

    let ix_data = crate::instruction::LiquidateFuturesPosition {
        _asset: *asset,
        _liability: *liability,
    };

    extend_derivative_liquidation_accounts(
        &mut accounts,
        asset,
        asset_market,
        liability,
        liability_market,
        quote_pool,
        quote_pool_node,
    )?;

    Ok(Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    })
}

fn extend_derivative_liquidation_accounts(
    accounts: &mut Vec<AccountMeta>,
    asset: &Pubkey,
    asset_market: &Pubkey,
    liability: &Pubkey,
    liability_market: &Pubkey,
    quote_pool: &Pubkey,
    quote_pool_node: &Pubkey,
) -> Result<(), ProgramError> {
    if *asset == quote_mint::id() {
        if *liability_market == Pubkey::default() {
            return Err(ProgramError::InvalidArgument);
        }
        if *quote_pool == Pubkey::default() {
            return Err(ProgramError::InvalidArgument);
        }
        if *quote_pool_node == Pubkey::default() {
            return Err(ProgramError::InvalidArgument);
        }
        accounts.extend(vec![
            AccountMeta::new(*liability_market, false),
            AccountMeta::new(*quote_pool, false),
            AccountMeta::new(*quote_pool_node, false),
        ]);
        return Ok(());
    }

    if *liability == quote_mint::id() {
        if *asset_market == Pubkey::default() {
            return Err(ProgramError::InvalidArgument);
        }
        if *quote_pool == Pubkey::default() {
            return Err(ProgramError::InvalidArgument);
        }
        if *quote_pool_node == Pubkey::default() {
            return Err(ProgramError::InvalidArgument);
        }
        accounts.extend(vec![
            AccountMeta::new(*asset_market, false),
            AccountMeta::new(*quote_pool, false),
            AccountMeta::new(*quote_pool_node, false),
        ]);
        return Ok(());
    }

    if *asset_market == Pubkey::default() {
        return Err(ProgramError::InvalidArgument);
    }
    if *liability_market == Pubkey::default() {
        return Err(ProgramError::InvalidArgument);
    }
    if *quote_pool == Pubkey::default() {
        return Err(ProgramError::InvalidArgument);
    }
    if *quote_pool_node == Pubkey::default() {
        return Err(ProgramError::InvalidArgument);
    }
    accounts.extend(vec![
        AccountMeta::new(*asset_market, false),
        AccountMeta::new(*liability_market, false),
        AccountMeta::new(*quote_pool, false),
        AccountMeta::new(*quote_pool_node, false),
    ]);
    Ok(())
}
