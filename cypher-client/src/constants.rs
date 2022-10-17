use fixed::types::I80F48;

pub const B_CLEARING: &[u8] = b"CYPHER_CLEARING";
pub const B_POOL: &[u8] = b"CYPHER_POOL";
pub const B_POOL_VAULT: &[u8] = b"CYPHER_POOL_VAULT";
pub const B_CYPHER_ACCOUNT: &[u8] = b"CYPHER_ACCOUNT";
pub const B_CYPHER_SUB_ACCOUNT: &[u8] = b"CYPHER_SUB_ACCOUNT";
pub const B_CYPHER_MARKET: &[u8] = b"CYPHER_MARKET";
pub const B_WHITELIST: &[u8] = b"CYPHER_WHITELIST";
pub const B_ORDERS_ACCOUNT: &[u8] = b"CYPHER_ORDERS_ACCOUNT";
pub const B_OPEN_ORDERS: &[u8] = b"OPEN_ORDERS";

pub const SUB_ACCOUNT_ALIAS_LEN: usize = 32;
pub const MARKET_NAME_LEN: usize = 32;

pub const MARKETS_MAX_CNT: usize = 23;
pub const TOKENS_MAX_CNT: usize = MARKETS_MAX_CNT + 1;
pub const QUOTE_TOKEN_IDX: usize = TOKENS_MAX_CNT - 1;

pub const INV_ONE_HUNDRED_FIXED: I80F48 = I80F48::from_bits(2_814_749_767_106_i128);

pub const QUOTE_DECIMALS: u8 = 6;
pub const ORDER_MAX_CNT: u8 = 128;
