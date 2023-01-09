pub mod aob;
pub mod constants;
pub mod instructions;
pub mod serum;
pub mod utils;

use agnostic_orderbook::state::Side as AobSide;
use anchor_lang::prelude::*;
use anchor_spl::dex::serum_dex::matching::Side as DexSide;
use bonfida_utils::fp_math::fp32_mul;
use constants::{INV_ONE_HUNDRED_FIXED, QUOTE_TOKEN_IDX};
use fixed::types::I80F48;
use utils::adjust_decimals;

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
declare_id!("6prLRRLSvwWLkCBc7V2B3FWi716AssNyPfp1NH88751v");

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
    declare_id!("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("DESVgJVGajEgKGXhb6XmqDHGz3VjdgP7rEVESBgxmroY");
}

pub mod cache_account {
    use anchor_lang::declare_id;
    #[cfg(feature = "mainnet-beta")]
    declare_id!("6x5U4c41tfUYGEbTXofFiHcfyx3rqJZsT4emrLisNGGL");
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("3ac3b5RYdEogzXHr7xMESiyYKkDzXGShuSJWX1ZPWHRP");
}

pub mod wrapped_sol {
    use anchor_lang::declare_id;
    declare_id!("So11111111111111111111111111111111111111112");
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

impl PartialEq for ProductsType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ProductsType::Stub, ProductsType::Stub) => true,
            (ProductsType::Stub, ProductsType::Pyth) => false,
            (ProductsType::Stub, ProductsType::Switchboard) => false,
            (ProductsType::Pyth, ProductsType::Pyth) => true,
            (ProductsType::Pyth, ProductsType::Stub) => false,
            (ProductsType::Pyth, ProductsType::Switchboard) => false,
            (ProductsType::Switchboard, ProductsType::Switchboard) => true,
            (ProductsType::Switchboard, ProductsType::Stub) => false,
            (ProductsType::Switchboard, ProductsType::Pyth) => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (ProductsType::Stub, ProductsType::Stub) => false,
            (ProductsType::Stub, ProductsType::Pyth) => true,
            (ProductsType::Stub, ProductsType::Switchboard) => true,
            (ProductsType::Pyth, ProductsType::Pyth) => false,
            (ProductsType::Pyth, ProductsType::Stub) => true,
            (ProductsType::Pyth, ProductsType::Switchboard) => true,
            (ProductsType::Switchboard, ProductsType::Switchboard) => false,
            (ProductsType::Switchboard, ProductsType::Stub) => true,
            (ProductsType::Switchboard, ProductsType::Pyth) => true,
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

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (SubAccountMargining::Cross, SubAccountMargining::Cross) => false,
            (SubAccountMargining::Cross, SubAccountMargining::Isolated) => true,
            (SubAccountMargining::Isolated, SubAccountMargining::Cross) => true,
            (SubAccountMargining::Isolated, SubAccountMargining::Isolated) => false,
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

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (SettlementType::CashSettled, SettlementType::CashSettled) => false,
            (SettlementType::CashSettled, SettlementType::PhysicalDelivery) => true,
            (SettlementType::PhysicalDelivery, SettlementType::CashSettled) => true,
            (SettlementType::PhysicalDelivery, SettlementType::PhysicalDelivery) => false,
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

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (MarketType::Default, MarketType::Default) => false,
            (MarketType::Default, MarketType::PairFuture) => true,
            (MarketType::Default, MarketType::PerpetualFuture) => true,
            (MarketType::Default, MarketType::PreIDO) => true,
            (MarketType::Default, MarketType::IndexFuture) => true,
            (MarketType::PairFuture, MarketType::Default) => true,
            (MarketType::PairFuture, MarketType::PairFuture) => false,
            (MarketType::PairFuture, MarketType::PerpetualFuture) => true,
            (MarketType::PairFuture, MarketType::PreIDO) => true,
            (MarketType::PairFuture, MarketType::IndexFuture) => true,
            (MarketType::PerpetualFuture, MarketType::Default) => true,
            (MarketType::PerpetualFuture, MarketType::PairFuture) => true,
            (MarketType::PerpetualFuture, MarketType::PerpetualFuture) => false,
            (MarketType::PerpetualFuture, MarketType::PreIDO) => true,
            (MarketType::PerpetualFuture, MarketType::IndexFuture) => true,
            (MarketType::PreIDO, MarketType::Default) => true,
            (MarketType::PreIDO, MarketType::PairFuture) => true,
            (MarketType::PreIDO, MarketType::PerpetualFuture) => true,
            (MarketType::PreIDO, MarketType::PreIDO) => false,
            (MarketType::PreIDO, MarketType::IndexFuture) => true,
            (MarketType::IndexFuture, MarketType::Default) => true,
            (MarketType::IndexFuture, MarketType::PairFuture) => true,
            (MarketType::IndexFuture, MarketType::PerpetualFuture) => true,
            (MarketType::IndexFuture, MarketType::PreIDO) => true,
            (MarketType::IndexFuture, MarketType::IndexFuture) => false,
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

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (
                MarginCollateralRatioType::Initialization,
                MarginCollateralRatioType::Initialization,
            ) => false,
            (MarginCollateralRatioType::Initialization, MarginCollateralRatioType::Maintenance) => {
                true
            }
            (MarginCollateralRatioType::Maintenance, MarginCollateralRatioType::Initialization) => {
                true
            }
            (MarginCollateralRatioType::Maintenance, MarginCollateralRatioType::Maintenance) => {
                false
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

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (WhitelistStatus::Pending, WhitelistStatus::Pending) => false,
            (WhitelistStatus::Pending, WhitelistStatus::Active) => true,
            (WhitelistStatus::Pending, WhitelistStatus::Revoked) => true,
            (WhitelistStatus::Active, WhitelistStatus::Pending) => true,
            (WhitelistStatus::Active, WhitelistStatus::Active) => false,
            (WhitelistStatus::Active, WhitelistStatus::Revoked) => true,
            (WhitelistStatus::Revoked, WhitelistStatus::Pending) => true,
            (WhitelistStatus::Revoked, WhitelistStatus::Active) => true,
            (WhitelistStatus::Revoked, WhitelistStatus::Revoked) => false,
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

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (Side::Bid, Side::Bid) => false,
            (Side::Bid, Side::Ask) => true,
            (Side::Ask, Side::Bid) => true,
            (Side::Ask, Side::Ask) => false,
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

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (DerivativeOrderType::Limit, DerivativeOrderType::Limit) => false,
            (DerivativeOrderType::Limit, DerivativeOrderType::ImmediateOrCancel) => true,
            (DerivativeOrderType::Limit, DerivativeOrderType::FillOrKill) => true,
            (DerivativeOrderType::Limit, DerivativeOrderType::PostOnly) => true,
            (DerivativeOrderType::ImmediateOrCancel, DerivativeOrderType::Limit) => true,
            (DerivativeOrderType::ImmediateOrCancel, DerivativeOrderType::ImmediateOrCancel) => {
                false
            }
            (DerivativeOrderType::ImmediateOrCancel, DerivativeOrderType::FillOrKill) => true,
            (DerivativeOrderType::ImmediateOrCancel, DerivativeOrderType::PostOnly) => true,
            (DerivativeOrderType::FillOrKill, DerivativeOrderType::Limit) => true,
            (DerivativeOrderType::FillOrKill, DerivativeOrderType::ImmediateOrCancel) => true,
            (DerivativeOrderType::FillOrKill, DerivativeOrderType::FillOrKill) => false,
            (DerivativeOrderType::FillOrKill, DerivativeOrderType::PostOnly) => true,
            (DerivativeOrderType::PostOnly, DerivativeOrderType::Limit) => true,
            (DerivativeOrderType::PostOnly, DerivativeOrderType::ImmediateOrCancel) => true,
            (DerivativeOrderType::PostOnly, DerivativeOrderType::FillOrKill) => true,
            (DerivativeOrderType::PostOnly, DerivativeOrderType::PostOnly) => false,
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

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (OrderType::Limit, OrderType::Limit) => false,
            (OrderType::Limit, OrderType::ImmediateOrCancel) => true,
            (OrderType::Limit, OrderType::PostOnly) => true,
            (OrderType::ImmediateOrCancel, OrderType::Limit) => true,
            (OrderType::ImmediateOrCancel, OrderType::ImmediateOrCancel) => false,
            (OrderType::ImmediateOrCancel, OrderType::PostOnly) => true,
            (OrderType::PostOnly, OrderType::Limit) => true,
            (OrderType::PostOnly, OrderType::ImmediateOrCancel) => true,
            (OrderType::PostOnly, OrderType::PostOnly) => false,
        }
    }
}

impl Clearing {
    pub fn init_margin_ratio(&self) -> I80F48 {
        I80F48::from(self.config.init_margin)
    }

    pub fn maint_margin_ratio(&self) -> I80F48 {
        I80F48::from(self.config.maint_margin)
    }

    pub fn target_margin_ratio(&self) -> I80F48 {
        I80F48::from(self.config.target_margin)
    }

    pub fn liq_liqor_fee(&self) -> I80F48 {
        I80F48::from(self.config.liq_liqor_fee)
    }

    pub fn liq_insurance_fee(&self) -> I80F48 {
        I80F48::from(self.config.liq_insurance_fee)
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
            .map(|c| (c.assets_value(), c.liabilities_value()))
            .to_vec()
    }
}

impl CypherSubAccount {
    /// gets the c-ratio for this sub account
    pub fn get_margin_c_ratio(
        &self,
        cache_account: &CacheAccount,
        mcr_type: MarginCollateralRatioType,
    ) -> I80F48 {
        let liabs_value = self.get_liabilities_value(cache_account, mcr_type);
        if liabs_value == I80F48::ZERO {
            I80F48::MAX
        } else {
            let assets_value = self.get_assets_value(cache_account, mcr_type);
            assets_value.saturating_div(liabs_value)
        }
    }

    /// gets the assets value, liabilities value and c-ratio respectively
    pub fn get_margin_c_ratio_components(
        &self,
        cache_account: &CacheAccount,
        mcr_type: MarginCollateralRatioType,
    ) -> (I80F48, I80F48, I80F48) {
        let liabilities_value = self.get_liabilities_value(cache_account, mcr_type);
        let assets_value = self.get_assets_value(cache_account, mcr_type);

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
    ) -> I80F48 {
        let mut assets_value = I80F48::ZERO;
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
                    let spot_value = adjust_decimals(spot_position_size, cache.decimals)
                        .checked_mul(spot_oracle_price)
                        .and_then(|n| n.checked_mul(spot_asset_weight))
                        .unwrap();
                    assets_value += spot_value;
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
                    let derivative_value = adjust_decimals(derivative_position_size, decimals)
                        .checked_mul(derivative_price)
                        .and_then(|n| n.checked_mul(derivative_asset_weight))
                        .unwrap();
                    assets_value += derivative_value;
                }
                cum_pc_total += position.derivative.open_orders_cache.pc_total;
            }
        }

        let quote_position = self.positions[QUOTE_TOKEN_IDX].spot;
        let quote_cache = cache_account.get_price_cache(quote_position.cache_index as usize);

        assets_value += adjust_decimals(I80F48::from(cum_pc_total), quote_cache.decimals)
            .checked_mul(I80F48::from_bits(quote_cache.oracle_price))
            .unwrap();

        assets_value
    }

    /// gets the liabilities value of this sub account
    pub fn get_liabilities_value(
        &self,
        cache_account: &CacheAccount,
        mcr_type: MarginCollateralRatioType,
    ) -> I80F48 {
        let mut liabilities_value = I80F48::ZERO;

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
                    let spot_value = adjust_decimals(spot_position, cache.decimals)
                        .abs()
                        .checked_mul(spot_oracle_price)
                        .and_then(|n| n.checked_mul(spot_liability_weight))
                        .unwrap();
                    liabilities_value += spot_value
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
                    let derivative_value = adjust_decimals(derivative_position, decimals)
                        .abs()
                        .checked_mul(derivative_price)
                        .and_then(|n| n.checked_mul(derivative_liability_weight))
                        .unwrap();
                    liabilities_value += derivative_value;
                }
            }
        }

        liabilities_value
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

impl Pool {
    /// the pool's utilization rate
    pub fn utilization_rate(&self) -> I80F48 {
        let borrows = self.borrows();
        if borrows == I80F48::ZERO {
            I80F48::ZERO
        } else {
            borrows.saturating_div(self.deposits())
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

    /// the borrows of this pool
    pub fn borrows(&self) -> I80F48 {
        I80F48::from_bits(self.borrows)
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
        fp32_mul(base_amount, scaled_price_fp32)
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
        fp32_mul(base_amount, scaled_price_fp32)
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
