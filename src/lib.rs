use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, BlockHeight};
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
}
