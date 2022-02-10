use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, AccountId, Balance, BlockHeight, Promise};
use near_sdk::{ext_contract, log, near_bindgen, PanicOnDefault};

use rand::{rngs::StdRng, Rng, SeedableRng};

mod raffle;
use raffle::*;

type Id = u64;

const YOCTO_NEAR: Balance = 1;
const GAS_COST: u64 = 5_000_000_000_000;

near_sdk::setup_alloc!();

#[ext_contract(nft_contract)]
pub trait NFT_ext_contract {
    #[payable]
    fn nft_transfer(&mut self, receiver_id: AccountId, token_id: TokenId, memo: Option<String>);
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    // block_index, random_seed for random number generation
    block_index: BlockHeight,
    random_seed: [u8; 32],

    // Raffles state
    raffles: UnorderedMap<Id, Raffle>,
    next_id: Id,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            block_index: 0,
            random_seed: [0; 32],
            raffles: UnorderedMap::new(b"a"),
            next_id: 0,
        }
    }

    pub fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: String,
        _msg: String,
    ) {
        let nft_contract = env::predecessor_account_id();
        // let participants_number =
        // let ticket_price

        // let raffle = Raffle {
        //     creator: sender_id,
        //     prize: NFT {
        //         smart_contract: nft_contract,
        //         id: token_id,
        //     },
        //     // participants_number: u32::
        //     participants: Vec::new(),
        //     // ticket_price:
        //     status: Status::Opened,
        //     winner: None,
        // };
    }

    #[payable]
    pub fn join_raffle(&mut self, raffle_id: Id) {
        assert!(
            self.raffles.get(&raffle_id).is_some(),
            "No raffle with Id - {}",
            raffle_id
        );

        let mut raffle = self.raffles.get(&raffle_id).unwrap();

        if let Status::Opened = raffle.status {
            let deposit = env::attached_deposit();

            assert!(deposit >= raffle.ticket_price, "Small deposit");

            let refund = deposit - raffle.ticket_price;
            if refund != 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }

            raffle.participants.push(env::predecessor_account_id());

            log!(
                "Participant {} joined to raffle {}",
                env::predecessor_account_id(),
                raffle_id
            );

            if raffle.participants.len() as u32 == raffle.participants_number {
                self.draw(raffle_id);
            }
        } else {
            panic!("Raffle {} is not active anymore", raffle_id);
        }
    }

    fn draw(&mut self, raffle_id: Id) {
        let mut raffle = self.raffles.get(&raffle_id).unwrap();

        let p_number = raffle.participants_number;
        let t_price = raffle.ticket_price;

        let random = self.generate_random(p_number);
        let winner = raffle.participants[random].clone();

        Promise::new(raffle.creator.clone()).transfer((p_number as u128) * t_price);

        let token_id: TokenId = raffle.prize.clone().id;
        let nft_contract = raffle.prize.smart_contract.clone();

        log!("Winner is {}", winner);

        raffle.status = Status::Closed;
        raffle.winner = Some(winner.clone());

        self.raffles.insert(&raffle_id, &raffle);

        // NFT transfer to winner
        nft_contract::nft_transfer(winner, token_id, None, &nft_contract, YOCTO_NEAR, GAS_COST);
    }

    // Random number generation
    fn generate_random(&mut self, high: u32) -> usize {
        if env::block_index() != self.block_index {
            self.block_index = env::block_index();
            self.random_seed = env::random_seed().try_into().unwrap();
        }

        let mut rng: StdRng = SeedableRng::from_seed(self.random_seed);
        self.random_seed[0] += 1;

        let random: usize = rng.gen_range(0, high) as usize;

        log!("Generated number: {}", random);

        random
    }

    pub fn active_raffles(&self) -> Vec<(Id, Raffle)> {
        self.raffles
            .iter()
            .filter(|(_, v)| {
                if let Status::Opened = v.status {
                    true
                } else {
                    false
                }
            })
            .map(|(k, v)| (k, v))
            .collect()
    }

    pub fn get_raffle_by_id(&self, raffle_id: Id) -> Raffle {
        self.raffles
            .get(&raffle_id)
            .expect(&format!("No raffle with Id - {}", raffle_id))
    }
}
