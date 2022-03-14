use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, BlockHeight, PanicOnDefault, Promise,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

mod raffle;
use raffle::*;

type Id = u64;

const YOCTO_NEAR: Balance = 1;
const GAS_COST: u64 = 5_000_000_000_000;

near_sdk::setup_alloc!();

// External NFT-contract for cross-contract call
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
        assert!(!env::state_exists(), "Contract already initialized");

        log!("Contract is initialized");

        Self {
            block_index: 0,
            random_seed: [0; 32],
            raffles: UnorderedMap::new(b"a"),
            next_id: 0,
        }
    }

    pub fn nft_on_transfer(
        &mut self,
        _sender: AccountId,
        previous_owner_id: AccountId,
        token_id: String,
        msg: String,
    ) -> bool {
        let nft_contract = env::predecessor_account_id();

        // Parsing memo msg from nft-transfer-call function
        let participants_number = &msg[..5];
        let ticket_price = &msg[6..];

        let participants_number: u32 = participants_number.parse().unwrap();
        let ticket_price: Balance = ticket_price.parse().unwrap();

        // Initialize the raffle
        let raffle = Raffle {
            creator: previous_owner_id,
            prize: NFT {
                smart_contract: nft_contract,
                id: token_id,
            },
            participants_number,
            participants: Vec::new(),
            ticket_price,
            status: Status::Opened,
            winner: None,
        };

        let id = self.next_id;
        self.next_id += 1;

        self.raffles.insert(&id, &raffle);

        // Log an event with raffle Id for client
        log!("");

        // Do not return NFT back to the owner
        false
    }

    #[payable]
    pub fn join_raffle(&mut self, raffle_id: Id) {
        // Get raffle by Id
        let mut raffle = self
            .raffles
            .get(&raffle_id)
            .expect("No raffle with such Id");

        // If the raffle is closed => panic!
        if let Status::Opened = raffle.status {
            // Check the deposit amount
            let deposit = env::attached_deposit();

            assert!(deposit >= raffle.ticket_price, "Small deposit");

            // Refund, if the deposit is > than ticket_price
            let refund = deposit - raffle.ticket_price;
            if refund != 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }

            // Add participant
            raffle.participants.push(env::predecessor_account_id());

            log!(
                "Participant {} joined to raffle {}",
                env::predecessor_account_id(),
                raffle_id
            );

            // If the participant is the last one in this raffle => draw
            if raffle.participants.len() as u32 == raffle.participants_number {
                self.draw(raffle_id);
            }
        } else {
            panic!("Raffle is not active anymore");
        }
    }

    // Raffle draw function
    fn draw(&mut self, raffle_id: Id) {
        let mut raffle = self.raffles.get(&raffle_id).unwrap();

        let p_number = raffle.participants_number;
        let t_price = raffle.ticket_price;

        let random = self.generate_random(p_number);
        let winner = raffle.participants[random].clone();

        // Amount of NEAR, return to the creator
        let amount = (p_number as u128) * t_price;

        // Transfer money from tickets to raffle creator
        Promise::new(raffle.creator.clone()).transfer(amount);

        log!(
            "Money ({} YoctoNEAR) was transferred to a raffle creator ({})",
            amount,
            raffle.creator
        );

        let token_id: TokenId = raffle.prize.clone().id;
        let nft_contract = raffle.prize.smart_contract.clone();

        // Set the winner and close the raffle
        raffle.winner = Some(winner.clone());
        raffle.status = Status::Closed;

        log!("Winner is {}", winner);

        self.raffles.insert(&raffle_id, &raffle);

        // NFT transfer to winner; cross-contract call
        nft_contract::nft_transfer(winner, token_id, None, &nft_contract, YOCTO_NEAR, GAS_COST);
    }

    // Random number generation
    fn generate_random(&mut self, high: u32) -> usize {
        // If this transaction is in the new block
        // we can use new random_seed
        if env::block_index() != self.block_index {
            self.block_index = env::block_index();
            self.random_seed = env::random_seed().try_into().unwrap();
        }

        // Creating generator from current random_seed
        let mut rng: StdRng = SeedableRng::from_seed(self.random_seed);

        // Changing random_seed for next transaction
        // because if it's in the same block, it will give us the same output
        self.random_seed[0] = self.random_seed[0].wrapping_add(1);

        // Random number on interval [0; high)
        let random: usize = rng.gen_range(0, high) as usize;

        log!("Generated number: {}", random);

        random
    }

    // Return JSON Data of currently active raffles
    pub fn active_raffles(&self) -> Vec<(Id, Raffle)> {
        self.raffles
            .iter()
            .filter(|(_, v)| matches!(v.status, Status::Opened))
            .map(|(k, v)| (k, v))
            .collect()
    }

    // Return JSON Data of raffle by given ID
    pub fn get_raffle_by_id(&self, raffle_id: Id) -> Raffle {
        self.raffles
            .get(&raffle_id)
            .expect("No raffle with such Id")
    }
}
