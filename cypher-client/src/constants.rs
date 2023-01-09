use fixed::types::I80F48;

pub const ONE_MINUTE: u64 = 60;
pub const ONE_HOUR: u64 = ONE_MINUTE * 60;
pub const ONE_DAY: u64 = ONE_HOUR * 24;
pub const ONE_WEEK: u64 = ONE_DAY * 7;
pub const ONE_YEAR: u64 = ONE_DAY * 365;

pub const INV_ONE_HUNDRED_FIXED: I80F48 = I80F48::from_bits(2_814_749_767_106_i128);
pub const INV_TEN_THOUSAND_FIXED: I80F48 = I80F48::from_bits(28_147_497_671_i128);
pub const INV_ONE_DECIMAL_ADJ_FIXED: I80F48 = I80F48::from_bits(28147497671065);
pub const INV_TWO_DECIMAL_ADJ_FIXED: I80F48 = I80F48::from_bits(2814749767106);
pub const INV_THREE_DECIMAL_ADJ_FIXED: I80F48 = I80F48::from_bits(281474976710);
pub const INV_FOUR_DECIMAL_ADJ_FIXED: I80F48 = I80F48::from_bits(28147497671);
pub const INV_FIVE_DECIMAL_ADJ_FIXED: I80F48 = I80F48::from_bits(2814749767);
pub const INV_SIX_DECIMAL_ADJ_FIXED: I80F48 = I80F48::from_bits(281474976);
pub const INV_SEVEN_DECIMAL_ADJ_FIXED: I80F48 = I80F48::from_bits(28147497);
pub const INV_EIGHT_DECIMAL_ADJ_FIXED: I80F48 = I80F48::from_bits(2814749);
pub const INV_NINE_DECIMAL_ADJ_FIXED: I80F48 = I80F48::from_bits(281474);

pub const B_CLEARING: &[u8] = b"CYPHER_CLEARING";
pub const B_POOL: &[u8] = b"CYPHER_POOL";
pub const B_POOL_NODE: &[u8] = b"CYPHER_POOL_NODE";
pub const B_POOL_NODE_VAULT: &[u8] = b"CYPHER_POOL_NODE_VAULT";
pub const B_POOL_NODE_VAULT_SIGNER: &[u8] = b"CYPHER_POOL_NODE_VAULT_SIGNER";
pub const B_CYPHER_ACCOUNT: &[u8] = b"CYPHER_ACCOUNT";
pub const B_CYPHER_SUB_ACCOUNT: &[u8] = b"CYPHER_SUB_ACCOUNT";
pub const B_ORACLE_PRODUCTS: &[u8] = b"CYPHER_ORACLE_PRODUCTS";
pub const B_CYPHER_MARKET: &[u8] = b"CYPHER_MARKET";
pub const B_WHITELIST: &[u8] = b"CYPHER_WHITELIST";
pub const B_ORDERS_ACCOUNT: &[u8] = b"CYPHER_ORDERS_ACCOUNT";
pub const B_OPEN_ORDERS: &[u8] = b"OPEN_ORDERS";

/// The length of the encoded sub account alias.
pub const SUB_ACCOUNT_ALIAS_LEN: usize = 32;
/// The length of the encoded market name.
pub const MARKET_NAME_LEN: usize = 32;

/// The maximum number of spot positions held.
pub const MARKETS_MAX_CNT: usize = 23;
/// The maximum number of tokens.
pub const TOKENS_MAX_CNT: usize = MARKETS_MAX_CNT + 1;
/// The quote token index.
pub const QUOTE_TOKEN_IDX: usize = TOKENS_MAX_CNT - 1;

/// The maximum number of nodes that a pool can have.
pub const NODES_MAX_CNT: usize = 24;

/// The maximum number of orders in the orders acocunt.
pub const ORDER_MAX_CNT: u8 = 128;

/// The quote token exponent used to calculate the quote token oracle price from the pyth account.
pub const QUOTE_TOKEN_EXPONENT: u32 = 8;
/// The decimals of the quote token.
pub const QUOTE_TOKEN_DECIMALS: u8 = 6;
/// The maximum number of loops during the matching on a new order instruction.
pub const ORDERBOOK_MATCH_CAP: u16 = 10;

/// The time to live for oracle prices, in slots.
pub const ORACLE_PRICE_TTL_IN_SLOT: u64 = 250;
