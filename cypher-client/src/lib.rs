#![allow(clippy::too_many_arguments)]
pub mod aob;
pub mod constants;
pub mod instructions;
pub mod serum;
pub mod utils;

use agnostic_orderbook::state::Side as AobSide;
use anchor_lang::prelude::*;
use anchor_spl::dex::serum_dex::matching::Side as DexSide;
use bonfida_utils::fp_math::fp32_mul_floor;
use constants::{INV_ONE_HUNDRED_FIXED, QUOTE_TOKEN_IDX};
use fixed::types::I80F48;
use std::{mem::take, ops::Mul};
use utils::adjust_decimals;

use crate::constants::TOKENS_MAX_CNT;

anchor_gen::generate_cpi_interface!(
    idl_path = "idl.json",
    zero_copy(
        Clearing,
        ClearingConfig,
        FeeTier,
        Whitelist,
        CypherAccount,
        CypherSubAccount,
        SubAccountCache,
        OpenOrdersCache,
        SpotPosition,
        DerivativePosition,
        PositionSlot,
        AgnosticMarket,
        FuturesMarket,
        PerpetualMarket,
        MarketConfig,
        Pool,
        PoolNode,
        NodeInfo,
        PoolConfig,
        OpenOrder,
        OrdersAccount,
        PriceHistory,
        CallBackInfo,
        CacheAccount,
        Cache,
    )
);

#[cfg(feature = "mainnet-beta")]
declare_id!("CYPH3o83JX6jY6NkbproSpdmQ5VWJtxjfJ5P8veyYVu3");
#[cfg(not(feature = "mainnet-beta"))]
declare_id!("9i1FSiiXcLSLPfeWcBMaLa19ueQ2zZopzHvw4s7hT7ty");

pub mod quote_mint {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("GE2GoxjfHo9uPJGDxwVifPFomBybhsh4m5SMqaw7vPBw");
}

pub mod pyth {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("gSbePebfvPy7tRqimPoVecS2UsBvYv46ynrzWocc92s");
}

pub mod pyth_quote_product {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("8GWTTbNiXdmyZREXbjsZBmCRuzdPrW55dnZGDkTRjWvb");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("6NpdXrQEpmDZ3jZKmM2rhdmkd3H6QAk23j2x8bkXcHKA");
}

pub mod pyth_quote_price {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7");
}

pub mod dex {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("EoTcMgcDRTJVZDMZWBoU6rhYHZfkNTVEAfz3uUJRcYGj");
}

pub mod cache_account {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("6x5U4c41tfUYGEbTXofFiHcfyx3rqJZsT4emrLisNGGL");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("9j2BAs64tYjQdaHsMbFnY4VKUnsLMTc8vrXpXcvP6ujz");
}

pub mod wrapped_sol {
    use anchor_lang::declare_id;
    declare_id!("So11111111111111111111111111111111111111112");
}

pub mod cypher_token {
    use anchor_lang::declare_id;
    declare_id!("CYPHK4sZe7A4tdgTgLSotkkEzadtxqKu5JjuvaQRkYah");
}

impl From<DexSide> for Side {
    fn from(side: DexSide) -> Self {
        match side {
            DexSide::Ask => Side::Ask,
            DexSide::Bid => Side::Bid,
        }
    }
}

impl From<AobSide> for Side {
    fn from(side: AobSide) -> Self {
        match side {
            AobSide::Ask => Side::Ask,
            AobSide::Bid => Side::Bid,
        }
    }
}

impl ToString for Side {
    fn to_string(&self) -> String {
        match self {
            Side::Bid => "Bid".to_string(),
            Side::Ask => "Ask".to_string(),
        }
    }
}

impl PartialEq for PositionSlot {
    fn eq(&self, other: &Self) -> bool {
        self.derivative == other.derivative && self.spot == other.spot
    }
}

impl PartialEq for SpotPosition {
    fn eq(&self, other: &Self) -> bool {
        self.cache_index == other.cache_index
            && self.open_orders_cache == other.open_orders_cache
            && self.position == other.position
            && self.token_mint == other.token_mint
    }
}

impl PartialEq for DerivativePosition {
    fn eq(&self, other: &Self) -> bool {
        self.cache_index == other.cache_index
            && self.open_orders_cache == other.open_orders_cache
            && self.base_position == other.base_position
            && self.market == other.market
            && self.long_funding_settled == other.long_funding_settled
            && self.short_funding_settled == other.short_funding_settled
            && self.market_type == other.market_type
    }
}

impl PartialEq for OpenOrdersCache {
    fn eq(&self, other: &Self) -> bool {
        self.coin_free == other.coin_free
            && self.coin_total == other.coin_total
            && self.pc_free == other.pc_free
            && self.pc_total == other.pc_total
    }
}

impl PartialEq for OperatingStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (OperatingStatus::Active, OperatingStatus::Active) => true,
            (OperatingStatus::Active, OperatingStatus::ReduceOnly) => false,
            (OperatingStatus::Active, OperatingStatus::CancelOnly) => false,
            (OperatingStatus::Active, OperatingStatus::Halted) => false,
            (OperatingStatus::ReduceOnly, OperatingStatus::Active) => false,
            (OperatingStatus::ReduceOnly, OperatingStatus::ReduceOnly) => true,
            (OperatingStatus::ReduceOnly, OperatingStatus::CancelOnly) => false,
            (OperatingStatus::ReduceOnly, OperatingStatus::Halted) => false,
            (OperatingStatus::CancelOnly, OperatingStatus::Active) => false,
            (OperatingStatus::CancelOnly, OperatingStatus::ReduceOnly) => false,
            (OperatingStatus::CancelOnly, OperatingStatus::CancelOnly) => true,
            (OperatingStatus::CancelOnly, OperatingStatus::Halted) => false,
            (OperatingStatus::Halted, OperatingStatus::Active) => false,
            (OperatingStatus::Halted, OperatingStatus::ReduceOnly) => false,
            (OperatingStatus::Halted, OperatingStatus::CancelOnly) => false,
            (OperatingStatus::Halted, OperatingStatus::Halted) => true,
        }
    }
}

impl PartialEq for ClearingType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ClearingType::Public, ClearingType::Public) => true,
            (ClearingType::Public, ClearingType::Private) => false,
            (ClearingType::Private, ClearingType::Public) => false,
            (ClearingType::Private, ClearingType::Private) => true,
        }
    }
}

impl PartialEq for ProductsType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ProductsType::Stub, ProductsType::Stub) => true,
            (ProductsType::Stub, ProductsType::Pyth) => false,
            (ProductsType::Stub, ProductsType::Switchboard) => false,
            (ProductsType::Stub, ProductsType::Chainlink) => false,
            (ProductsType::Pyth, ProductsType::Pyth) => true,
            (ProductsType::Pyth, ProductsType::Stub) => false,
            (ProductsType::Pyth, ProductsType::Switchboard) => false,
            (ProductsType::Pyth, ProductsType::Chainlink) => false,
            (ProductsType::Switchboard, ProductsType::Switchboard) => true,
            (ProductsType::Switchboard, ProductsType::Stub) => false,
            (ProductsType::Switchboard, ProductsType::Pyth) => false,
            (ProductsType::Switchboard, ProductsType::Chainlink) => false,
            (ProductsType::Chainlink, ProductsType::Switchboard) => false,
            (ProductsType::Chainlink, ProductsType::Chainlink) => true,
            (ProductsType::Chainlink, ProductsType::Stub) => false,
            (ProductsType::Chainlink, ProductsType::Pyth) => false,
        }
    }
}

impl PartialEq for SubAccountMargining {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SubAccountMargining::Cross, SubAccountMargining::Cross) => true,
            (SubAccountMargining::Cross, SubAccountMargining::Isolated) => false,
            (SubAccountMargining::Isolated, SubAccountMargining::Cross) => false,
            (SubAccountMargining::Isolated, SubAccountMargining::Isolated) => true,
        }
    }
}

impl PartialEq for SettlementType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SettlementType::CashSettled, SettlementType::CashSettled) => true,
            (SettlementType::CashSettled, SettlementType::PhysicalDelivery) => false,
            (SettlementType::PhysicalDelivery, SettlementType::CashSettled) => false,
            (SettlementType::PhysicalDelivery, SettlementType::PhysicalDelivery) => true,
        }
    }
}

impl PartialEq for MarketType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MarketType::Default, MarketType::Default) => true,
            (MarketType::Default, MarketType::PairFuture) => false,
            (MarketType::Default, MarketType::PerpetualFuture) => false,
            (MarketType::Default, MarketType::PreIDO) => false,
            (MarketType::Default, MarketType::IndexFuture) => false,
            (MarketType::PairFuture, MarketType::Default) => false,
            (MarketType::PairFuture, MarketType::PairFuture) => true,
            (MarketType::PairFuture, MarketType::PerpetualFuture) => false,
            (MarketType::PairFuture, MarketType::PreIDO) => false,
            (MarketType::PairFuture, MarketType::IndexFuture) => false,
            (MarketType::PerpetualFuture, MarketType::Default) => false,
            (MarketType::PerpetualFuture, MarketType::PairFuture) => false,
            (MarketType::PerpetualFuture, MarketType::PerpetualFuture) => true,
            (MarketType::PerpetualFuture, MarketType::PreIDO) => false,
            (MarketType::PerpetualFuture, MarketType::IndexFuture) => false,
            (MarketType::PreIDO, MarketType::Default) => false,
            (MarketType::PreIDO, MarketType::PairFuture) => false,
            (MarketType::PreIDO, MarketType::PerpetualFuture) => false,
            (MarketType::PreIDO, MarketType::PreIDO) => true,
            (MarketType::PreIDO, MarketType::IndexFuture) => false,
            (MarketType::IndexFuture, MarketType::Default) => false,
            (MarketType::IndexFuture, MarketType::PairFuture) => false,
            (MarketType::IndexFuture, MarketType::PerpetualFuture) => false,
            (MarketType::IndexFuture, MarketType::PreIDO) => false,
            (MarketType::IndexFuture, MarketType::IndexFuture) => true,
        }
    }
}

impl PartialEq for MarginCollateralRatioType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                MarginCollateralRatioType::Initialization,
                MarginCollateralRatioType::Initialization,
            ) => true,
            (MarginCollateralRatioType::Initialization, MarginCollateralRatioType::Maintenance) => {
                false
            }
            (MarginCollateralRatioType::Maintenance, MarginCollateralRatioType::Initialization) => {
                false
            }
            (MarginCollateralRatioType::Maintenance, MarginCollateralRatioType::Maintenance) => {
                true
            }
        }
    }
}

impl PartialEq for WhitelistStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (WhitelistStatus::Pending, WhitelistStatus::Pending) => true,
            (WhitelistStatus::Pending, WhitelistStatus::Active) => false,
            (WhitelistStatus::Pending, WhitelistStatus::Revoked) => false,
            (WhitelistStatus::Active, WhitelistStatus::Pending) => false,
            (WhitelistStatus::Active, WhitelistStatus::Active) => true,
            (WhitelistStatus::Active, WhitelistStatus::Revoked) => false,
            (WhitelistStatus::Revoked, WhitelistStatus::Pending) => false,
            (WhitelistStatus::Revoked, WhitelistStatus::Active) => false,
            (WhitelistStatus::Revoked, WhitelistStatus::Revoked) => true,
        }
    }
}

impl PartialEq for Side {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Side::Bid, Side::Bid) => true,
            (Side::Bid, Side::Ask) => false,
            (Side::Ask, Side::Bid) => false,
            (Side::Ask, Side::Ask) => true,
        }
    }
}

impl PartialEq for DerivativeOrderType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DerivativeOrderType::Limit, DerivativeOrderType::Limit) => true,
            (DerivativeOrderType::Limit, DerivativeOrderType::ImmediateOrCancel) => false,
            (DerivativeOrderType::Limit, DerivativeOrderType::FillOrKill) => false,
            (DerivativeOrderType::Limit, DerivativeOrderType::PostOnly) => false,
            (DerivativeOrderType::ImmediateOrCancel, DerivativeOrderType::Limit) => false,
            (DerivativeOrderType::ImmediateOrCancel, DerivativeOrderType::ImmediateOrCancel) => {
                true
            }
            (DerivativeOrderType::ImmediateOrCancel, DerivativeOrderType::FillOrKill) => false,
            (DerivativeOrderType::ImmediateOrCancel, DerivativeOrderType::PostOnly) => false,
            (DerivativeOrderType::FillOrKill, DerivativeOrderType::Limit) => false,
            (DerivativeOrderType::FillOrKill, DerivativeOrderType::ImmediateOrCancel) => false,
            (DerivativeOrderType::FillOrKill, DerivativeOrderType::FillOrKill) => true,
            (DerivativeOrderType::FillOrKill, DerivativeOrderType::PostOnly) => false,
            (DerivativeOrderType::PostOnly, DerivativeOrderType::Limit) => false,
            (DerivativeOrderType::PostOnly, DerivativeOrderType::ImmediateOrCancel) => false,
            (DerivativeOrderType::PostOnly, DerivativeOrderType::FillOrKill) => false,
            (DerivativeOrderType::PostOnly, DerivativeOrderType::PostOnly) => true,
        }
    }
}

impl PartialEq for OrderType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (OrderType::Limit, OrderType::Limit) => true,
            (OrderType::Limit, OrderType::ImmediateOrCancel) => false,
            (OrderType::Limit, OrderType::PostOnly) => false,
            (OrderType::ImmediateOrCancel, OrderType::Limit) => false,
            (OrderType::ImmediateOrCancel, OrderType::ImmediateOrCancel) => true,
            (OrderType::ImmediateOrCancel, OrderType::PostOnly) => false,
            (OrderType::PostOnly, OrderType::Limit) => false,
            (OrderType::PostOnly, OrderType::ImmediateOrCancel) => false,
            (OrderType::PostOnly, OrderType::PostOnly) => true,
        }
    }
}

impl Clearing {
    pub fn init_margin_ratio(&self) -> I80F48 {
        I80F48::from(self.config.init_margin)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    pub fn maint_margin_ratio(&self) -> I80F48 {
        I80F48::from(self.config.maint_margin)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    pub fn target_margin_ratio(&self) -> I80F48 {
        I80F48::from(self.config.target_margin)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    pub fn liq_liqor_fee(&self) -> I80F48 {
        I80F48::from(self.config.liq_liqor_fee)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .and_then(|n| n.checked_add(I80F48::ONE))
            .unwrap()
    }

    pub fn liq_insurance_fee(&self) -> I80F48 {
        I80F48::from(self.config.liq_insurance_fee)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    pub fn get_fee_tiers(&self) -> Vec<FeeTier> {
        self.fee_tiers.to_vec()
    }

    /// gets a fee tier by it's identifier, which can be found in a user's `CypherAccount`
    pub fn get_fee_tier(&self, fee_tier: u8) -> FeeTier {
        let ft = self.fee_tiers.iter().find(|ft| ft.tier == fee_tier);
        match ft {
            Some(ft) => *ft,
            None => FeeTier::default(),
        }
    }
}

impl CacheAccount {
    /// gets a price cache as mutable
    pub fn get_price_cache(&self, price_cache_idx: usize) -> &Cache {
        &self.caches[price_cache_idx]
    }

    /// gets the cache for a given oracle products
    pub fn get_cache_for_oracle_products(&self, oracle_products: &Pubkey) -> Option<&Cache> {
        match self
            .caches
            .iter()
            .find(|c| &c.oracle_products == oracle_products)
        {
            Some(c) => Some(c),
            None => None,
        }
    }
}

impl Cache {
    /// the deposit index of the spot token this cache represents
    pub fn deposit_index(&self) -> I80F48 {
        I80F48::from_bits(self.deposit_index)
    }

    /// the borrow index of the spot token this cache represents
    pub fn borrow_index(&self) -> I80F48 {
        I80F48::from_bits(self.borrow_index)
    }

    /// the oracle price
    pub fn oracle_price(&self) -> I80F48 {
        I80F48::from_bits(self.oracle_price)
    }

    /// the twap price
    pub fn market_price(&self) -> I80F48 {
        I80F48::from_bits(self.market_price)
    }

    // spot
    /// gets the init asset weight of the spot token
    pub fn spot_init_asset_weight(&self) -> I80F48 {
        I80F48::from(self.spot_init_asset_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the maint asset weight of the spot token
    pub fn spot_maint_asset_weight(&self) -> I80F48 {
        I80F48::from(self.spot_maint_asset_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the init liab weight of the spot token
    pub fn spot_init_liab_weight(&self) -> I80F48 {
        I80F48::from(self.spot_init_liab_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the maint liab weight of the spot token
    pub fn spot_maint_liab_weight(&self) -> I80F48 {
        I80F48::from(self.spot_maint_liab_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    // futures
    /// gets the init asset weight of the futures position
    pub fn futures_init_asset_weight(&self) -> I80F48 {
        I80F48::from(self.futures_init_asset_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the maint asset weight of the futures position
    pub fn futures_maint_asset_weight(&self) -> I80F48 {
        I80F48::from(self.futures_maint_asset_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the init liab weight of the futures position
    pub fn futures_init_liab_weight(&self) -> I80F48 {
        I80F48::from(self.futures_init_liab_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the maint liab weight of the futures position
    pub fn futures_maint_liab_weight(&self) -> I80F48 {
        I80F48::from(self.futures_maint_liab_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    // perps
    /// gets the init asset weight of the perp position
    pub fn perp_init_asset_weight(&self) -> I80F48 {
        I80F48::from(self.perp_init_asset_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the maint asset weight of the perp position
    pub fn perp_maint_asset_weight(&self) -> I80F48 {
        I80F48::from(self.perp_maint_asset_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the init liab weight of the perp position
    pub fn perp_init_liab_weight(&self) -> I80F48 {
        I80F48::from(self.perp_init_liab_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the maint liab weight of the perp position
    pub fn perp_maint_liab_weight(&self) -> I80F48 {
        I80F48::from(self.perp_maint_liab_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }
}

impl CypherAccount {
    /// gets the assets value of this account
    pub fn get_assets_value(&self) -> I80F48 {
        let mut assets_value = I80F48::ZERO;

        for cache in self.sub_account_caches.iter() {
            if cache.margining == SubAccountMargining::Cross {
                assets_value += cache.assets_value();
            }
        }

        assets_value
    }

    /// gets the liabilites value of this account
    pub fn get_liabilities_value(&self) -> I80F48 {
        let mut liabilities_value = I80F48::ZERO;

        for cache in self.sub_account_caches.iter() {
            if cache.margining == SubAccountMargining::Cross {
                liabilities_value += cache.liabilities_value();
            }
        }

        liabilities_value
    }

    /// gets the c-ratio for this account
    pub fn get_margin_c_ratio(&self) -> I80F48 {
        let mut assets_value = I80F48::ZERO;
        let mut liabilities_value = I80F48::ZERO;

        for cache in self.sub_account_caches.iter() {
            if cache.margining == SubAccountMargining::Cross {
                assets_value += cache.assets_value();
                liabilities_value += cache.liabilities_value();
            }
        }

        if liabilities_value == I80F48::ZERO {
            I80F48::MAX
        } else {
            assets_value / liabilities_value
        }
    }

    /// gets the c-ratio components for this account
    pub fn get_margin_c_ratio_components(&self) -> Vec<(I80F48, I80F48)> {
        self.sub_account_caches
            .iter()
            .filter(|c| c.sub_account != Default::default())
            .map(|c| (c.assets_value(), c.liabilities_value()))
            .collect()
    }
}

impl CypherSubAccount {
    /// positions iterator
    pub fn iter_position_slots<'a>(&'a self) -> impl Iterator<Item = &PositionSlot> {
        struct Iter<'a> {
            positions: &'a [PositionSlot],
        }
        impl<'a> Iterator for Iter<'a> {
            type Item = &'a PositionSlot;
            fn next(&mut self) -> Option<Self::Item> {
                loop {
                    match take(&mut self.positions).split_first() {
                        Some((head, rems)) => {
                            self.positions = rems;
                            if head.spot.token_mint != Pubkey::default()
                                || head.derivative.market != Pubkey::default()
                            {
                                return Some(head);
                            }
                        }
                        None => return None,
                    }
                }
            }
        }
        Iter {
            positions: &self.positions[..],
        }
    }

    /// gets the position index for the given identifier.
    ///
    /// this can be a token mint for a spot position or a market's public key for derivatives
    pub fn get_position_idx(&self, identifier: &Pubkey, is_spot: bool) -> Option<usize> {
        if *identifier == quote_mint::ID && is_spot {
            return Some(QUOTE_TOKEN_IDX);
        }
        self.iter_position_slots().position(|p| {
            if is_spot {
                p.spot.token_mint == *identifier
            } else {
                p.derivative.market == *identifier
            }
        })
    }

    /// gets the derivative positions
    pub fn get_spot_positions(&self) -> Vec<SpotPosition> {
        self.positions
            .iter()
            .filter(|p| p.spot.token_mint != Pubkey::default())
            .map(|p| p.spot)
            .collect()
    }

    /// gets the spot position at the given index
    pub fn get_spot_position(&self, position_idx: usize) -> &SpotPosition {
        assert!(position_idx < TOKENS_MAX_CNT);
        &self.positions[position_idx].spot
    }

    /// gets the spot position at the given index
    pub fn get_spot_position_mut(&mut self, position_idx: usize) -> &mut SpotPosition {
        assert!(position_idx < TOKENS_MAX_CNT);
        &mut self.positions[position_idx].spot
    }

    /// gets the derivative positions
    pub fn get_derivative_positions(&self) -> Vec<DerivativePosition> {
        self.positions
            .iter()
            .filter(|p| p.derivative.market != Pubkey::default())
            .map(|p| p.derivative)
            .collect()
    }

    /// gets the derivative position at the given index
    pub fn get_derivative_position(&self, position_idx: usize) -> &DerivativePosition {
        assert!(position_idx < TOKENS_MAX_CNT);
        &self.positions[position_idx].derivative
    }

    /// gets the derivative position at the given index
    pub fn get_derivative_position_mut(&mut self, position_idx: usize) -> &mut DerivativePosition {
        assert!(position_idx < TOKENS_MAX_CNT);
        &mut self.positions[position_idx].derivative
    }

    /// gets the c-ratio for this sub account
    pub fn get_margin_c_ratio(
        &self,
        cache_account: &CacheAccount,
        mcr_type: MarginCollateralRatioType,
    ) -> I80F48 {
        let (liabs_value, _) = self.get_liabilities_value(cache_account, mcr_type);
        if liabs_value == I80F48::ZERO {
            I80F48::MAX
        } else {
            let (assets_value, _) = self.get_assets_value(cache_account, mcr_type);
            assets_value.saturating_div(liabs_value)
        }
    }

    /// gets the assets value, liabilities value and c-ratio respectively
    pub fn get_margin_c_ratio_components(
        &self,
        cache_account: &CacheAccount,
        mcr_type: MarginCollateralRatioType,
    ) -> (I80F48, I80F48, I80F48) {
        let (liabilities_value, _) = self.get_liabilities_value(cache_account, mcr_type);
        let (assets_value, _) = self.get_assets_value(cache_account, mcr_type);

        if liabilities_value == I80F48::ZERO {
            (I80F48::MAX, assets_value, I80F48::ZERO)
        } else {
            (
                assets_value.saturating_div(liabilities_value),
                assets_value,
                liabilities_value,
            )
        }
    }

    /// gets the assets value of this sub account
    pub fn get_assets_value(
        &self,
        cache_account: &CacheAccount,
        mcr_type: MarginCollateralRatioType,
    ) -> (I80F48, I80F48) {
        let mut assets_value = I80F48::ZERO;
        let mut assets_value_unweighted = I80F48::ZERO;
        let mut cum_pc_total: u64 = 0;

        for position in self.positions.iter() {
            // spot
            if position.spot.token_mint != Pubkey::default() {
                // get the relevant price cache
                let cache = cache_account.get_price_cache(position.spot.cache_index as usize);
                // convert oracle price to fixed type
                let spot_oracle_price = cache.oracle_price();
                // get asset weight according to margin collateral ratio type
                let spot_asset_weight = match mcr_type {
                    MarginCollateralRatioType::Initialization => cache.spot_init_asset_weight(),
                    MarginCollateralRatioType::Maintenance => cache.spot_maint_asset_weight(),
                };
                let spot_position = position.spot.total_position(cache);
                if spot_position.is_positive() {
                    let spot_position_size = spot_position
                        .checked_add(I80F48::from(position.spot.open_orders_cache.coin_total))
                        .unwrap();
                    let spot_value_unweighted = adjust_decimals(spot_position_size, cache.decimals)
                        .checked_mul(spot_oracle_price)
                        .unwrap();
                    assets_value_unweighted += spot_value_unweighted;
                    assets_value += spot_value_unweighted
                        .checked_mul(spot_asset_weight)
                        .unwrap();
                }
                cum_pc_total += position.spot.open_orders_cache.pc_total;
            }

            // derivatives
            if position.derivative.market != Pubkey::default() {
                // get the relevant price cache
                let cache = cache_account.get_price_cache(position.derivative.cache_index as usize);
                let decimals = if position.derivative.market_type == MarketType::PerpetualFuture {
                    cache.perp_decimals
                } else {
                    cache.futures_decimals
                };
                // convert the orresponding price to fixed type
                let derivative_price =
                    if position.derivative.market_type == MarketType::PerpetualFuture {
                        cache.oracle_price()
                    } else {
                        let market_price = cache.market_price();
                        if market_price == I80F48::ZERO {
                            cache.oracle_price()
                        } else {
                            market_price
                        }
                    };
                // get asset weight according to margin collateral ratio type
                let derivative_asset_weight = match (mcr_type, position.derivative.market_type) {
                    (MarginCollateralRatioType::Initialization, MarketType::PairFuture) => {
                        cache.futures_init_asset_weight()
                    }
                    (MarginCollateralRatioType::Initialization, MarketType::PerpetualFuture) => {
                        cache.perp_init_asset_weight()
                    }
                    (MarginCollateralRatioType::Initialization, MarketType::PreIDO) => {
                        cache.futures_init_asset_weight()
                    }
                    (MarginCollateralRatioType::Initialization, MarketType::IndexFuture) => {
                        cache.futures_init_asset_weight()
                    }
                    (MarginCollateralRatioType::Maintenance, MarketType::PairFuture) => {
                        cache.futures_maint_asset_weight()
                    }
                    (MarginCollateralRatioType::Maintenance, MarketType::PerpetualFuture) => {
                        cache.perp_maint_asset_weight()
                    }
                    (MarginCollateralRatioType::Maintenance, MarketType::PreIDO) => {
                        cache.futures_maint_asset_weight()
                    }
                    (MarginCollateralRatioType::Maintenance, MarketType::IndexFuture) => {
                        cache.futures_maint_asset_weight()
                    }
                    _ => unreachable!(),
                };
                let derivative_position = position.derivative.base_position();
                if derivative_position.is_positive() {
                    let derivative_position_size = derivative_position
                        .checked_add(I80F48::from(
                            position.derivative.open_orders_cache.coin_total,
                        ))
                        .unwrap();
                    let derivative_value_unweighted =
                        adjust_decimals(derivative_position_size, decimals)
                            .checked_mul(derivative_price)
                            .unwrap();
                    assets_value_unweighted += derivative_value_unweighted;
                    assets_value += derivative_value_unweighted
                        .checked_mul(derivative_asset_weight)
                        .unwrap();
                }
                // we are going to take derivative coins locked and will price them at the oracle price
                // regardless of whatever price the limit ask orders are actually placed at
                // we do this because these limit asks are actually considered a liability
                // if they weren't, we would run into a risk of a user spamming limit asks without them affecting the c-ratio
                let derivative_coin_locked = position.derivative.open_orders_cache.coin_locked();
                if derivative_coin_locked != 0 {
                    let coin_locked_value_unweighted =
                        adjust_decimals(I80F48::from(derivative_coin_locked), decimals)
                            .checked_mul(derivative_price)
                            .unwrap();
                    assets_value_unweighted += coin_locked_value_unweighted;
                    assets_value += coin_locked_value_unweighted
                        .checked_mul(derivative_asset_weight)
                        .unwrap();
                }
                cum_pc_total += position.derivative.open_orders_cache.pc_total;
            }
        }

        let quote_position = self.positions[QUOTE_TOKEN_IDX].spot;
        let quote_cache = cache_account.get_price_cache(quote_position.cache_index as usize);
        let cum_pc_total_value = adjust_decimals(I80F48::from(cum_pc_total), quote_cache.decimals)
            .checked_mul(I80F48::from_bits(quote_cache.oracle_price))
            .unwrap();

        let quote_asset_weight = match mcr_type {
            MarginCollateralRatioType::Initialization => quote_cache.spot_init_asset_weight(),
            MarginCollateralRatioType::Maintenance => quote_cache.spot_maint_asset_weight(),
        };
        assets_value_unweighted += cum_pc_total_value;
        assets_value += cum_pc_total_value.checked_mul(quote_asset_weight).unwrap();

        (assets_value, assets_value_unweighted)
    }

    /// gets the liabilities value of this sub account
    pub fn get_liabilities_value(
        &self,
        cache_account: &CacheAccount,
        mcr_type: MarginCollateralRatioType,
    ) -> (I80F48, I80F48) {
        let mut liabilities_value = I80F48::ZERO;
        let mut liabilities_value_unweighted = I80F48::ZERO;

        for position in self.positions.iter() {
            // spot
            if position.spot.token_mint != Pubkey::default() {
                // get the relevant price cache
                let cache = cache_account.get_price_cache(position.spot.cache_index as usize);
                // convert oracle price to fixed type
                let spot_oracle_price = cache.oracle_price();
                // get liability weight according to margin collateral ratio type
                let spot_liability_weight = match mcr_type {
                    MarginCollateralRatioType::Initialization => cache.spot_init_liab_weight(),
                    MarginCollateralRatioType::Maintenance => cache.spot_maint_liab_weight(),
                };
                // get total spot position value according to index
                let spot_position = position.spot.total_position(cache);
                if spot_position.is_negative() {
                    let spot_value_unweighted = adjust_decimals(spot_position, cache.decimals)
                        .abs()
                        .checked_mul(spot_oracle_price)
                        .unwrap();
                    liabilities_value_unweighted += spot_value_unweighted;
                    liabilities_value += spot_value_unweighted
                        .checked_mul(spot_liability_weight)
                        .unwrap();
                }
            }
            // derivatives
            if position.derivative.market != Pubkey::default() {
                // get the relevant price cache
                let cache = cache_account.get_price_cache(position.derivative.cache_index as usize);
                let decimals = if position.derivative.market_type == MarketType::PerpetualFuture {
                    cache.perp_decimals
                } else {
                    cache.futures_decimals
                };
                // convert the orresponding price to fixed type
                let derivative_price =
                    if position.derivative.market_type == MarketType::PerpetualFuture {
                        cache.oracle_price()
                    } else {
                        let market_price = cache.market_price();
                        if market_price == I80F48::ZERO {
                            cache.oracle_price()
                        } else {
                            market_price
                        }
                    };

                // get liability weight according to margin collateral ratio type
                let derivative_liability_weight = match (mcr_type, position.derivative.market_type)
                {
                    (MarginCollateralRatioType::Initialization, MarketType::PairFuture) => {
                        cache.futures_init_liab_weight()
                    }
                    (MarginCollateralRatioType::Initialization, MarketType::PerpetualFuture) => {
                        cache.perp_init_liab_weight()
                    }
                    (MarginCollateralRatioType::Initialization, MarketType::PreIDO) => {
                        cache.futures_init_liab_weight()
                    }
                    (MarginCollateralRatioType::Initialization, MarketType::IndexFuture) => {
                        cache.futures_init_liab_weight()
                    }
                    (MarginCollateralRatioType::Maintenance, MarketType::PairFuture) => {
                        cache.futures_maint_liab_weight()
                    }
                    (MarginCollateralRatioType::Maintenance, MarketType::PerpetualFuture) => {
                        cache.perp_maint_liab_weight()
                    }
                    (MarginCollateralRatioType::Maintenance, MarketType::PreIDO) => {
                        cache.futures_maint_liab_weight()
                    }
                    (MarginCollateralRatioType::Maintenance, MarketType::IndexFuture) => {
                        cache.futures_maint_liab_weight()
                    }
                    _ => unreachable!(),
                };
                let derivative_position = position.derivative.base_position();
                if derivative_position.is_negative() {
                    let derivative_value_unweighted =
                        adjust_decimals(derivative_position, decimals)
                            .abs()
                            .checked_mul(derivative_price)
                            .and_then(|n| n.checked_mul(derivative_liability_weight))
                            .unwrap();
                    liabilities_value_unweighted += derivative_value_unweighted;
                    liabilities_value += derivative_value_unweighted
                        .checked_mul(derivative_liability_weight)
                        .unwrap();
                }
            }
        }

        (liabilities_value, liabilities_value_unweighted)
    }

    pub fn is_bankrupt(&self, clearing: &Clearing, cache_account: &CacheAccount) -> Result<bool> {
        let quote_position = self.positions[QUOTE_TOKEN_IDX].spot;
        let quote_cache = cache_account.get_price_cache(quote_position.cache_index as usize);
        let quote_position_size = quote_position.total_position(quote_cache);
        // if the quote token has a deposit we'll use it as the starter for the largest deposit value
        let mut largest_deposit_value = if quote_position_size.is_positive() {
            adjust_decimals(
                quote_position_size
                    .checked_mul(quote_cache.oracle_price())
                    .unwrap(),
                quote_cache.decimals,
            )
        } else {
            I80F48::ZERO
        };
        // if the quote token has a borrow we'll use it's value as the starter for the lowest borrow price
        let mut lowest_borrow_price = if quote_position_size.is_negative() {
            quote_cache.oracle_price()
        } else {
            I80F48::MAX
        };

        for position in self.iter_position_slots() {
            // spot
            if position.spot.token_mint != Pubkey::default() {
                let cache = cache_account.get_price_cache(position.spot.cache_index as usize);
                let spot_oracle_price = cache.oracle_price();
                let spot_position = position.spot.total_position(cache);
                // calculate spot deposit value, if the spot position actually represents a deposit
                let spot_deposit_value = if spot_position.is_positive() {
                    adjust_decimals(
                        spot_position.checked_mul(spot_oracle_price).unwrap(),
                        cache.decimals,
                    )
                } else {
                    I80F48::ZERO
                };
                // if the spot position represents a borrow, update the lowest borrow price
                if spot_position.is_negative() {
                    lowest_borrow_price = I80F48::min(lowest_borrow_price, spot_oracle_price);
                }
                // update largest deposit value according to previously calculated spot deposit value
                largest_deposit_value = I80F48::max(largest_deposit_value, spot_deposit_value);
            }

            // derivatives
            if position.derivative.market != Pubkey::default() {
                let cache = cache_account.get_price_cache(position.derivative.cache_index as usize);
                let decimals = if position.derivative.market_type == MarketType::PerpetualFuture {
                    cache.perp_decimals
                } else {
                    cache.futures_decimals
                };
                // convert the orresponding price to fixed type
                let derivative_price =
                    if position.derivative.market_type == MarketType::PerpetualFuture {
                        cache.oracle_price()
                    } else {
                        cache.market_price()
                    };
                let derivative_position = position.derivative.base_position();
                // calculate derivative deposit value, if the derivative position actually represents a deposit
                let derivative_deposit_value = if derivative_position.is_positive() {
                    adjust_decimals(
                        derivative_position.checked_mul(derivative_price).unwrap(),
                        decimals,
                    )
                } else {
                    I80F48::ZERO
                };
                // if the derivative position represents a borrow, update the lowest borrow price
                if derivative_position.is_negative() {
                    lowest_borrow_price = I80F48::min(lowest_borrow_price, derivative_price);
                }
                // update largest deposit value according to previously calculated derivative deposit value
                largest_deposit_value =
                    I80F48::max(largest_deposit_value, derivative_deposit_value);
            }
        }

        if lowest_borrow_price == I80F48::MAX {
            return Ok(false);
        }

        let liq_fee = clearing.liq_liqor_fee() + clearing.liq_insurance_fee();
        let collateral_for_min_borrow_unit = liq_fee.checked_mul(lowest_borrow_price).unwrap();

        if collateral_for_min_borrow_unit > largest_deposit_value {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl OrdersAccount {
    pub fn get_orders(&self) -> Vec<&OpenOrder> {
        self.open_orders
            .iter()
            .filter(|o| o.order_id != u128::default())
            .collect::<Vec<&OpenOrder>>()
    }
}

impl SubAccountCache {
    /// the value of the assets of this sub account
    pub fn assets_value(&self) -> I80F48 {
        I80F48::from_bits(self.assets_value)
    }
    /// the value of the liabilities of this sub account
    pub fn liabilities_value(&self) -> I80F48 {
        I80F48::from_bits(self.liabilities_value)
    }
    /// the value of the cached c-ratio
    pub fn c_ratio(&self) -> I80F48 {
        I80F48::from_bits(self.c_ratio)
    }
}

impl SpotPosition {
    /// the position, denominated in the base token
    pub fn position(&self) -> I80F48 {
        I80F48::from_bits(self.position)
    }

    pub fn total_position(&self, cache: &Cache) -> I80F48 {
        let position = self.position();
        if position.is_positive() {
            position * cache.deposit_index()
        } else {
            position * cache.borrow_index()
        }
    }
}

impl DerivativePosition {
    /// the deposits of this position
    pub fn base_position(&self) -> I80F48 {
        I80F48::from_bits(self.base_position)
    }

    /// gets the position size taking into account orders still locked in the open orders account
    /// - regardless of whether the position is positive or negative we will add the amount of contracts
    /// locked in the open orders due to ask orders
    ///
    /// this is because whenever an ask gets placed on the book,
    /// the `spent_amount` is the number of contracts and
    /// it gets subtracted to the position size as an accounting mechanism
    pub fn total_position(&self) -> I80F48 {
        let mut base_position = self.base_position();
        // we need to add both potentially locked and free coins to the position
        // locked coins are unmatched from ask orders, so they should be
        base_position += I80F48::from(self.open_orders_cache.coin_total);
        base_position
    }
}

impl OpenOrdersCache {
    pub fn coin_locked(&self) -> u64 {
        self.coin_total - self.coin_free
    }
}

impl Pool {
    /// the pool's utilization rate
    pub fn utilization_rate(&self) -> I80F48 {
        let borrows = self.total_borrows();
        if borrows == I80F48::ZERO {
            I80F48::ZERO
        } else {
            borrows.saturating_div(self.total_deposits())
        }
    }

    /// the pool's optimal APR
    pub fn optimal_apr(&self) -> I80F48 {
        I80F48::from_num(self.config.optimal_apr)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// the pool's max APR
    pub fn max_apr(&self) -> I80F48 {
        I80F48::from_num(self.config.max_apr)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// the pool's optimal utilization rate
    pub fn optimal_util(&self) -> I80F48 {
        I80F48::from_num(self.config.optimal_util)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// the deposits of this pool
    pub fn deposits(&self) -> I80F48 {
        I80F48::from_bits(self.deposits)
    }

    /// the deposits of this pool
    pub fn total_deposits(&self) -> I80F48 {
        self.deposits().mul(self.deposit_index())
    }

    /// the borrows of this pool
    pub fn borrows(&self) -> I80F48 {
        I80F48::from_bits(self.borrows)
    }

    /// the deposits of this pool
    pub fn total_borrows(&self) -> I80F48 {
        self.borrows().mul(self.borrow_index())
    }

    /// the deposit index of this pool
    pub fn deposit_index(&self) -> I80F48 {
        I80F48::from_bits(self.deposit_index)
    }

    /// the borrows of this pool
    pub fn borrow_index(&self) -> I80F48 {
        I80F48::from_bits(self.borrow_index)
    }

    /// the pool's borrow interest rate
    pub fn borrow_rate(&self) -> I80F48 {
        let utilization = self.utilization_rate();
        let optimal_apr = self.optimal_apr();
        let max_apr = self.max_apr();
        let optimal_util = self.optimal_util();

        if utilization > optimal_util {
            let extra_util = utilization - optimal_util;
            let slope = (max_apr - optimal_apr)
                .checked_div(I80F48::ONE - optimal_util)
                .unwrap();
            optimal_apr + (slope.checked_mul(extra_util).unwrap())
        } else {
            let slope = optimal_apr.checked_div(optimal_util).unwrap();
            slope.checked_mul(utilization).unwrap()
        }
    }

    /// the pool's deposit interest rate
    pub fn deposit_rate(&self) -> I80F48 {
        self.borrow_rate()
            .saturating_mul(self.utilization_rate())
            .saturating_div(I80F48::ONE)
    }

    /// accumulated borrow interest payments
    pub fn accum_borrow_interest_payment(&self) -> I80F48 {
        I80F48::from_bits(self.accum_borrow_interest_payment)
    }

    /// accumulated deposit interest payments
    pub fn accum_deposit_interest_payment(&self) -> I80F48 {
        I80F48::from_bits(self.accum_deposit_interest_payment)
    }
}

impl PoolNode {
    /// accumulated borrows
    pub fn accum_borrows(&self) -> I80F48 {
        I80F48::from_bits(self.accum_borrows)
    }

    /// accumulated repays
    pub fn accum_repays(&self) -> I80F48 {
        I80F48::from_bits(self.accum_borrows)
    }
}

impl FuturesMarket {
    /// the twap price
    pub fn market_price(&self) -> I80F48 {
        I80F48::from_bits(self.market_price)
    }

    pub fn total_raised(&self) -> I80F48 {
        I80F48::from_bits(self.total_raised)
    }
}

impl PerpetualMarket {
    /// the long funding
    pub fn long_funding(&self) -> I80F48 {
        I80F48::from_bits(self.long_funding)
    }

    /// the short funding
    pub fn short_funding(&self) -> I80F48 {
        I80F48::from_bits(self.short_funding)
    }
}

pub trait Market: Send + Sync {
    fn event_queue(&self) -> Pubkey;
    fn base_multiplier(&self) -> u64;
    fn quote_multiplier(&self) -> u64;
    fn decimals(&self) -> u8;
    fn unscale_base_amount(&self, base_amount: u64) -> Option<u64>;
    fn unscale_quote_amount(&self, quote_amount: u64) -> Option<u64>;
    fn get_quote_from_base(&self, base_amount: u64, scaled_price_fp32: u64) -> Option<u64>;
}

impl Market for PerpetualMarket {
    fn event_queue(&self) -> Pubkey {
        self.inner.event_queue
    }
    fn unscale_base_amount(&self, base_amount: u64) -> Option<u64> {
        base_amount.checked_mul(self.inner.base_multiplier)
    }

    fn unscale_quote_amount(&self, quote_amount: u64) -> Option<u64> {
        quote_amount.checked_mul(self.inner.quote_multiplier)
    }

    fn get_quote_from_base(&self, base_amount: u64, scaled_price_fp32: u64) -> Option<u64> {
        fp32_mul_floor(base_amount, scaled_price_fp32)
            .and_then(|n| (n as u128).checked_mul(self.inner.quote_multiplier as u128))
            .and_then(|n| n.checked_div(self.inner.base_multiplier as u128))
            .and_then(|n| n.try_into().ok())
    }

    fn base_multiplier(&self) -> u64 {
        self.inner.base_multiplier
    }

    fn quote_multiplier(&self) -> u64 {
        self.inner.quote_multiplier
    }

    fn decimals(&self) -> u8 {
        self.inner.config.decimals
    }
}

impl Market for FuturesMarket {
    fn event_queue(&self) -> Pubkey {
        self.inner.event_queue
    }
    fn unscale_base_amount(&self, base_amount: u64) -> Option<u64> {
        base_amount.checked_mul(self.inner.base_multiplier)
    }

    fn unscale_quote_amount(&self, quote_amount: u64) -> Option<u64> {
        quote_amount.checked_mul(self.inner.quote_multiplier)
    }

    fn get_quote_from_base(&self, base_amount: u64, scaled_price_fp32: u64) -> Option<u64> {
        fp32_mul_floor(base_amount, scaled_price_fp32)
            .and_then(|n| (n as u128).checked_mul(self.inner.quote_multiplier as u128))
            .and_then(|n| n.checked_div(self.inner.base_multiplier as u128))
            .and_then(|n| n.try_into().ok())
    }

    fn base_multiplier(&self) -> u64 {
        self.inner.base_multiplier
    }

    fn quote_multiplier(&self) -> u64 {
        self.inner.quote_multiplier
    }

    fn decimals(&self) -> u8 {
        self.inner.config.decimals
    }
}
