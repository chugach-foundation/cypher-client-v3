use anchor_lang::prelude::Pubkey;
use solana_sdk::{
    address_lookup_table_account::AddressLookupTableAccount,
    hash::Hash,
    instruction::Instruction,
    message::{v0, CompileError, Message, VersionedMessage},
    signature::Keypair,
    signer::{Signer, SignerError},
    transaction::{Transaction, VersionedTransaction},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Signer Error: {:?}", self)]
    SignerError(SignerError),
    #[error("Compile Error: {:?}", self)]
    CompileError(CompileError),
}

#[derive(Debug, Default)]
pub struct TransactionBuilder {
    pub ixs: Vec<Instruction>,
}

impl TransactionBuilder {
    pub fn new() -> TransactionBuilder {
        TransactionBuilder::default()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.ixs.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.ixs.is_empty()
    }

    #[inline(always)]
    pub fn add(&mut self, ix: Instruction) {
        self.ixs.push(ix);
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.ixs.clear();
    }

    #[inline(always)]
    pub fn build(
        &self,
        recent_blockhash: Hash,
        payer: &Keypair,
        additional_signers: Option<&Vec<Keypair>>,
    ) -> Transaction {
        let message = Message::new(&self.ixs[..], Some(&payer.pubkey()));
        let mut txn = Transaction::new_unsigned(message);
        txn.partial_sign(&[payer], recent_blockhash);
        if let Some(adsigners) = additional_signers {
            for adsigner in adsigners {
                txn.partial_sign(&[adsigner], recent_blockhash);
            }
        }
        txn
    }

    #[inline(always)]
    pub fn build_versioned(
        &self,
        recent_blockhash: Hash,
        payer: &Keypair,
        additional_signers: Option<&Vec<Keypair>>,
        lookup_table_address: &Pubkey,
        lookup_table: AddressLookupTableAccount,
    ) -> Result<VersionedTransaction, Error> {
        let message = match v0::Message::try_compile(
            &payer.pubkey(),
            &self.ixs[..],
            &[lookup_table],
            recent_blockhash,
        ) {
            Ok(m) => m,
            Err(e) => {
                return Err(Error::CompileError(e));
            }
        };

        let mut all_signers = vec![payer];
        if let Some(adsigners) = additional_signers {
            all_signers.extend(adsigners);
        }

        match VersionedTransaction::try_new(VersionedMessage::V0(message), &[payer]) {
            Ok(t) => Ok(t),
            Err(e) => {
                return Err(Error::SignerError(e));
            }
        }
    }
}
