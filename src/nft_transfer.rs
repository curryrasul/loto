use super::*;
use crate::TokenId;

// External NFT-contract for cross-contract call
#[ext_contract(nft_contract)]
pub trait ext_contract {
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: Option<String>,
    );
}
