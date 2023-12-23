//maintain player states and game state
// and we have 2 playbacks, either agent or a log
// via trait that will do

use std::cmp::{max, min};

use crate::{Rank, set_used_card, rank_cards};
use crate::{
    ActionEnum, Card, CardUsedType, ChipType, GameLog, GameState, HoleCards, InitialPlayerState,
    PlayerState, PokerError, Position, Round,
};

use log::trace;
use crate::game::game_runner_source::GameRunnerSource;
use crate::game::game_runner_source::GameRunnerSourceEnum;

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
    ) -> Result<(), PokerError> {
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

        Ok(())
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
            max_pot += min(
                money_put_in,
                all_in_for,
            )
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

            trace!(
                "Player {} has put in {}",
                player_state.player_name,
                cur_round_putting_in_pot
            );

            check_round_pot += cur_round_putting_in_pot;
            if player_state.folded {
                continue;
            }
            if player_state.all_in {
                self.game_state.current_to_call.ok_or(format!(
                    "
                Player {} is all in for {} but there is no current to call",
                    &player_state.player_name,
                    player_state.initial_stack 
                ))?;

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

    fn move_to_next_round(&mut self) -> Result<(), PokerError> {
        trace!("Move to next round");

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

        self.game_state.current_round = self
            .game_state
            .current_round
            .next()
            .ok_or(format!("No next round {}", self.game_state.current_round))?;

        self.game_state.current_to_act =
            Position::first_to_act(player_count as _, self.game_state.current_round);

        self.move_if_needed_next_to_act()?;

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

        if self.game_state.round_pot > 0 {
            return Err(format!(
                "Round pot is {} but should be 0",
                self.game_state.round_pot
            ).into());
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

        let max_pots: Vec<ChipType> = self.game_state.player_states.iter().map(|p| 
            self.calc_max_pot(p.initial_stack)).collect();

        //best is last
        hand_rankings.sort();

        let mut all_pot_left_to_split = self.game_state.prev_round_pot;

        while !hand_rankings.is_empty() {

            let winning_rank = hand_rankings.last().unwrap().0;

            //Find 1st index with same rank
            let first_index = hand_rankings
                .iter()
                .position(|(rank, _)| *rank == winning_rank)
                .unwrap();

            //Now take a slice of all the players with the same rank
            //we need to sort by max_pot, lowest first
            let mut tie_hand_rankings = Vec::from_iter(hand_rankings[first_index..].iter());

            tie_hand_rankings.sort_by(|(_, player_index1), (_, player_index2)| {
                let p1_max_pot = max_pots[*player_index1];
                let p2_max_pot = max_pots[*player_index2];
                p1_max_pot.cmp(&p2_max_pot)
            });
            
            let mut pot_left_to_split = all_pot_left_to_split;
            let mut cur_tie_count = tie_hand_rankings.len() as ChipType;

            for (_, player_index) in tie_hand_rankings.iter() {
                let player_state = &mut self.game_state.player_states[*player_index];

                let max_winnings = min(max_pots[*player_index], pot_left_to_split);

                let winnings = max_winnings / cur_tie_count;

                trace!(
                    "Player #{} named {} won {}, split with {} other players",
                    player_index,
                    &player_state.player_name,
                    winnings,
                    tie_hand_rankings.len()
                );

                player_state.stack += winnings;
                cur_tie_count -= 1;
                pot_left_to_split -= winnings;

                all_pot_left_to_split -= winnings;
            }

            //remove all the players with the same rank
            hand_rankings.truncate(first_index);
        }

        for player_index in 0..self.game_state.player_states.len() {
            self.game_runner_source.set_final_player_state(player_index,
               & self.game_state.player_states[player_index], None)?;
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

        if cur_active_player_count == 1 {
            self.finish()?;
            return Ok(true);
        }

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
                    self.game_state.player_states[player_index].cur_round_putting_in_pot.unwrap_or(
                        0,
                    )
                )
            }
            ActionEnum::Call => {
                let amt_to_call = self.game_state.current_to_call.unwrap();
                self.handle_put_money_in_pot(player_index, amt_to_call)?;
            }
            ActionEnum::Raise(raise_amt) => {
                //Can only raise if there is a current to call
                let amt_to_call = self.game_state.current_to_call.ok_or(format!(
                    "Player {} tried to raise {} but there is no current to call",
                    player_index, raise_amt
                ))?;

                //Would this raise essentially put this player all in ?
                if self.game_state.player_states[player_index].stack > raise_amt {
                    if raise_amt < self.game_state.min_raise + amt_to_call {
                        return Err(format!(
                        "Player #{} {} tried to raise {} but needs to be at least {} more than {}",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        raise_amt,
                        self.game_state.min_raise,
                        amt_to_call
                    )
                        .into());
                    }

                    //Also check multiple of bb
                    if (raise_amt - amt_to_call) % self.game_state.bb != 0 {
                        return Err(format!(
                            "Player #{} {} tried to raise {} but must be a multiple of big blind {}",
                            player_index,
                            &self.game_state.player_states[player_index].player_name,
                            raise_amt,
                            self.game_state.bb
                        )
                        .into());
                    }
                } else {
                    //if we are going all in with this raise, just check the raise is > 0
                    if raise_amt < amt_to_call {
                        return Err(format!(
                            "Player #{} {} tried to all-in raise {} but needs to be at least {} more than {}",
                            player_index,
                            &self.game_state.player_states[player_index].player_name,
                            raise_amt,
                            self.game_state.min_raise,
                            amt_to_call
                        )
                        .into());
                    }
                }

                self.game_state.min_raise = raise_amt - amt_to_call;
                self.game_state.current_to_call = Some(raise_amt);
                self.handle_put_money_in_pot(player_index, raise_amt)?;
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
            }
            ActionEnum::Bet(bet_amt) => {
                if self.game_state.current_to_call.is_some() {
                    return Err(format!(
                        "Player #{} {} tried to bet but there is a current to call, must call or raise or fold",
                        player_index, 
                        &self.game_state.player_states[player_index].player_name,
                    ).into());
                }

                //A bet all in doesn't need to be at least anything
                if bet_amt < self.game_state.player_states[player_index].stack {
                    if bet_amt < self.game_state.bb {
                        return Err(format!(
                            "Player #{} {} tried to bet {} but must be at least big blind {}",
                            player_index,
                            &self.game_state.player_states[player_index].player_name,
                            bet_amt,
                            self.game_state.bb
                        )
                        .into());
                    }
                    if bet_amt % self.game_state.bb != 0 {
                        return Err(format!(
                            "Player #{} {} tried to bet {} but must be a multiple of big blind {}",
                            player_index,
                            &self.game_state.player_states[player_index].player_name,
                            bet_amt,
                            self.game_state.bb
                        )
                        .into());
                    }
                }

                self.game_state.min_raise = bet_amt;
                self.game_state.current_to_call = Some(bet_amt);

                self.handle_put_money_in_pot(player_index, bet_amt)?;
            }
        }

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
}

#[cfg(test)]
mod tests {
    use crate::{init_test_logger, CardVec, game::game_log_source::GameLogSource};

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
        assert_eq!(game_runner.game_state.board, CardVec::try_from("2s 7c 8s").unwrap().0);

        //flop actions
        for _ in 0..2 {
            game_runner.process_next_action().unwrap();
        }

        assert_eq!(game_runner.game_state.current_round, Round::Turn);
        assert!(game_runner.game_state.player_states[0].all_in);
        assert_eq!(game_runner.game_state.prev_round_pot, 12 + 30 * 2 + 10 + 20);
        assert_eq!(game_runner.game_state.round_pot, 0);
        assert_eq!(game_runner.game_state.board, CardVec::try_from("2s 7c 8s 2h").unwrap().0);

        //turn actions
        for _ in 0..2 {
            game_runner.process_next_action().unwrap();
        }

        assert_eq!(game_runner.game_state.current_round, Round::River);
        assert!(game_runner.game_state.player_states[0].all_in);
        assert_eq!(game_runner.game_state.prev_round_pot, 12 + 30 * 2 + 10 + 30);
        assert_eq!(game_runner.game_state.round_pot, 0);
        assert_eq!(game_runner.game_state.board, CardVec::try_from("2s 7c 8s 2h 2d").unwrap().0);

        //river actions

        assert_eq!(true, game_runner.process_next_action().unwrap());
    }
}
