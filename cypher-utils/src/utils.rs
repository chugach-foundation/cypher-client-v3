use anchor_client::anchor_lang::{AccountDeserialize, AccountSerialize, Discriminator};
use log::warn;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::RpcFilterType,
};
use solana_sdk::{
    account::Account, commitment_config::CommitmentConfig, hash::Hash, instruction::Instruction,
    rent::Rent, signature::Signature, signer::Signer, system_instruction, transaction::Transaction,
};
use thiserror::Error;

use crate::transaction_builder::TransactionBuilder;

use {
    anchor_client::anchor_lang::{Owner, ZeroCopy},
    cypher_client::utils::get_zero_copy_account,
    solana_client::{client_error::ClientError, nonblocking::rpc_client::RpcClient},
    solana_sdk::{pubkey::Pubkey, signature::Keypair},
    std::{fs::File, io::Read, str::FromStr},
};

#[derive(Debug, Error)]
pub enum KeypairError {
    #[error("Error opening keypair file.")]
    FileOpen,
    #[error("Error reading keypair file.")]
    FileRead,
    #[error("Provided keypair file contents do not match keypair length.")]
    SizeMismatch,
    #[error("Error loading keypair.")]
    Load,
}

/// Encodes a string into an array of bytes fixed with 32 length.
#[inline(always)]
pub fn encode_string(alias: &str) -> [u8; 32] {
    let mut encoded = [0_u8; 32];
    let alias_bytes = alias.as_bytes();
    assert!(alias_bytes.len() <= 32);
    for (i, byte) in alias_bytes.iter().enumerate() {
        encoded[i] = *byte;
    }
    encoded
}

/// The length in bytes of a keypair, to match the underlying Ed25519 Keypair.
pub const KEYPAIR_LENGTH: usize = 64;

/// Loads a Solana [`Keypair`] from a file at the given path.
///
/// ### Errors
///
/// This function will return an error if something goes wrong while attempting to open or
/// read the file, or finally in case the [`Keypair`] bytes in the file are invalid.
///
/// ### Format
///
/// The file should have the following format, and in total should have [`KEYPAIR_LENGTH`] bytes.
///
/// \[123, 34, 78, 0, 1, 3, 45 (...)\]
#[inline(always)]
pub fn load_keypair(path: &str) -> Result<Keypair, KeypairError> {
    let fd = File::open(path);

    let mut file = match fd {
        Ok(f) => f,
        Err(_) => {
            return Err(KeypairError::FileOpen);
        }
    };

    let file_string = &mut String::new();
    let file_read_res = file.read_to_string(file_string);

    let _ = if let Err(_) = file_read_res {
        return Err(KeypairError::FileRead);
    };

    let keypair_bytes: Vec<u8> = file_string
        .replace('[', "")
        .replace(']', "")
        .replace(',', " ")
        .split(' ')
        .map(|x| u8::from_str(x).unwrap())
        .collect();

    if keypair_bytes.len() != KEYPAIR_LENGTH {
        return Err(KeypairError::SizeMismatch);
    }

    let keypair = Keypair::from_bytes(keypair_bytes.as_ref());

    match keypair {
        Ok(kp) => Ok(kp),
        Err(_) => Err(KeypairError::Load),
    }
}

/// Gets all program accounts according to the given filters for the given program.
pub async fn get_program_accounts(
    rpc_client: &RpcClient,
    filters: Vec<RpcFilterType>,
    program_id: &Pubkey,
) -> Result<Vec<(Pubkey, Account)>, ClientError> {
    let accounts_res = rpc_client
        .get_program_accounts_with_config(
            program_id,
            RpcProgramAccountsConfig {
                filters: Some(filters),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    commitment: Some(CommitmentConfig::default()),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .await;

    match accounts_res {
        Ok(a) => Ok(a),
        Err(e) => Err(e),
    }
}

/// Gets an Account's state and attempts decoding it into the given Account type.
///
/// ### Errors
///
/// This function will return an error if something goes wrong with the RPC request
/// or the given account has an invalid Anchor discriminator for the given type.
#[inline(always)]
pub async fn get_cypher_program_account<
    T: AccountSerialize + AccountDeserialize + Discriminator + Clone + Owner,
>(
    rpc_client: &RpcClient,
    account: &Pubkey,
) -> Result<Box<T>, ClientError> {
    let account_res = rpc_client.get_account_data(account).await;
    let account_data = match account_res {
        Ok(a) => a,
        Err(e) => {
            return Err(e);
        }
    };

    let state = Box::new(<T>::try_deserialize(&mut account_data.as_slice()).unwrap());

    Ok(state)
}

/// Gets an Account's state and attempts decoding it into the given Account type.
///
/// ### Errors
///
/// This function will return an error if something goes wrong with the RPC request
/// or the given account has an invalid Anchor discriminator for the given type.
#[inline(always)]
pub async fn get_cypher_zero_copy_account<T: ZeroCopy + Owner>(
    rpc_client: &RpcClient,
    account: &Pubkey,
) -> Result<Box<T>, ClientError> {
    let account_res = rpc_client.get_account_data(account).await;
    let account_data = match account_res {
        Ok(a) => a,
        Err(e) => {
            return Err(e);
        }
    };

    let state = get_zero_copy_account::<T>(&account_data);

    Ok(state)
}

/// Gets multiple Account's state and attempts decoding them into the given Account type.
///
/// ### Errors
///
/// This function will return an error if something goes wrong with the RPC request
/// or the given accounts have an invalid Anchor discriminator for the given type.
#[inline(always)]
pub async fn get_multiple_cypher_program_accounts<
    T: AccountSerialize + AccountDeserialize + Discriminator + Clone + Owner,
>(
    rpc_client: &RpcClient,
    accounts: &[Pubkey],
) -> Result<Vec<Box<T>>, ClientError> {
    let account_res = rpc_client.get_multiple_accounts(accounts).await;
    let account_datas = match account_res {
        Ok(a) => a,
        Err(e) => {
            return Err(e);
        }
    };

    let states = account_datas
        .iter()
        .filter(|a| a.is_some())
        .map(|a| Box::new(<T>::try_deserialize(&mut a.as_ref().unwrap().data.as_slice()).unwrap()))
        .collect::<Vec<Box<T>>>();

    Ok(states)
}

/// Gets multiple Account's state and attempts decoding them into the given Account type.
///
/// ### Errors
///
/// This function will return an error if something goes wrong with the RPC request
/// or the given accounts have an invalid Anchor discriminator for the given type.
#[inline(always)]
pub async fn get_multiple_cypher_zero_copy_accounts<T: ZeroCopy + Owner>(
    rpc_client: &RpcClient,
    accounts: &[Pubkey],
) -> Result<Vec<Box<T>>, ClientError> {
    let account_res = rpc_client.get_multiple_accounts(accounts).await;
    let account_datas = match account_res {
        Ok(a) => a,
        Err(e) => {
            return Err(e);
        }
    };

    let states = account_datas
        .iter()
        .filter(|a| a.is_some())
        .map(|a| get_zero_copy_account::<T>(&a.as_ref().unwrap().data))
        .collect::<Vec<Box<T>>>();

    Ok(states)
}

#[inline(always)]
pub async fn send_transactions(
    rpc_client: &RpcClient,
    ixs: Vec<Instruction>,
    signer: &Keypair,
    confirm: bool,
) -> Result<Vec<Signature>, ClientError> {
    let mut txn_builder = TransactionBuilder::new();
    let mut submitted: bool = false;
    let mut signatures: Vec<Signature> = Vec::new();
    let mut prev_tx: Transaction = Transaction::default();

    let blockhash = rpc_client.get_latest_blockhash().await.unwrap();

    for ix in ixs {
        if txn_builder.len() != 0 {
            let tx = txn_builder.build(blockhash, signer, None);
            // we do this to attempt to pack as many ixs in a tx as possible
            // there's more efficient ways to do it but we'll do it in the future
            if tx.message_data().len() > 1000 {
                let res = send_transaction(rpc_client, &prev_tx, confirm).await;
                match res {
                    Ok(s) => {
                        submitted = true;
                        txn_builder.clear();
                        signatures.push(s);
                    }
                    Err(e) => {
                        warn!(
                            "There was an error submitting transaction: {}",
                            e.to_string()
                        );
                    }
                }
            } else {
                txn_builder.add(ix);
                prev_tx = tx;
            }
        } else {
            txn_builder.add(ix);
        }
    }

    if !submitted || txn_builder.len() != 0 {
        let tx = txn_builder.build(blockhash, signer, None);
        let res = send_transaction(rpc_client, &tx, confirm).await;
        match res {
            Ok(s) => {
                signatures.push(s);
            }
            Err(e) => {
                warn!(
                    "There was an error submitting transaction: {}",
                    e.to_string()
                );
                let err = e.get_transaction_error().unwrap();
                warn!("Error: {}", err.to_string());
                return Err(e);
            }
        }
    }

    Ok(signatures)
}

/// Sends a transaction
#[inline(always)]
pub async fn send_transaction(
    rpc_client: &RpcClient,
    tx: &Transaction,
    confirm: bool,
) -> Result<Signature, ClientError> {
    let submit_res = if confirm {
        rpc_client.send_and_confirm_transaction(tx).await
    } else {
        rpc_client.send_transaction(tx).await
    };
    match submit_res {
        Ok(s) => Ok(s),
        Err(e) => Err(e),
    }
}

/// Creates a transaction with the given blockhash, instructions, payer and signers.
pub fn create_transaction(
    blockhash: Hash,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: Option<&[&Keypair]>,
) -> Transaction {
    let mut all_signers = vec![payer];
    if let Some(signers) = signers {
        all_signers.extend_from_slice(signers);
    }
    let mut transaction = Transaction::new_with_payer(ixs, Some(&payer.pubkey()));

    transaction.sign(&all_signers, blockhash);
    transaction
}

/// Gets the System Program's CreateAccount instruction with the given parameters.
pub fn get_create_account_ix(
    payer: &Keypair,
    target: &Keypair,
    space: usize,
    pid: &Pubkey,
    extra_rent: Option<u64>,
) -> Instruction {
    let rent = if extra_rent.is_some() {
        Rent::default().minimum_balance(space) + extra_rent.unwrap()
    } else {
        Rent::default().minimum_balance(space)
    };
    system_instruction::create_account(&payer.pubkey(), &target.pubkey(), rent, space as u64, pid)
}
