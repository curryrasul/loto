use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::{AccountId, Balance};

pub type TokenId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[allow(clippy::upper_case_acronyms)]
pub struct NFT {
    pub smart_contract: AccountId,
    pub id: TokenId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Status {
    Opened,
    Closed,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Raffle {
    pub creator: AccountId,
    pub prize: NFT,
    pub participants_number: u32,
    pub participants: Vec<AccountId>,
    pub ticket_price: Balance,
    pub status: Status,
    pub winner: Option<AccountId>,
}
