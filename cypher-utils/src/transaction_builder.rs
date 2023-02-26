use solana_sdk::{
    hash::Hash, instruction::Instruction, message::Message, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

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
}
