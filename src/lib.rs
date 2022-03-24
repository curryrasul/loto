use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId, Balance, PanicOnDefault, Promise};
use rand::{rngs::StdRng, Rng, SeedableRng};

mod consts;
mod event;
mod nft_transfer;
mod raffle;
mod types;

use consts::*;
use event::*;
use nft_transfer::*;
use raffle::*;
use types::*;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
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
            raffles: UnorderedMap::new(b"a"),
            next_id: 0,
        }
    }

    pub fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: String,
        msg: String,
    ) -> bool {
        assert_ne!(
            sender_id,
            env::predecessor_account_id(),
            "Only NFT-contract can call this function"
        );

        let nft_contract = env::predecessor_account_id();

        // Parsing message arguments
        let split: Vec<&str> = msg.split(',').into_iter().collect();
        if split.len() != 2 {
            log!("Wrong message format");
            return true;
        }

        let participants_number = split[0].parse::<u32>();
        let ticket_price = split[1].parse::<Balance>();

        if !matches!(participants_number, Ok(_)) || !matches!(ticket_price, Ok(_)) {
            log!("Wrong message format");
            return true;
        }

        let participants_number = participants_number.unwrap();

        if participants_number == 0 || participants_number == 1 {
            log!("Minimum number of participants is 2");
            return true;
        }

        let ticket_price = ticket_price.unwrap() * ONE_NEAR;

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

        let event = Event { raffle_id: id };

        // Log an event with raffle Id for client
        log!("{}", event);

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
            // Check if the participant is not the creator
            assert_ne!(
                raffle.creator,
                env::predecessor_account_id(),
                "Creator cannot register himself"
            );

            // Check the deposit amount
            let deposit = env::attached_deposit();

            assert!(deposit >= raffle.ticket_price, "Small deposit");

            // Refund, if the deposit is > than ticket_price
            let refund = deposit - raffle.ticket_price;
            if refund != 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
                log!("{} $NEAR was refunded", refund / ONE_NEAR);
            }

            // Add participant
            raffle.participants.push(env::predecessor_account_id());

            log!(
                "Participant {} joined to raffle {}",
                env::predecessor_account_id(),
                raffle_id
            );

            self.raffles.insert(&raffle_id, &raffle);

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

        let random = self.generate_random(p_number) as usize;
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
        nft_contract::nft_transfer(
            winner,
            token_id,
            0,
            None,
            &nft_contract,
            YOCTO_NEAR,
            GAS_COST,
        );
    }

    // Random number generation
    fn generate_random(&mut self, high: u32) -> u32 {
        // Creating generator from current random_seed
        let mut rng: StdRng = SeedableRng::from_seed(env::random_seed().try_into().unwrap());

        // Random number on interval [0; high)
        let random: u32 = rng.gen_range(0, high);

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
