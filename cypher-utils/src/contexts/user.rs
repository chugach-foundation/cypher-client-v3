use anchor_spl::token::{spl_token, TokenAccount};
use cypher_client::{
    instructions::deposit_funds,
    utils::{derive_pool_vault_address, derive_token_address},
    wrapped_sol, PositionSlot,
};
use solana_sdk::{instruction::Instruction, signature::Signature};
use {
    cypher_client::{
        instructions::{create_account, create_subaccount, withdraw_funds},
        utils::{derive_account_address, derive_sub_account_address},
        CypherAccount, CypherSubAccount,
    },
    solana_client::{client_error::ClientError, nonblocking::rpc_client::RpcClient},
    solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer},
    std::sync::Arc,
};

use crate::utils::{
    create_transaction, encode_string, get_create_account_ix, get_cypher_zero_copy_account,
    get_multiple_cypher_program_accounts, send_transaction, send_transactions,
};

use super::ContextError;

/// Represents a [`CypherSubAccount`].
pub struct SubAccountContext {
    /// The account's pubkey.
    pub address: Pubkey,
    /// The account's state.
    pub state: Box<CypherSubAccount>,
}

impl SubAccountContext {
    pub fn new(address: Pubkey, state: Box<CypherSubAccount>) -> Self {
        Self { address, state }
    }
}

/// Represents a [`CypherAccount`].
pub struct AccountContext {
    /// The account's pubkey.
    pub address: Pubkey,
    /// The account's state.
    pub state: Box<CypherAccount>,
}

impl AccountContext {
    pub fn new(address: Pubkey, state: Box<CypherAccount>) -> Self {
        Self { address, state }
    }
}

/// Represents a cypher user context.
///
/// This structure allows loading [`CypherAccount`]s, their corresponding
/// [`CypherSubAccount`]s and performing certain operations with them.
///
/// Due to flexibility and implementation specific constraints, this structure
/// will not abstract any functionality related to order placement and management.
pub struct UserContext {
    pub authority: Pubkey,

    pub account_ctx: AccountContext,
    pub sub_account_ctxs: Vec<SubAccountContext>,
}

impl UserContext {
    pub fn new(
        authority: Pubkey,
        account_ctx: AccountContext,
        sub_account_ctxs: Vec<SubAccountContext>,
    ) -> Self {
        Self {
            authority,
            account_ctx,
            sub_account_ctxs,
        }
    }

    /// Creates the [`CypherAccount`] and a [`CypherSubAccount`],
    /// if an account number is provided then that [`CypherAccount`] will be created,
    /// and subsequently loaded, if not then the first account will be derived.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC
    /// request or the provided keypair's corresponding address does not have sufficient
    /// balance to create the accounts.
    pub async fn create(
        rpc_client: &Arc<RpcClient>,
        authority: &Keypair,
        clearing: &Pubkey,
        account_number: Option<u8>,
        sub_account_alias: Option<String>,
    ) -> Result<Self, ContextError> {
        let (account, account_bump, account_number) = if account_number.is_some() {
            let n = account_number.unwrap();
            let a = derive_account_address(&authority.pubkey(), n);
            (a.0, a.1, n)
        } else {
            let a = derive_account_address(&authority.pubkey(), 0);
            (a.0, a.1, 0)
        };

        let sub_accounts_alias = if sub_account_alias.is_some() {
            encode_string(&sub_account_alias.unwrap())
        } else {
            [0; 32]
        };

        // to keep it simple for now we will simply create the first sub account whenever we create
        // the master account
        let sub_account_number = 0;
        let (sub_account, sub_account_bump) =
            derive_sub_account_address(&authority.pubkey(), sub_account_number);
        let ixs = vec![
            create_account(
                clearing,
                &authority.pubkey(),
                &authority.pubkey(),
                &account,
                account_bump,
                account_number,
            ),
            create_subaccount(
                &authority.pubkey(),
                &authority.pubkey(),
                &account,
                &sub_account,
                sub_account_bump,
                sub_account_number,
                sub_accounts_alias,
            ),
        ];

        let sig = send_transactions(&rpc_client, ixs, authority, true).await;

        UserContext::load(rpc_client, &authority.pubkey(), Some(account_number)).await
    }

    /// Loads the [`CypherAccount`] and any existing [`CypherSubAccount`],
    /// if an account number is provided then that account will be loaded,
    /// if not then the first account will be derived.
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC
    /// request or any of the Accounts have an invalid Anchor discriminator.
    pub async fn load(
        rpc_client: &Arc<RpcClient>,
        authority: &Pubkey,
        account_number: Option<u8>,
    ) -> Result<Self, ContextError> {
        let account = if account_number.is_some() {
            derive_account_address(authority, account_number.unwrap()).0
        } else {
            derive_account_address(authority, 0).0
        };

        let account_state = match get_cypher_account(&rpc_client, &account).await {
            Ok(s) => s,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };

        let sub_accounts = account_state
            .sub_account_caches
            .iter()
            .filter(|a| a.sub_account != Pubkey::default())
            .map(|a| a.sub_account)
            .collect::<Vec<Pubkey>>();

        let sub_account_ctxs = if !sub_accounts.is_empty() {
            match get_multiple_cypher_program_accounts::<CypherSubAccount>(
                &rpc_client,
                &sub_accounts,
            )
            .await
            {
                Ok(s) => s
                    .iter()
                    .enumerate()
                    .map(|(idx, a)| SubAccountContext {
                        address: sub_accounts[idx],
                        state: a.clone(),
                    })
                    .collect::<Vec<SubAccountContext>>(),
                Err(e) => {
                    return Err(ContextError::ClientError(e));
                }
            }
        } else {
            Vec::new()
        };

        Ok(Self::new(
            *authority,
            AccountContext {
                address: account,
                state: account_state,
            },
            sub_account_ctxs,
        ))
    }

    /// Creates the a [`CypherSubAccount`], if an account number is provided
    /// then that [`CypherSubAccount`] will be created, if not then the first account will be derived.
    /// Calling this method also reloads the [`UserContext`].
    ///
    /// ### Errors
    ///
    /// This function will return an error if something goes wrong during the RPC
    /// request or the provided keypair's corresponding address does not have sufficient
    /// balance to create the accounts.
    pub async fn create_sub_account(
        &mut self,
        rpc_client: &Arc<RpcClient>,
        signer: &Keypair,
        sub_account_number: u8,
        sub_account_alias: Option<String>,
    ) -> Result<(), ContextError> {
        let sub_accounts_alias = if sub_account_alias.is_some() {
            encode_string(&sub_account_alias.unwrap())
        } else {
            [0; 32]
        };

        // to keep it simple for now we will simply create the first sub account whenever we create
        // the master account
        let (sub_account, sub_account_bump) =
            derive_sub_account_address(&signer.pubkey(), sub_account_number);
        let ixs = vec![create_subaccount(
            &signer.pubkey(),
            &signer.pubkey(),
            &self.account_ctx.address,
            &sub_account,
            sub_account_bump,
            sub_account_number,
            sub_accounts_alias,
        )];

        let sig = send_transactions(&rpc_client, ixs, signer, true).await;

        self.reload(rpc_client).await
    }

    /// Deposits the given SPL Token Mint.
    ///
    /// ### Assumptions
    ///
    /// - The amount specified is in the token's native units.
    /// - The user already has an Associated Token Account for the given SPL Token Mint with sufficient balance.
    ///
    /// ### Error
    ///
    /// This function will return an error if something goes wrong during the RPC
    /// request OR if it was unable to find a [`CypherSubAccount`] has a free spot position slot.
    pub async fn deposit(
        &self,
        rpc_client: &Arc<RpcClient>,
        signer: &Keypair,
        cache_account: &Pubkey,
        pool: &Pubkey,
        token_mint: &Pubkey,
        amount: u64,
    ) -> Result<Signature, ContextError> {
        let sub_account = match self.get_sub_account_with_position(token_mint) {
            Some(sa) => sa,
            None => {
                return Err(ContextError::AccountNotFound(format!(
                    "Could not find Sub Account with token mint: {}",
                    token_mint
                )))
            }
        };

        let mut ixs: Vec<Instruction> = Vec::new();
        let (pool_vault, _) = derive_pool_vault_address(pool);

        // We will simply assume that the user has an ATA for the given token mint if it is not the Wrapped SOL mint
        let (source_token_account, keypair) = if token_mint == &wrapped_sol::ID {
            // In the case where this is a Wrapped SOL deposit we will need to create a token account with rent
            // plus however much we want to deposit before depositing
            let token_account = Keypair::new();
            ixs.extend(vec![
                get_create_account_ix(
                    &signer,
                    &token_account,
                    TokenAccount::LEN,
                    &spl_token::id(),
                    Some(amount),
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &token_account.pubkey(),
                    token_mint,
                    &signer.pubkey(),
                )
                .unwrap(), // this is prone to blowing up, should do it some other way
            ]);
            (token_account.pubkey(), Some(token_account))
        } else {
            (derive_token_address(&self.authority, token_mint), None)
        };

        ixs.push(deposit_funds(
            &self.account_ctx.state.clearing,
            cache_account,
            &self.account_ctx.address,
            &sub_account.address,
            pool,
            &source_token_account,
            &pool_vault,
            token_mint,
            &signer.pubkey(),
            amount,
        ));

        // If it a Wrapped SOL deposit we can close the account after depositing
        if token_mint == &wrapped_sol::ID {
            ixs.push(
                spl_token::instruction::close_account(
                    &spl_token::id(),
                    &source_token_account,
                    &signer.pubkey(),
                    &signer.pubkey(),
                    &[&signer.pubkey()],
                )
                .unwrap(), // this too is prone to blowing up, should be done some other way
            );
        }

        let blockhash = match rpc_client.get_latest_blockhash().await {
            Ok(h) => h,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };

        let tx = if keypair.is_some() {
            create_transaction(blockhash, &ixs, signer, Some(&[&keypair.unwrap()]))
        } else {
            create_transaction(blockhash, &ixs, signer, None)
        };

        match send_transaction(rpc_client, &tx, true).await {
            Ok(s) => Ok(s),
            Err(e) => Err(ContextError::ClientError(e)),
        }
    }

    /// Withdraws the given SPL Token Mint.
    ///
    /// ### Assumptions
    ///
    /// - The amount specified is in the token's native units.
    /// - The user already has an Associated Token Account for the given SPL Token Mint.
    ///
    /// ### Error
    ///
    /// This function will return an error if something goes wrong during the RPC
    /// request OR if it was unable to find a [`CypherSubAccount`] which holds the corresponding SPL Token Mint.
    pub async fn withdraw(
        &mut self,
        rpc_client: &Arc<RpcClient>,
        signer: &Keypair,
        cache_account: &Pubkey,
        pool: &Pubkey,
        token_mint: &Pubkey,
        amount: u64,
    ) -> Result<Signature, ContextError> {
        let sub_account = match self.get_sub_account_with_position(token_mint) {
            Some(sa) => sa,
            None => {
                return Err(ContextError::AccountNotFound(format!(
                    "Could not find Sub Account with token mint: {}",
                    token_mint
                )))
            }
        };

        let mut ixs: Vec<Instruction> = Vec::new();
        let (pool_vault, _) = derive_pool_vault_address(pool);

        // We will simply assume that the user has an ATA for the given token mint if it is not the Wrapped SOL mint
        let (destination_token_account, keypair) = if token_mint == &wrapped_sol::ID {
            // In the case where this is a Wrapped SOL withdraw we will need to create a token account with rent
            // before we actually do the withdrawal
            let token_account = Keypair::new();
            ixs.extend(vec![
                get_create_account_ix(
                    &signer,
                    &token_account,
                    TokenAccount::LEN,
                    &spl_token::id(),
                    None,
                ),
                spl_token::instruction::initialize_account(
                    &spl_token::id(),
                    &token_account.pubkey(),
                    token_mint,
                    &signer.pubkey(),
                )
                .unwrap(), // this is prone to blowing up, should do it some other way
            ]);
            (token_account.pubkey(), Some(token_account))
        } else {
            (derive_token_address(&self.authority, token_mint), None)
        };

        ixs.push(withdraw_funds(
            &self.account_ctx.state.clearing,
            cache_account,
            &self.account_ctx.address,
            &sub_account.address,
            pool,
            &destination_token_account,
            &pool_vault,
            token_mint,
            &self.authority,
            amount,
        ));

        // If it a Wrapped SOL withdrawal we can close the account after it has occurred
        if token_mint == &wrapped_sol::ID {
            ixs.push(
                spl_token::instruction::close_account(
                    &spl_token::id(),
                    &destination_token_account,
                    &signer.pubkey(),
                    &signer.pubkey(),
                    &[&signer.pubkey()],
                )
                .unwrap(), // this too is prone to blowing up, should be done some other way
            );
        }

        let blockhash = match rpc_client.get_latest_blockhash().await {
            Ok(h) => h,
            Err(e) => {
                return Err(ContextError::ClientError(e));
            }
        };

        let tx = if keypair.is_some() {
            create_transaction(blockhash, &ixs, signer, Some(&[&keypair.unwrap()]))
        } else {
            create_transaction(blockhash, &ixs, signer, None)
        };

        match send_transaction(rpc_client, &tx, true).await {
            Ok(s) => Ok(s),
            Err(e) => Err(ContextError::ClientError(e)),
        }
    }

    /// Reloads this [`UserContext`] fetching all [`CypherAccount`] and [`CypherSubAccount`].
    ///
    /// ### Error
    ///
    /// This function will return an error if something goes wrong during the RPC
    /// request.
    pub async fn reload(&mut self, rpc_client: &Arc<RpcClient>) -> Result<(), ContextError> {
        self.account_ctx.state =
            match get_cypher_account(&rpc_client, &self.account_ctx.address).await {
                Ok(s) => s,
                Err(e) => {
                    return Err(ContextError::ClientError(e));
                }
            };

        let sub_accounts = self
            .account_ctx
            .state
            .sub_account_caches
            .iter()
            .filter(|a| a.sub_account != Pubkey::default())
            .map(|a| a.sub_account)
            .collect::<Vec<Pubkey>>();

        self.sub_account_ctxs = if !sub_accounts.is_empty() {
            match get_multiple_cypher_program_accounts::<CypherSubAccount>(
                &rpc_client,
                &sub_accounts,
            )
            .await
            {
                Ok(s) => s
                    .iter()
                    .enumerate()
                    .map(|(idx, a)| SubAccountContext {
                        address: sub_accounts[idx],
                        state: a.clone(),
                    })
                    .collect::<Vec<SubAccountContext>>(),
                Err(e) => {
                    return Err(ContextError::ClientError(e));
                }
            }
        } else {
            Vec::new()
        };

        Ok(())
    }

    /// Gets the sub account with the position pertaining to the given identifier.
    ///
    /// The identifier should be the SPL Token Mint pubkey for a spot position and the
    /// [`PerpetualMarket`] or [`FuturesMarket`] pubkey for a derivative position.
    pub fn get_sub_account_with_position(&self, identifier: &Pubkey) -> Option<&SubAccountContext> {
        for account in self.sub_account_ctxs.iter() {
            if !account
                .state
                .positions
                .iter()
                .filter(|p| p.derivative.market == *identifier || p.spot.token_mint == *identifier)
                .collect::<Vec<&PositionSlot>>()
                .is_empty()
            {
                return Some(&account);
            }
        }
        None
    }

    /// Gets a sub account with a free slot, if it exists and is currently loaded.
    ///
    /// ### Arguments
    /// * `is_spot` - whether the free slot should be for a spot position
    ///
    /// The identifier should be the SPL Token Mint pubkey for a spot position and the
    /// [`PerpetualMarket`] or [`FuturesMarket`] pubkey for a derivative position.
    pub fn get_sub_account_with_free_slot(&self, is_spot: bool) -> Option<&SubAccountContext> {
        for account in self.sub_account_ctxs.iter() {
            if !account
                .state
                .positions
                .iter()
                .filter(|p| {
                    if is_spot {
                        p.derivative.market == Pubkey::default()
                    } else {
                        p.spot.token_mint == Pubkey::default()
                    }
                })
                .collect::<Vec<&PositionSlot>>()
                .is_empty()
            {
                return Some(&account);
            }
        }
        None
    }
}

/// Fetches the [`CypherAccount`] with the given pubkey.
///
/// ### Error
///
/// This function will return an error if something goes wrong during the RPC
/// request or the Account has an invalid Anchor discriminator.
pub async fn get_cypher_account(
    rpc_client: &RpcClient,
    account: &Pubkey,
) -> Result<Box<CypherAccount>, ClientError> {
    match get_cypher_zero_copy_account::<CypherAccount>(rpc_client, account).await {
        Ok(s) => Ok(s),
        Err(e) => Err(e),
    }
}

/// Fetches the [`CypherSubAccount`] with the given pubkey.
///
/// ### Error
///
/// This function will return an error if something goes wrong during the RPC
/// request or the Account has an invalid Anchor discriminator.
pub async fn get_cypher_sub_account(
    rpc_client: &RpcClient,
    account: &Pubkey,
) -> Result<Box<CypherSubAccount>, ClientError> {
    match get_cypher_zero_copy_account::<CypherSubAccount>(rpc_client, account).await {
        Ok(s) => Ok(s),
        Err(e) => Err(e),
    }
}
