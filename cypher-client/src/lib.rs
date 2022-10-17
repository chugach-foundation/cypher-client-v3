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
        CallBackInfo
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
    pub fn get_margin_c_ratio(&self) -> I80F48 {
        let liabs_value = self.get_liabilities_value();
        if liabs_value == I80F48::ZERO {
            I80F48::MAX
        } else {
            let assets_value = self.get_assets_value();
            assets_value / liabs_value
        }
    }

    /// gets the assets value, liabilities value and c-ratio respectively
    pub fn get_margin_c_ratio_components(&self) -> (I80F48, I80F48, I80F48) {
        let liabilities_value = self.get_liabilities_value();
        let assets_value = self.get_assets_value();

        if liabilities_value == I80F48::ZERO {
            (assets_value, I80F48::ZERO, I80F48::MAX)
        } else {
            (
                assets_value,
                liabilities_value,
                assets_value / liabilities_value,
            )
        }
    }

    /// gets the assets value of this sub account
    pub fn get_assets_value(&self) -> I80F48 {
        let quote_position = self.positions[QUOTE_TOKEN_IDX].spot.total_position();
        let mut assets_value = if quote_position.is_positive() {
            quote_position
        } else {
            I80F48::ZERO
        };

        for position in self.positions.iter() {
            // spot
            let spot_oracle_price = I80F48::from(position.spot.oracle_price);
            let spot_asset_weight = position.spot.asset_weight();
            let spot_position = position.spot.total_position();
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

            // derivatives
            let derivative_price = I80F48::from(position.derivative.price);
            let derivative_asset_weight = position.derivative.asset_weight();
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

        assets_value
    }

    /// gets the liabilities value of this sub account
    pub fn get_liabilities_value(&self) -> I80F48 {
        let quote_position = self.positions[QUOTE_TOKEN_IDX].spot.total_position();
        let mut liabilities_value = if quote_position.is_negative() {
            quote_position
        } else {
            I80F48::ZERO
        };

        for position in self.positions.iter() {
            // spot
            let spot_oracle_price = I80F48::from(position.spot.oracle_price);
            let spot_liability_weight = position.spot.liability_weight();
            let spot_position = position.spot.total_position();
            if spot_position.is_negative() {
                liabilities_value += spot_position
                    .abs()
                    .checked_mul(spot_oracle_price)
                    .and_then(|n| n.checked_mul(spot_liability_weight))
                    .unwrap();
            }
            // derivatives
            let derivative_price = I80F48::from(position.derivative.price);
            let derivative_liability_weight = position.derivative.liability_weight();
            let derivative_position = position.derivative.base_position();
            if derivative_position.is_negative() {
                liabilities_value += derivative_position
                    .abs()
                    .checked_mul(derivative_price)
                    .and_then(|n| n.checked_mul(derivative_liability_weight))
                    .unwrap();
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

    /// gets the position's weight as an asset
    pub fn asset_weight(&self) -> I80F48 {
        I80F48::from(self.asset_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the position's weight as a liability
    pub fn liability_weight(&self) -> I80F48 {
        I80F48::from(self.liability_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    pub fn total_position(&self) -> I80F48 {
        let position = self.position();
        if position.is_positive() {
            position * self.deposit_index()
        } else {
            position * self.borrow_index()
        }
    }

    /// the deposit index of the underlying token
    pub fn deposit_index(&self) -> I80F48 {
        I80F48::from_bits(self.deposit_index)
    }

    /// the borrow index of the underlying token
    pub fn borrow_index(&self) -> I80F48 {
        I80F48::from_bits(self.borrow_index)
    }
}

impl DerivativePosition {
    /// the deposits of this position
    pub fn base_position(&self) -> I80F48 {
        I80F48::from_bits(self.base_position)
    }

    /// gets the position's weight as an asset
    pub fn asset_weight(&self) -> I80F48 {
        I80F48::from(self.asset_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
    }

    /// gets the position's weight as a liability
    pub fn liability_weight(&self) -> I80F48 {
        I80F48::from(self.liability_weight)
            .checked_mul(INV_ONE_HUNDRED_FIXED)
            .unwrap()
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
