use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, AccountId, BlockHeight};
use near_sdk::{log, near_bindgen, PanicOnDefault};

use rand::{rngs::StdRng, Rng, SeedableRng};

mod raffle;
use raffle::*;

type Id = u64;

near_sdk::setup_alloc!();

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

    #[payable]
    pub fn join_raffle(&mut self, raffle_id: Id) {
        assert!(
            self.raffles.get(&raffle_id).is_some(),
            "No raffle with ID - {}",
            raffle_id
        );

        let deposit = env::attached_deposit();

        let mut raffle = self.raffles.get(&raffle_id).unwrap();
        assert_eq!(raffle.ticket_price, deposit, "Wrong deposit");

        raffle.participants.push(env::predecessor_account_id());

        log!(
            "{} is now participant in raffle {}",
            env::predecessor_account_id(),
            raffle_id
        );
    }

    // pub fn

    pub fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: String,
        _msg: String,
    ) {
    }

    pub(crate) fn generate_random(&mut self, low: u64, high: u64) -> u64 {
        if env::block_index() != self.block_index {
            self.block_index = env::block_index();
            self.random_seed = env::random_seed().try_into().unwrap();
        }

        let mut rng: StdRng = SeedableRng::from_seed(self.random_seed);
        self.random_seed.iter_mut().for_each(|x| *x += 1);

        let random = rng.gen_range(low, high);

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
