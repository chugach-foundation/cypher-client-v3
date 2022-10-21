pub mod aob;
pub mod constants;
pub mod instructions;
pub mod serum;
pub mod utils;

use anchor_lang::prelude::*;
use bonfida_utils::fp_math::fp32_mul;
use constants::{INV_ONE_HUNDRED_FIXED, QUOTE_TOKEN_IDX};
use fixed::types::I80F48;

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
declare_id!("cyph3iWWJctHgNosbRqxg4GjMHsEL8wAPBnKzPRxEdF");

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
    #[cfg(not(feature = "mainnet-beta"))]
    declare_id!("146KULKKVzc7EXVhv7J5fshHSroTADCCnYRFQtSfHGi7");
}

pub mod wrapped_sol {
    use anchor_lang::declare_id;
    declare_id!("So11111111111111111111111111111111111111112");
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
            .map(|c| {
                (
                    I80F48::from_bits(c.assets_value),
                    I80F48::from_bits(c.liabilities_value),
                )
            })
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
        let quote_position = self.positions[QUOTE_TOKEN_IDX].spot;
        let quote_cache = cache_account.get_price_cache(quote_position.cache_index as usize);
        let quote_position_size = quote_position.total_position(quote_cache);
        let mut assets_value = if quote_position_size.is_positive() {
            quote_position_size
        } else {
            I80F48::ZERO
        };

        for position in self.positions.iter() {
            // spot
            if position.spot.token_mint != Pubkey::default() {
                // get the relevant price cache
                let cache = cache_account.get_price_cache(position.spot.cache_index as usize);
                // convert oracle price to fixed type
                let spot_oracle_price = I80F48::from(cache.oracle_price);
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
                    assets_value += spot_position_size
                        .checked_mul(spot_oracle_price)
                        .and_then(|n| n.checked_mul(spot_asset_weight))
                        .unwrap();
                }
                assets_value += I80F48::from(position.spot.open_orders_cache.pc_total);
            }

            // derivatives
            if position.derivative.market != Pubkey::default() {
                // get the relevant price cache
                let cache = cache_account.get_price_cache(position.derivative.cache_index as usize);
                // convert the orresponding price to fixed type
                let derivative_price =
                    if position.derivative.market_type == MarketType::PerpetualFuture {
                        I80F48::from(cache.oracle_price)
                    } else {
                        I80F48::from(cache.market_price)
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
                    assets_value += derivative_position_size
                        .checked_mul(derivative_price)
                        .and_then(|n| n.checked_mul(derivative_asset_weight))
                        .unwrap();
                }
                assets_value += I80F48::from(position.derivative.open_orders_cache.pc_total);
            }
        }

        assets_value
    }

    /// gets the liabilities value of this sub account
    pub fn get_liabilities_value(
        &self,
        cache_account: &CacheAccount,
        mcr_type: MarginCollateralRatioType,
    ) -> I80F48 {
        let quote_position = self.positions[QUOTE_TOKEN_IDX].spot;
        let quote_cache = cache_account.get_price_cache(quote_position.cache_index as usize);
        let quote_position_size = quote_position.total_position(quote_cache);
        let mut liabilities_value = if quote_position_size.is_negative() {
            quote_position_size.abs()
        } else {
            I80F48::ZERO
        };

        for position in self.positions.iter() {
            // spot
            if position.spot.token_mint != Pubkey::default() {
                // get the relevant price cache
                let cache = cache_account.get_price_cache(position.spot.cache_index as usize);
                // convert oracle price to fixed type
                let spot_oracle_price = I80F48::from(cache.oracle_price);
                // get liability weight according to margin collateral ratio type
                let spot_liability_weight = match mcr_type {
                    MarginCollateralRatioType::Initialization => cache.spot_init_liab_weight(),
                    MarginCollateralRatioType::Maintenance => cache.spot_maint_liab_weight(),
                };
                // get total spot position value according to index
                let spot_position = position.spot.total_position(cache);
                if spot_position.is_negative() {
                    liabilities_value += spot_position
                        .abs()
                        .checked_mul(spot_oracle_price)
                        .and_then(|n| n.checked_mul(spot_liability_weight))
                        .unwrap();
                }
            }
            // derivatives
            if position.derivative.market != Pubkey::default() {
                // get the relevant price cache
                let cache = cache_account.get_price_cache(position.derivative.cache_index as usize);
                // convert the orresponding price to fixed type
                let derivative_price =
                    if position.derivative.market_type == MarketType::PerpetualFuture {
                        I80F48::from(cache.oracle_price)
                    } else {
                        I80F48::from(cache.market_price)
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
                    liabilities_value += derivative_position
                        .abs()
                        .checked_mul(derivative_price)
                        .and_then(|n| n.checked_mul(derivative_liability_weight))
                        .unwrap();
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
}

impl Pool {
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
}

pub trait Market {
    fn event_queue(&self) -> Pubkey;
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
}
