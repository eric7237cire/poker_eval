//maintain player states and game state
// and we have 2 playbacks, either agent or a log
// via trait that will do

use std::cmp::{max, min};

use crate::{rank_cards, set_used_card, PlayerAction, Rank};
use crate::{
    ActionEnum, Card, CardUsedType, ChipType, GameLog, GameState, HoleCards, InitialPlayerState,
    PlayerState, PokerError, Position, Round,
};

use crate::game::game_runner_source::GameRunnerSource;
use crate::game::game_runner_source::GameRunnerSourceEnum;
use log::trace;

// Enforces the poker rules
pub struct GameRunner {
    used_cards: CardUsedType,

    pub game_state: GameState,

    // Source of actions, cards
    pub game_runner_source: GameRunnerSourceEnum,
}

impl GameRunner {
    pub fn new(game_runner_source: GameRunnerSourceEnum) -> Result<Self, PokerError> {
        let initial_players = game_runner_source.get_initial_players();

        if initial_players.len() < 2 {
            return Err(PokerError::from_string(format!(
                "Invalid number of players {}",
                initial_players.len()
            )));
        }

        let player_states = initial_players
            .iter()
            .map(|p| PlayerState::new(&p))
            .collect();

        let sb = game_runner_source.get_small_blind();
        let bb = game_runner_source.get_big_blind();

        let game_state = GameState {
            player_states: player_states,
            current_to_act: Position::first_to_act(
                initial_players.len() as _,
                crate::Round::Preflop,
            ),
            prev_round_pot: 0,
            round_pot: 0,
            current_to_call: Some(bb),
            current_round: Round::Preflop,
            board: Vec::new(),
            sb,
            bb,
            actions: Vec::new(),
            min_raise: 0,
        };

        let mut r = GameRunner {
            game_state,
            game_runner_source,
            used_cards: CardUsedType::default(),
        };

        r.handle_blinds()?;
        r.init_used_hole_cards()?;

        Ok(r)
    }

    fn handle_blinds(&mut self) -> Result<(), PokerError> {
        let sb = self.game_state.sb;
        let bb = self.game_state.bb;
        self.handle_put_money_in_pot(0, sb)?;
        self.handle_put_money_in_pot(1, bb)?;

        self.game_state.min_raise = bb;

        Ok(())
    }

    fn init_used_hole_cards(&mut self) -> Result<(), PokerError> {
        for player_index in 0..self.game_state.player_states.len() {
            let hole_cards = self.game_runner_source.get_hole_cards(player_index)?;

            hole_cards.set_used(&mut self.used_cards)?;
        }

        Ok(())
    }

    //Note this puts the difference in the pot
    //This is make total chips this player has put into the pot this round == amount
    fn handle_put_money_in_pot(
        &mut self,
        player_index: usize,
        amount: ChipType,
    ) -> Result<ChipType, PokerError> {
        if player_index >= self.game_state.player_states.len() {
            return Err(PokerError::from_string(format!(
                "Invalid player index {}",
                player_index
            )));
        }
        let player_state = &mut self.game_state.player_states[player_index];

        if amount < player_state.cur_round_putting_in_pot.unwrap_or(0) {
            return Err(PokerError::from_string(format!(
                "Player {} tried to put {} in pot, but already put in {}",
                player_state.player_name,
                amount,
                player_state.cur_round_putting_in_pot.unwrap_or(0)
            )));
        }

        let max_actual_amount = amount - player_state.cur_round_putting_in_pot.unwrap_or(0);

        let actual_amount = if player_state.stack <= max_actual_amount {
            player_state.all_in = true;

            //max_pot is created when the round is done
            player_state.stack
        } else {
            max_actual_amount
        };

        assert!(player_state.stack >= actual_amount);

        player_state.stack -= actual_amount;
        player_state.cur_round_putting_in_pot =
            Some(player_state.cur_round_putting_in_pot.unwrap_or(0) + actual_amount);

        if max_actual_amount == actual_amount {
            assert_eq!(player_state.cur_round_putting_in_pot, Some(amount));
        }

        self.game_state.round_pot += actual_amount;

        Ok(actual_amount)
    }

    fn active_player_count(&self) -> usize {
        self.game_state
            .player_states
            .iter()
            .filter(|p| p.is_active())
            .count()
    }

    fn calc_max_pot(&self, all_in_for: ChipType) -> ChipType {
        let mut max_pot = 0;

        for player_state in &self.game_state.player_states {
            let money_put_in = player_state.initial_stack - player_state.stack;
            max_pot += min(money_put_in, all_in_for)
        }
        max_pot
    }

    fn check_pots_good(&self) -> Result<(), PokerError> {
        trace!("Check pots good in round {}", self.game_state.current_round);

        let mut check_round_pot = 0;

        //Do some sanity checks, each player either folded or put in the same amount or is all in
        for player_state in &self.game_state.player_states {
            let cur_round_putting_in_pot = player_state.cur_round_putting_in_pot.ok_or(format!(
                "Player {} has no cur_round_putting_in_pot, has not acted yet",
                player_state.player_name
            ))?;

            // trace!(
            //     "{} has put in {} this round.  Folded? {} All in? {} Initial {} Current {}",
            //     player_state.player_name,
            //     cur_round_putting_in_pot,
            //     player_state.folded,
            //     player_state.all_in,
            //     player_state.initial_stack,
            //     player_state.stack
            // );

            check_round_pot += cur_round_putting_in_pot;
            if !player_state.is_active() {
                continue;
            }
            if let Some(current_to_call) = self.game_state.current_to_call {
                if cur_round_putting_in_pot != current_to_call {
                    return Err(format!(
                        "Player {} has put in {} but current to call is {}",
                        player_state.player_name, cur_round_putting_in_pot, current_to_call
                    )
                    .into());
                }
            }
        }

        if check_round_pot != self.game_state.round_pot {
            return Err(format!(
                "Round pot is {} but sum of player pots is {}",
                self.game_state.round_pot, check_round_pot
            )
            .into());
        }

        Ok(())
    }

    fn move_if_needed_next_to_act(&mut self) -> Result<(), PokerError> {
        let n_players = self.game_state.player_states.len();
        for _ in 0..n_players {
            let player_index: usize = self.game_state.current_to_act.into();
            if self.game_state.player_states[player_index].is_active() {
                return Ok(());
            }

            trace!(
                "Player #{} named {} is all in or folded, moving to next player",
                player_index,
                &self.game_state.player_states[player_index].player_name
            );

            self.game_state.current_to_act = self
                .game_state
                .current_to_act
                .next(self.game_state.player_states.len() as _);
        }

        Err(format!("No players left to act").into())
    }

    fn close_betting_round(&mut self) -> Result<(), PokerError> {
        trace!("Close betting round");

        trace!("Check all players have called/folded/or are all in");

        self.check_pots_good()?;
        //calculate max_pot
        let player_count = self.game_state.player_states.len();
        // for player_index in 0..self.game_state.player_states.len() {
        //     if let Some(all_in) = self.game_state.player_states[player_index].all_in_for {
        //         let max_pot = self.calc_max_pot(all_in);
        //         self.game_state.player_states[player_index].max_pot = Some(max_pot);
        //     }
        // }

        //set cur round putting in pot to None
        for player_index in 0..self.game_state.player_states.len() {
            self.game_state.player_states[player_index].cur_round_putting_in_pot = None;

            if !self.game_state.player_states[player_index].is_active() {
                //They have acted essentially for this round
                self.game_state.player_states[player_index].cur_round_putting_in_pot = Some(0);
            }
        }

        self.game_state.prev_round_pot += self.game_state.round_pot;
        self.game_state.round_pot = 0;
        self.game_state.current_to_call = None;
        self.game_state.min_raise = 0;

        Ok(())
    }

    fn move_to_next_round(&mut self) -> Result<(), PokerError> {
        trace!("Move to next round");

        let player_count = self.game_state.player_states.len();

        self.close_betting_round()?;

        self.game_state.current_round = self
            .game_state
            .current_round
            .next()
            .ok_or(format!("No next round {}", self.game_state.current_round))?;

        self.game_state.current_to_act =
            Position::first_to_act(player_count as _, self.game_state.current_round);

        let num_active = self.active_player_count();

        if num_active > 0 {
            trace!("Moving if needed to first active player");
            self.move_if_needed_next_to_act()?;
        } else {
            trace!("No active players, keeping position as is");
        }

        let cards_needed = match self.game_state.current_round {
            Round::Flop => 3,
            Round::Turn => 1,
            Round::River => 1,
            _ => 0,
        };

        for _ in 0..cards_needed {
            let card = self.game_runner_source.get_next_board_card()?;
            set_used_card(card.into(), &mut self.used_cards)?;
            self.game_state.board.push(card);
        }

        Ok(())
    }

    fn finish(&mut self) -> Result<(), PokerError> {
        trace!("Finish game");

        self.close_betting_round()?;

        if self.game_state.round_pot > 0 {
            return Err(
                format!("Round pot is {} but should be 0", self.game_state.round_pot).into(),
            );
        }

        let mut hand_rankings: Vec<(Rank, usize)> = Vec::new();

        let mut eval_cards = self.game_state.board.clone();

        for player_index in 0..self.game_state.player_states.len() {
            if self.game_state.player_states[player_index].folded {
                trace!(
                    "Player #{} named {} folded, did not win, skipping",
                    player_index,
                    &self.game_state.player_states[player_index].player_name
                );
                continue;
            }

            let hole_cards = self.game_runner_source.get_hole_cards(player_index)?;

            hole_cards.add_to_eval(&mut eval_cards);
            let rank = rank_cards(&eval_cards);
            hand_rankings.push((rank, player_index));

            hole_cards.remove_from_eval(&mut eval_cards)?;
        }

        let mut max_pots: Vec<ChipType> = self
            .game_state
            .player_states
            .iter()
            .map(|p| self.calc_max_pot(p.initial_stack))
            .collect();

        //best is last
        hand_rankings.sort();

        let mut all_pot_left_to_split = self.game_state.prev_round_pot;

        trace!("All pot left to split {}", all_pot_left_to_split);

        while !hand_rankings.is_empty() {
            let winning_rank = hand_rankings.last().unwrap().0;

            //Find 1st index with same rank
            let first_index = hand_rankings
                .iter()
                .position(|(rank, _)| *rank == winning_rank)
                .unwrap();

            //Now take a slice of all the players with the same rank
            //we need to sort by max_pot, lowest last
            let mut tie_hand_rankings = Vec::from_iter(hand_rankings[first_index..].iter());

            tie_hand_rankings.sort_by(|(_, player_index1), (_, player_index2)| {
                let p1_max_pot = max_pots[*player_index1];
                let p2_max_pot = max_pots[*player_index2];
                p2_max_pot.cmp(&p1_max_pot)
            });

            let mut pot_left_to_split = all_pot_left_to_split;

            //If we split evenly then we can stop early with > 0
            while !tie_hand_rankings.is_empty() && pot_left_to_split > 0 {
                //pop last player who can win smallest pot
                let (_, player_index) = tie_hand_rankings.last().unwrap();

                let player_state = &mut self.game_state.player_states[*player_index];

                let side_pot_size = min(max_pots[*player_index], pot_left_to_split);
                trace!(
                    "Player #{} named {} can win at most {} of {} pot",
                    player_index,
                    &player_state.player_name,
                    side_pot_size,
                    pot_left_to_split
                );

                let winnings = side_pot_size / tie_hand_rankings.len() as ChipType;

                trace!(
                    "Split side pot {} with {} other players, giving {} to each",
                    side_pot_size,
                    tie_hand_rankings.len(),
                    winnings,
                );

                for (_, player_index) in &tie_hand_rankings {
                    let player_state = &mut self.game_state.player_states[*player_index];
                    player_state.stack += winnings;
                    trace!(
                        "Player #{} named {} now has {}+{}={}",
                        player_index,
                        &player_state.player_name,
                        winnings,
                        player_state.stack - winnings,
                        player_state.stack
                    );
                    pot_left_to_split -= winnings;
                    all_pot_left_to_split -= winnings;
                }

                //The max pot that anyone can win has also just been reduced by the side pot we just distributed
                let n_players = self.game_state.player_states.len();
                for pi in 0..n_players {
                    if side_pot_size > max_pots[pi] {
                        max_pots[pi] = 0;
                    } else {
                        max_pots[pi] -= side_pot_size;
                    }
                }

                tie_hand_rankings.pop().unwrap();
            }

            //remove all the players with the same rank
            hand_rankings.truncate(first_index);
        }

        for player_index in 0..self.game_state.player_states.len() {
            self.game_runner_source.set_final_player_state(
                player_index,
                &self.game_state.player_states[player_index],
                None,
            )?;
        }
        Ok(())
    }

    //Returns true when game is done
    pub fn process_next_action(&mut self) -> Result<bool, PokerError> {
        let player_index: usize = self.game_state.current_to_act.into();

        let cur_active_player_count = self.active_player_count();

        trace!(
            "Process next action for player #{} named {} ({} active players) in round {}",
            player_index,
            &self.game_state.player_states[player_index].player_name,
            cur_active_player_count,
            self.game_state.current_round
        );

        let action = self.game_runner_source.get_action(
            &self.game_state.player_states[player_index],
            &self.game_state,
        )?;

        trace!(
            "Player #{} named {} does action {}",
            player_index,
            &self.game_state.player_states[player_index].player_name,
            action
        );

        match action {
            ActionEnum::Fold => {
                self.game_state.player_states[player_index].folded = true;

                //if we folded before betting anything then make sure we have a not None value
                //to indicate we acted
                self.game_state.player_states[player_index].cur_round_putting_in_pot = Some(
                    self.game_state.player_states[player_index]
                        .cur_round_putting_in_pot
                        .unwrap_or(0),
                );

                let pot_eq = 100.0 * self.game_state.current_to_call.unwrap_or(0) as f64
                    / (self.game_state.current_to_call.unwrap_or(0) as f64
                        + self.game_state.pot() as f64);

                self.game_state.actions.push(PlayerAction {
                    player_index,
                    action,
                    round: self.game_state.current_round,
                    comment: Some(format!(
                        "Player #{} {} folded to {:.1}% pot equity with {} in the pot",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        pot_eq,
                        self.game_state.pot()
                    )),
                });
            }
            ActionEnum::Call => {
                let amt_to_call = self.game_state.current_to_call.unwrap();
                let actual_amt = self.handle_put_money_in_pot(player_index, amt_to_call)?;

                //pot has already changed
                let pot_eq = 100.0 * actual_amt as f64 / (self.game_state.pot() as f64);

                let player_state = &self.game_state.player_states[player_index];

                let comment = if player_state.all_in {
                    format!(
                        "Player #{} {} calls ALL IN {} of {} with {:.1}% pot equity with {} in the pot",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        actual_amt,
                        amt_to_call,
                        pot_eq,
                        self.game_state.pot())
                } else {
                    format!(
                        "Player #{} {} calls {} (of {}) with {:.1}% pot equity with {} in the pot",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        actual_amt,
                        amt_to_call,
                        pot_eq,
                        self.game_state.pot()
                    )
                };

                self.game_state.actions.push(PlayerAction {
                    player_index,
                    action,
                    round: self.game_state.current_round,
                    comment: Some(comment),
                });
            }
            ActionEnum::Raise(raise_amt) => {
                let amt_to_call = self.game_state.current_to_call.ok_or(format!(
                    "Player {} tried to raise {} but there is no current to call",
                    player_index, raise_amt
                ))?;

                self.check_able_to_raise(raise_amt)?;

                self.game_state.min_raise = raise_amt - amt_to_call;
                self.game_state.current_to_call = Some(raise_amt);
                let actual_amt = self.handle_put_money_in_pot(player_index, raise_amt)?;

                let pot_eq = 100.0 * actual_amt as f64 / (self.game_state.pot() as f64);

                let player_state = &self.game_state.player_states[player_index];

                let comment = if player_state.all_in {
                    format!(
                        "Player #{} {} raises ALL IN {} to {} with {:.1}% pot equity with {} in the pot",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        actual_amt,
                        raise_amt,
                        pot_eq,
                        self.game_state.pot())
                } else {
                    format!(
                        "Player #{} {} raises {} to {} with {:.1}% pot equity with {} in the pot",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        actual_amt,
                        raise_amt,
                        pot_eq,
                        self.game_state.pot()
                    )
                };

                self.game_state.actions.push(PlayerAction {
                    player_index,
                    action,
                    round: self.game_state.current_round,
                    comment: Some(comment),
                });
            }
            ActionEnum::Check => {
                if let Some(amt) = self.game_state.current_to_call {
                    if amt > 0 {
                        return Err(format!(
                            "Player #{} {} tried to check but there is a current to call",
                            player_index, &self.game_state.player_states[player_index].player_name,
                        )
                        .into());
                    }
                }

                self.game_state.current_to_call = Some(0);

                self.game_state.actions.push(PlayerAction {
                    player_index,
                    action,
                    round: self.game_state.current_round,
                    comment: None,
                });
            }
            ActionEnum::Bet(bet_amt) => {
                self.check_able_to_bet(bet_amt)?;

                self.game_state.min_raise = bet_amt;
                self.game_state.current_to_call = Some(bet_amt);

                let actual_amt = self.handle_put_money_in_pot(player_index, bet_amt)?;

                let pot_eq = 100.0 * actual_amt as f64 / (self.game_state.pot() as f64);

                let player_state = &self.game_state.player_states[player_index];

                let comment = if player_state.all_in {
                    format!(
                        "Player #{} {} bets ALL IN {} to {} with {:.1}% pot equity with {} in the pot",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        actual_amt,
                        bet_amt,
                        pot_eq,
                        self.game_state.pot())
                } else {
                    format!(
                        "Player #{} {} bets {} to {} with {:.1}% pot equity with {} in the pot",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        actual_amt,
                        bet_amt,
                        pot_eq,
                        self.game_state.pot()
                    )
                };

                self.game_state.actions.push(PlayerAction {
                    player_index,
                    action,
                    round: self.game_state.current_round,
                    comment: Some(comment),
                });
            }
        }

        let cur_active_player_count = self.active_player_count();

        let any_all_in = self.game_state.player_states.iter().any(|p| p.all_in);

        if cur_active_player_count == 0 && any_all_in {
            trace!("Only all in player left, and we have at least 1 all in, advancing to river");
            let cur_round = self.game_state.current_round as u8;
            for _ in cur_round..3 {
                trace!("Only all in player left, advancing round...");
                self.move_to_next_round()?;
            }
            trace!(
                "Only all in player left, finishing with round {}",
                self.game_state.current_round
            );
            self.finish()?;
            return Ok(true);
        }
        // if cur_active_player_count == 1 {
        //     trace!("Only 1 player left, game is done");
        //     self.finish()?;
        //     return Ok(true);
        // }

        self.game_state.current_to_act = self
            .game_state
            .current_to_act
            .next(self.game_state.player_states.len() as _);
        self.move_if_needed_next_to_act()?;

        let player_index: usize = self.game_state.current_to_act.into();

        //Do we need to move to the next round?
        //Either checks all around or everyone called
        if let Some(amt_to_call) = self.game_state.current_to_call {
            if let Some(cur_round_putting_in_pot) =
                self.game_state.player_states[player_index].cur_round_putting_in_pot
            {
                //If current player has called the amount needed we move to the next round
                if amt_to_call == cur_round_putting_in_pot {
                    trace!(
                        "Player #{} named {} has called {} and we are moving to next round",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        amt_to_call
                    );
                    if self.game_state.current_round == Round::River {
                        trace!("River is done, game is done");
                        self.finish()?;
                        return Ok(true);
                    }
                    let cur_round = self.game_state.current_round;
                    self.move_to_next_round()?;
                    assert_eq!(cur_round.next().unwrap(), self.game_state.current_round);
                }
            }
        }

        Ok(false)
    }

    fn check_able_to_bet(self: &Self, bet_amt: ChipType) -> Result<(), PokerError> {
        let player_index: usize = self.game_state.current_to_act.into();
        let player_state = &self.game_state.player_states[player_index];

        if self.game_state.current_to_call.unwrap_or(0) != 0 {
            return Err(format!(
                "Player #{} {} tried to bet {} but there is already a bet to call, must call or raise or fold",
                player_index,
                &player_state.player_name,
                bet_amt,
            ).into());
        }

        //A bet all in doesn't need to be at least anything
        if bet_amt < player_state.stack {
            if bet_amt < self.game_state.bb {
                return Err(format!(
                    "Player #{} {} tried to bet {} but must be at least big blind {}",
                    player_index, &player_state.player_name, bet_amt, self.game_state.bb
                )
                .into());
            }
            if bet_amt % self.game_state.bb != 0 {
                return Err(format!(
                    "Player #{} {} tried to bet {} but must be a multiple of big blind {}",
                    player_index, &player_state.player_name, bet_amt, self.game_state.bb
                )
                .into());
            }
        }

        if bet_amt > player_state.stack {
            return Err(format!(
                "Player #{} {} tried to bet {} but only has {}",
                player_index, &player_state.player_name, bet_amt, player_state.stack
            )
            .into());
        }

        Ok(())
    }

    fn check_able_to_raise(self: &Self, raise_amt: ChipType) -> Result<(), PokerError> {
        let player_index: usize = self.game_state.current_to_act.into();
        let player_state = &self.game_state.player_states[player_index];

        let amt_to_call = self.game_state.current_to_call.unwrap_or(0);

        //Can only raise if there is a current to call
        if amt_to_call == 0 {
            return Err(format!(
                "Player #{} {} tried to raise {} but there is no bet to call, must bet or fold",
                player_index, &player_state.player_name, raise_amt,
            )
            .into());
        }

        if raise_amt <= amt_to_call {
            return Err(format!(
                "Player #{} {} tried to raise {} but must be more than the call amount {} ",
                player_index, &player_state.player_name, raise_amt, amt_to_call
            )
            .into());
        }

        //A raise all in doesn't need to be at least anything
        if raise_amt < player_state.stack {
            if raise_amt < self.game_state.min_raise + self.game_state.current_to_call.unwrap() {
                return Err(format!(
                    "Player #{} {} tried to raise {} but needs to be at least {} more than {}",
                    player_index,
                    &player_state.player_name,
                    raise_amt,
                    self.game_state.min_raise,
                    self.game_state.current_to_call.unwrap()
                )
                .into());
            }

            //Also check multiple of bb
            if (raise_amt - self.game_state.current_to_call.unwrap()) % self.game_state.bb != 0 {
                return Err(format!(
                    "Player #{} {} tried to raise {} but must be a multiple of big blind {}",
                    player_index, &player_state.player_name, raise_amt, self.game_state.bb
                )
                .into());
            }
        }

        if raise_amt > player_state.stack {
            return Err(format!(
                "Player #{} {} tried to raise {} but only has {}",
                player_index, &player_state.player_name, raise_amt, player_state.stack
            )
            .into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use log::debug;

    use crate::{game::game_log_source::GameLogSource, init_test_logger, CardVec};

    use super::*;

    #[test]
    fn test_game_runner() {
        init_test_logger();

        let hh = "
*** Players ***
Plyr A - 12 - As 2c
Plyr B - 147 - 3d 3c
Plyr C - 55 - 7d 3h
Plyr D - 55 - Ks Kd
*** Blinds ***
Plyr A - 5
Plyr B - 10
*** Preflop ***
Plyr C calls    # UTG acts first
Plyr D raises 20
Plyr A calls
Plyr B raises 30
Plyr C folds
Plyr D calls
*** Flop ***
2s 7c 8s
Plyr B bets 10
Plyr D calls
*** Turn ***
2h
Plyr B bets 10
Plyr D folds
*** River ***
2d
Plyr A bets 15 # This never gets used
Plyr B raises 30 # minimum raise
Plyr A raises 45
Plyr B calls
*** Summary ***
Plyr A - 46 # 10 from plyr C, 12 from everyone else
Plyr B - 163 # Plyr B loses 100 with 2h As Kh 2d 7c
Plyr C - 45 # Folded 10
Plyr D - 15 # Lost 30, 10
    ";
        let game_log: GameLog = hh.parse().unwrap();

        let game_log_source = GameLogSource::new(game_log);

        let mut game_runner = GameRunner::new(GameRunnerSourceEnum::from(game_log_source)).unwrap();

        game_runner.process_next_action().unwrap();
        game_runner.process_next_action().unwrap();

        assert_eq!(game_runner.game_state.current_round, Round::Preflop);
        assert_eq!(
            game_runner.game_state.current_to_act,
            Position::try_from(0).unwrap()
        );
        assert_eq!(
            game_runner.game_state.player_states[3].cur_round_putting_in_pot,
            Some(20)
        );

        //preflop actions
        for _ in 0..4 {
            game_runner.process_next_action().unwrap();
        }

        assert_eq!(game_runner.game_state.current_round, Round::Flop);
        assert!(game_runner.game_state.player_states[0].all_in);
        assert_eq!(game_runner.game_state.prev_round_pot, 12 + 30 * 2 + 10);
        assert_eq!(game_runner.game_state.round_pot, 0);
        assert_eq!(
            game_runner.game_state.board,
            CardVec::try_from("2s 7c 8s").unwrap().0
        );

        //flop actions
        for _ in 0..2 {
            game_runner.process_next_action().unwrap();
        }

        assert_eq!(game_runner.game_state.current_round, Round::Turn);
        assert!(game_runner.game_state.player_states[0].all_in);
        assert_eq!(game_runner.game_state.prev_round_pot, 12 + 30 * 2 + 10 + 20);
        assert_eq!(game_runner.game_state.round_pot, 0);
        assert_eq!(
            game_runner.game_state.board,
            CardVec::try_from("2s 7c 8s 2h").unwrap().0
        );

        //turn actions
        for _ in 0..2 {
            game_runner.process_next_action().unwrap();
        }

        assert_eq!(game_runner.game_state.current_round, Round::River);
        assert!(game_runner.game_state.player_states[0].all_in);
        assert_eq!(game_runner.game_state.prev_round_pot, 12 + 30 * 2 + 10 + 30);
        assert_eq!(game_runner.game_state.round_pot, 0);
        assert_eq!(
            game_runner.game_state.board,
            CardVec::try_from("2s 7c 8s 2h 2d").unwrap().0
        );

        //river actions

        assert_eq!(true, game_runner.process_next_action().unwrap());
    }

    #[test]
    fn test_multi_splits() {
        init_test_logger();

        //We are going to have the best 4 hands split, then next best 3 hands split,
        //then next best 2 hands split, then last guy getting the diff

        //So 4 players with A5 A4 A3 A2
        //Board K Q J 9 8
        // 3 players with K5 k4 k3
        // 2 players with Q5 Q4
        // 1 player with J5 (with most chips)

        let hh = "
*** Players ***
Player C1 - 340 - Qd 2d
Player B3 - 231 - Kd 4d
Player A1 - 100 - Ad 3h
Player B2 - 220 - Kc 3s
Player A2 - 110 - As 2c
Player B1 - 200 - Kh 2h
Player A4 - 103 - Ac 4c
Player A3 - 130 - Ah 3c
Player C2 - 320 - Qs 4s
Player D1 - 400 - Jd 3d
*** Blinds ***
Player C1 - 1
Player B3 - 5
*** Preflop ***
Player A1 calls    # UTG acts first
Player B2 calls
Player A2 calls
Player B1 calls
Player A4 raises 103
Player A3 calls
Player C2 calls
Player D1 calls
Player C1 calls
Player B3 calls
Player A1 calls # all in
Player B2 calls
Player A2 calls
Player B1 calls
*** Flop ***
Ts 9h 8c
Player C1 checks
Player B3 checks
Player B2 checks
Player A2 checks
Player B1 checks
Player A3 bets 27 # all in for 130
Player C2 calls
Player D1 calls
Player C1 calls
Player B3 calls
Player B2 calls
Player A2 calls # all in
Player B1 calls
*** Turn ***
6h
Player C1 bets 100
Player B3 raises 101 # all in
Player B2 calls
Player B1 calls
Player C2 calls
Player D1 raises 270 # all in
Player C1 calls
Player C2 calls
*** River ***
5d
*** Summary ***
Player C1 - 173 # 133 + 20*2
Player B3 - 234 # 190 + 11*4
Player A1 - 250 # 100 * 10 / 4 == 250
Player B2 - 190 # 140 + 20*5/2
Player A2 - 287 # 250 (side pot with A1) + 9 (side pot with A4) + (7*8) / 2
Player B1 - 140 # 70 * 6 / 3
Player A4 - 259 # 250 + (3*9) / 3 
Player A3 - 427 # 250 + 9 + 28 + 20*7
Player C2 - 133 # (320-231)*3 / 2 == 133.5 rounded down
Player D1 - 60 # Keeps what's left of his stack
    ";
        let game_log: GameLog = hh.parse().unwrap();

        let game_log_source = GameLogSource::new(game_log);

        let mut game_runner = GameRunner::new(GameRunnerSourceEnum::from(game_log_source)).unwrap();

        for _ in 0..200 {
            let action_count_before = game_runner.game_state.actions.len();
            let r = game_runner.process_next_action().unwrap();
            if r {
                break;
            }
            let action_count_after = game_runner.game_state.actions.len();
            debug!(
                "Last action: {}",
                &game_runner.game_state.actions.last().as_ref().unwrap()
            );
            assert_eq!(action_count_before + 1, action_count_after);
        }
    }
}
