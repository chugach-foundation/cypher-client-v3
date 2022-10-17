use {
    base64::DecodeError,
    solana_account_decoder::{UiAccount, UiAccountEncoding},
};

#[derive(Debug, PartialEq)]
pub enum AccountDecodingError {
    InvalidAccountResponseFormat,
    InvalidAccountDataEncoding,
    AccountInfoDecoding(DecodeError),
}

pub fn get_account_info(account: &UiAccount) -> Result<Vec<u8>, AccountDecodingError> {
    let (ai, enc) = match &account.data {
        solana_account_decoder::UiAccountData::Binary(s, e) => (s, *e),
        _ => return Err(AccountDecodingError::InvalidAccountResponseFormat),
    };

    if enc != UiAccountEncoding::Base64 {
        return Err(AccountDecodingError::InvalidAccountDataEncoding);
    }

    let account_data_res = base64::decode(ai);
    match account_data_res {
        Ok(a) => Ok(a),
        Err(e) => return Err(AccountDecodingError::AccountInfoDecoding(e)),
    }
}
