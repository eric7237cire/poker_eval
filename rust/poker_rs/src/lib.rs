//! RS-Poker is a library for poker
//! Currently RS-Poker supports:
//!
//! * Hand Iteration.
//! * Hand Ranking.
//! * Hand Range parsing.
//! * Hand Range generation.
//! * ICM tournament values
//! * Monte carlo holdem
//! * Holdem Game State with action validation
//! * Holdem agents
//! * Holdem game simulation
//!
//! Our focus is on correctness and performance.
//!
//! ## Core library
//!
//! The core of the library contains code that is relevant
//! to all poker variants. Card suits, values, hand
//! values, and datastructures used in other parts of the crate.
//!
//! ## Holdem
//!
//! Holdem is the best supported variant.
//!
//! ### Starting Hands
//!
//! The StartingHand module contains the following key components:
//!
//! `Suitedness`: This is an enum type that represents how the suits of a hand
//! correspond to each other. It has three variants:
//!
//! * Suited: All of the cards are the same suit.
//! * OffSuit: None of the cards are the same suit.
//! * Any: Makes no promises about the suit.
//!
//! `HoldemStartingHand`: This represents the two-card starting hand of
//! Texas Hold'em. It can generate all possible actual starting
//! hands given two values and a suitedness condition.
//!
//! ```rust
//! use rs_poker::core::Value;
//! use rs_poker::holdem::{StartingHand, Suitedness};
//!
//! let hands = StartingHand::default(Value::Ace, Value::Ace, Suitedness::OffSuit).possible_hands();
//! assert_eq!(6, hands.len());
//! ```
//!
//! ### Range parsing
//!
//! A lot of discussion online is around ranges. For example:
//!  "High Jack had a range of KQo+ and 99+"
//!
//! The range parsing module allows turning those range strings into vectors of
//! possible hands.
//!
//! ```
//! use rs_poker::holdem::RangeParser;
//! let hands = RangeParser::parse_one("KQo+").unwrap();
//!
//! // There are 24 different combinations of off suit
//! // connectors King + Queen or higher
//! assert_eq!(24, hands.len());
//! ```
//! ### Monte Carlo Game simulation
//!
//! Sometimes it's important to know your expected equity
//! in a pot vs a given set of card. In doing that it's useful
//! to quickly simulate what could happen.
//!
//! The `MonteCarloGame` strcut does that:
//!
//! ``` rust
//! use rs_poker::core::{Card, Hand, Suit, Value};
//! use rs_poker::holdem::MonteCarloGame;
//!
//! let hero = Hand::new_with_cards(vec![
//!     Card::new(Value::Jack, Suit::Spade),
//!     Card::new(Value::Jack, Suit::Heart),
//! ]);
//! let villan = Hand::new_with_cards(vec![
//!     Card::new(Value::Ace, Suit::Spade),
//!     Card::new(Value::King, Suit::Spade),
//! ]);
//! let mut monte_sim = MonteCarloGame::new(vec![hero, villan]).unwrap();
//! let mut wins: [u64; 2] = [0, 0];
//! for _ in 0..100_000 {
//!     let r = monte_sim.simulate();
//!     monte_sim.reset();
//!     // You can handle ties however you like here
//!     wins[r.0.ones().next().unwrap()] += 1
//! }
//!
//! // Jacks hold up most of the time
//! assert!(wins[0] > wins[1]);
//! ```
//!
//! ## Simulated ICM
//!
//! Not all chips are equal; when rewards for tounaments are highly in favor of
//! placing higher, sometimes the correct decision comes down to expected value
//! of the whole tournament.
//!
//! ```
//! use rand::{thread_rng, Rng};
//! use rs_poker::simulated_icm::simulate_icm_tournament;
//!
//! let payments = vec![10_000, 6_000, 4_000, 1_000, 800];
//! let mut rng = thread_rng();
//! let chips: Vec<i32> = (0..4).map(|_| rng.gen_range(100..300_000)).collect();
//! let simulated_results = simulate_icm_tournament(&chips, &payments);
//!
//! // There's one payout per player still remaining.
//! // You can run this over and over again to get an
//! // average expected value.
//! assert_eq!(chips.len(), simulated_results.len());
//! ```
//!
//! ## Holdem arena
//!
//! The holdem arena is the newest addition to `rs-poker` and the most
//! experimental. So it's the most likely to change in the future.
//!
//! The arena is code to simulate different strategies and get outcomes. For
//! example if you want to simulate the different between different vpip's.
//! Simply code an agent with configurable starting hand range and see what the
//! expected values are. The arena is configurable for number of players from
//! heads up all the way to full ring.
//!
//! The arena has three main parts:
//! * `GameState` this holds the current state of all the chips, bets, player
//!   all in status, and if players are active in a hand or round.
//! * `Agent` is the trait needed to implement different automatic players in
//!   the poker.
//! * `HoldemSimulation` this is the main wrapper struct that handles calling
//!   the agents and force folding the agents for any invalid actions.
#![deny(clippy::all)]
extern crate rand;

/// Allow all the core poker functionality to be used
/// externally. Everything in core should be agnostic
/// to poker style.
pub mod core;
/// The holdem specific code. This contains range
/// parsing, game state, and starting hand code.
pub mod holdem;

/// Given a tournament calculate the implied
/// equity in the total tournament.
pub mod simulated_icm;

/// Simulate poker games via agents that
/// play. Then determine who wins the most over
/// time
#[cfg(feature = "arena")]
pub mod arena;
