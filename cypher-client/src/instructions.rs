use anchor_lang::{
    prelude::{AccountMeta, Pubkey, Rent},
    solana_program::{instruction::Instruction, sysvar::SysvarId},
    system_program, InstructionData, ToAccountMetas,
};
use anchor_spl::token;

use crate::{
    accounts::{
        CacheOraclePrices, CancelFuturesOrder, CancelPerpOrder, CancelSpotOrder,
        CancelSpotOrderDex, ClaimIdoProceeds, CloseCacheAccount, CloseClearing, CloseFuturesMarket,
        CloseOracleProducts, ClosePerpMarket, ClosePool, CloseSpotOpenOrders, ConsumeFuturesEvents,
        ConsumePerpEvents, CreateAccount, CreateFuturesMarket, CreateOracleProducts,
        CreateOrdersAccount, CreatePerpMarket, CreatePool, CreatePrivateClearing,
        CreatePublicClearing, CreateSubAccount, CreateWhitelist, CreateWhitelistedAccount,
        DepositDeliverable, DepositFunds, InitCacheAccount, InitSpotOpenOrders, NewFuturesOrder,
        NewPerpOrder, NewSpotOrder, NewSpotOrderDex, RollMarketExpiry, SetAccountDelegate,
        SetOracleProducts, SetSubAccountDelegate, SettleFuturesFunds, SettlePerpFunds,
        SettlePosition, SettlePositionWithDelivery, SettleSpotFunds, SettleSpotFundsDex,
        TransferBetweenSubAccounts, UpdateAccountMargin, UpdateFundingRate, UpdateMarketExpiration,
        UpdateTokenIndex, WithdrawFunds,
    },
    constants::SUB_ACCOUNT_ALIAS_LEN,
    CancelOrderArgs, CreateClearingArgs, CreateFuturesMarketArgs, CreateOracleProductsArgs,
    CreatePerpetualMarketArgs, CreatePoolArgs, NewDerivativeOrderArgs, NewSpotOrderArgs,
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

pub fn create_subaccount(
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
    quote_mint: &Pubkey,
    quote_vault: &Pubkey,
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
        quote_mint: *quote_mint,
        quote_vault: *quote_vault,
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
    quote_mint: &Pubkey,
    quote_vault: &Pubkey,
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
        quote_mint: *quote_mint,
        quote_vault: *quote_vault,
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
    token_mint: &Pubkey,
    token_vault: &Pubkey,
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
        token_mint: *token_mint,
        token_vault: *token_vault,
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
    clearing: &Pubkey,
    oracle_products: &Pubkey,
    payer: &Pubkey,
    authority: &Pubkey,
    product_accounts: Option<&[Pubkey]>,
    args: CreateOracleProductsArgs,
) -> Instruction {
    let mut accounts = CreateOracleProducts {
        clearing: *clearing,
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
        dex_program: anchor_spl::dex::ID,
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

pub fn cache_oracle_prices(
    cache_account: &Pubkey,
    oracle_products: &Pubkey,
    price_accounts: &[Pubkey],
    pool: &Option<Pubkey>,
    futures_market: &Option<Pubkey>,
    perp_market: &Option<Pubkey>,
) -> Instruction {
    let mut accounts = CacheOraclePrices {
        cache_account: *cache_account,
        oracle_products: *oracle_products,
    }
    .to_account_metas(Some(false));
    if pool.is_some() {
        accounts.push(AccountMeta::new_readonly(pool.unwrap(), false));
    }
    if futures_market.is_some() {
        accounts.push(AccountMeta::new_readonly(futures_market.unwrap(), false));
    }
    if perp_market.is_some() {
        accounts.push(AccountMeta::new_readonly(perp_market.unwrap(), false));
    }
    accounts.extend(
        price_accounts
            .iter()
            .map(|p| AccountMeta::new_readonly(*p, false)),
    );
    let ix_data = crate::instruction::CacheOraclePrices {};
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
        dex_program: anchor_spl::dex::ID,
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
    destination_token_account: &Pubkey,
    token_vault: &Pubkey,
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
        token_vault: *token_vault,
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
    quote_pool: &Pubkey,
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
        quote_pool: *quote_pool,
        authority: *authority,
    };
    let ix_data = crate::instruction::NewFuturesOrder { _args: args };
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
    quote_pool: &Pubkey,
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
        quote_pool: *quote_pool,
        authority: *authority,
    };
    let ix_data = crate::instruction::CancelFuturesOrder { _args: args };
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
    quote_pool: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = SettleFuturesFunds {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        quote_pool: *quote_pool,
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
    quote_pool: &Pubkey,
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
        quote_pool: *quote_pool,
        authority: *authority,
    };
    let ix_data = crate::instruction::NewPerpOrder { _args: args };
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
    quote_pool: &Pubkey,
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
        quote_pool: *quote_pool,
        authority: *authority,
    };
    let ix_data = crate::instruction::CancelPerpOrder { _args: args };
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
    quote_pool: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = SettlePerpFunds {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        open_orders: *open_orders,
        quote_pool: *quote_pool,
        authority: *authority,
    };
    let ix_data = crate::instruction::SettlePerpFunds {};
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
    asset_pool: &Pubkey,
    quote_pool: &Pubkey,
    asset_mint: &Pubkey,
    quote_mint: &Pubkey,
    asset_vault: &Pubkey,
    quote_vault: &Pubkey,
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
    vault_signer: &Pubkey,
    args: NewSpotOrderArgs,
) -> Instruction {
    let accounts = NewSpotOrder {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        asset_pool: *asset_pool,
        quote_pool: *quote_pool,
        asset_mint: *asset_mint,
        quote_mint: *quote_mint,
        asset_vault: *asset_vault,
        quote_vault: *quote_vault,
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
            vault_signer: *vault_signer,
            rent: Rent::id(),
            token_program: token::ID,
            dex_program: anchor_spl::dex::ID,
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
    asset_pool: &Pubkey,
    quote_pool: &Pubkey,
    asset_mint: &Pubkey,
    quote_mint: &Pubkey,
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
    vault_signer: &Pubkey,
    args: CancelOrderArgs,
) -> Instruction {
    let accounts = CancelSpotOrder {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        asset_pool: *asset_pool,
        quote_pool: *quote_pool,
        asset_mint: *asset_mint,
        quote_mint: *quote_mint,
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
            vault_signer: *vault_signer,
            token_program: token::ID,
            dex_program: anchor_spl::dex::ID,
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
    asset_pool: &Pubkey,
    quote_pool: &Pubkey,
    asset_mint: &Pubkey,
    quote_mint: &Pubkey,
    asset_vault: &Pubkey,
    quote_vault: &Pubkey,
    authority: &Pubkey,
    // dex accounts
    market: &Pubkey,
    open_orders: &Pubkey,
    coin_vault: &Pubkey,
    pc_vault: &Pubkey,
    vault_signer: &Pubkey,
    args: CancelOrderArgs,
) -> Instruction {
    let accounts = SettleSpotFunds {
        clearing: *clearing,
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        asset_pool: *asset_pool,
        quote_pool: *quote_pool,
        asset_mint: *asset_mint,
        quote_mint: *quote_mint,
        asset_vault: *asset_vault,
        quote_vault: *quote_vault,
        authority: *authority,
        SettleSpotFundsdex: SettleSpotFundsDex {
            market: *market,
            open_orders: *open_orders,
            coin_vault: *coin_vault,
            pc_vault: *pc_vault,
            vault_signer: *vault_signer,
            token_program: token::ID,
            dex_program: anchor_spl::dex::ID,
        },
    };
    let ix_data = crate::instruction::CancelSpotOrder { _args: args };
    Instruction {
        program_id: crate::id(),
        accounts: accounts.to_account_metas(Some(false)),
        data: ix_data.data(),
    }
}

pub fn update_token_index(pool: &Pubkey) -> Instruction {
    let accounts = UpdateTokenIndex { pool: *pool };
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
    asset_pool: &Pubkey,
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
        asset_pool: *asset_pool,
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
    token_mint: &Pubkey,
    token_vault: &Pubkey,
    source_token_account: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = DepositDeliverable {
        market: *market,
        pool: *pool,
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
    quote_pool: &Pubkey,
) -> Instruction {
    let accounts = SettlePosition {
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        quote_pool: *quote_pool,
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
    quote_pool: &Pubkey,
    underlying_mint: &Pubkey,
    underlying_pool: &Pubkey,
) -> Instruction {
    let accounts = SettlePositionWithDelivery {
        cache_account: *cache_account,
        master_account: *master_account,
        sub_account: *sub_account,
        market: *market,
        quote_pool: *quote_pool,
        underlying_mint: *underlying_mint,
        underlying_pool: *underlying_pool,
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
    token_mint: &Pubkey,
    token_pool: &Pubkey,
    token_vault: &Pubkey,
    ido_authority: &Pubkey,
    destination_token_account: &Pubkey,
) -> Instruction {
    let accounts = ClaimIdoProceeds {
        market: *market,
        token_mint: *token_mint,
        token_pool: *token_pool,
        token_vault: *token_vault,
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
    token_mint: &Pubkey,
    token_vault: &Pubkey,
    rent_destination: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let accounts = ClosePool {
        pool: *pool,
        oracle_products: *oracle_products,
        token_mint: *token_mint,
        token_vault: *token_vault,
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
