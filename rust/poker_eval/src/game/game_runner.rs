//maintain player states and game state
// and we have 2 playbacks, either agent or a log
// via trait that will do

use std::cmp::min;

use crate::pre_calc::fast_eval::fast_hand_eval;
use crate::pre_calc::perfect_hash::load_boomperfect_hash;
use crate::pre_calc::rank::Rank;
use crate::{set_used_card, Board, Card, GameLog, InitialPlayerState, PlayerAction};
use crate::{
    ActionEnum, CardUsedType, ChipType, FinalPlayerState, GameState, PlayerState, PokerError,
    Position, Round,
};

use crate::game::game_runner_source::GameRunnerSource;
use crate::game::game_runner_source::GameRunnerSourceEnum;
use boomphf::Mphf;

use log::trace;

// Enforces the poker rules
pub struct GameRunner {
    used_cards: CardUsedType,

    pub game_state: GameState,

    // Source of actions, cards
    pub game_runner_source: GameRunnerSourceEnum,

    hash_func: Mphf<u32>,
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
            current_to_call: bb,
            current_round: Round::Preflop,
            board: Board::new(),
            sb,
            bb,
            actions: Vec::new(),
            min_raise: bb,
            num_left_to_act: initial_players.len() as _,
            total_active_players: initial_players.len() as _,
            total_players_all_in: 0,
        };

        let mut r = GameRunner {
            game_state,
            game_runner_source,
            used_cards: CardUsedType::default(),
            hash_func: load_boomperfect_hash(),
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

            //All in is only set after we put money in the pot
            let last_action_index = self.game_state.actions.len() - 1;
            self.game_state.actions[last_action_index].is_all_in = player_state.all_in;

            //max_pot is created when the round is done
            player_state.stack
        } else {
            max_actual_amount
        };

        assert!(player_state.stack >= actual_amount);

        player_state.stack -= actual_amount;
        player_state.total_put_in_pot += actual_amount;
        player_state.cur_round_putting_in_pot =
            Some(player_state.cur_round_putting_in_pot.unwrap_or(0) + actual_amount);

        if max_actual_amount == actual_amount {
            assert_eq!(player_state.cur_round_putting_in_pot, Some(amount));
        }

        self.game_state.round_pot += actual_amount;

        Ok(actual_amount)
    }

    // fn active_player_count(&self) -> usize {
    //     self.game_state
    //         .player_states
    //         .iter()
    //         .filter(|p| p.is_active())
    //         .count()
    // }

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

        let current_to_call = self.game_state.current_to_call;

        //Do some sanity checks, each player either folded or put in the same amount or is all in
        for player_state in &self.game_state.player_states {
            //we can have an active player that has not acted if there's only 1
            //So this check is just to see everyone has put in, not necesarily if they have acted
            let cur_round_putting_in_pot = player_state.cur_round_putting_in_pot.unwrap_or(0);

            check_round_pot += cur_round_putting_in_pot;
            if !player_state.is_active() {
                continue;
            }
            if cur_round_putting_in_pot != current_to_call {
                return Err(format!(
                    "Player {} has put in {} but current to call is {}",
                    player_state.player_name, cur_round_putting_in_pot, current_to_call
                )
                .into());
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

    fn find_next_to_act(&self) -> Result<Position, PokerError> {
        let n_players = self.game_state.player_states.len();
        let mut pos = self.game_state.current_to_act;
        for _ in 0..n_players {
            let player_index: usize = pos.into();
            if self.game_state.player_states[player_index].is_active() {
                return Ok(pos);
            }

            trace!(
                "Player #{} named {} is all in or folded, moving to next player",
                player_index,
                &self.game_state.player_states[player_index].player_name
            );

            pos = pos.next(self.game_state.player_states.len() as _);
        }

        Err(format!("No players left to act").into())
    }

    fn close_betting_round(&mut self) -> Result<(), PokerError> {
        trace!("Close betting round");

        self.check_pots_good()?;
        //calculate max_pot
        let _player_count = self.game_state.player_states.len();
        // for player_index in 0..self.game_state.player_states.len() {
        //     if let Some(all_in) = self.game_state.player_states[player_index].all_in_for {
        //         let max_pot = self.calc_max_pot(all_in);
        //         self.game_state.player_states[player_index].max_pot = Some(max_pot);
        //     }
        // }

        //set cur round putting in pot to None
        for player_index in 0..self.game_state.player_states.len() {
            self.game_state.player_states[player_index].cur_round_putting_in_pot = None;
        }

        self.game_state.prev_round_pot += self.game_state.round_pot;
        self.game_state.round_pot = 0;
        self.game_state.current_to_call = 0;
        self.game_state.min_raise = self.game_state.bb;

        Ok(())
    }

    fn move_to_next_round(&mut self) -> Result<(), PokerError> {
        trace!(
            "Done with {}, Move to next round",
            self.game_state.current_round
        );

        let player_count = self.game_state.player_states.len();

        self.close_betting_round()?;

        self.game_state.current_round = self
            .game_state
            .current_round
            .next()
            .ok_or(format!("No next round {}", self.game_state.current_round))?;

        self.game_state.current_to_act =
            Position::first_to_act(player_count as _, self.game_state.current_round);

        // let num_active = self.active_player_count();
        // assert_eq!(num_active as u8, self.game_state.total_active_players);

        self.game_state.num_left_to_act = self.game_state.total_active_players;

        if self.game_state.total_active_players > 0 {
            trace!("Moving if needed to first active player");
            self.game_state.current_to_act = self.find_next_to_act()?;
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
            self.game_state.board.add_card(card).unwrap();
        }

        //to have it calculated
        let _index_ = self.game_state.board.get_index();

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

        //Add a short cut if only 1 non folded left
        if self.game_state.total_active_players == 1 && self.game_state.total_players_all_in == 0 {
            let player_index = self
                .game_state
                .player_states
                .iter()
                .position(|p| p.is_active())
                .unwrap();

            self.game_state.player_states[player_index].stack += self.game_state.pot();
            self.game_state.player_states[player_index].final_state =
                Some(FinalPlayerState::EveryoneElseFolded);

            for player_index in 0..self.game_state.player_states.len() {
                self.game_runner_source.set_final_player_state(
                    player_index,
                    &self.game_state.player_states[player_index],
                    None,
                )?;
            }
            return Ok(());
        }

        assert!(
            self.game_state.total_active_players > 1 || self.game_state.total_players_all_in > 0
        );

        let mut hand_rankings: Vec<(Rank, usize)> = Vec::new();
        //let mut hand_ranking_strings: Vec<Option<String>> = vec![None; self.game_state.player_states.len()];

        //let mut eval_cards = self.game_state.board.as_slice_card().to_vec();

        for player_index in 0..self.game_state.player_states.len() {
            let p_data = &self.game_state.player_states[player_index];

            assert!(p_data.initial_stack >= p_data.stack);
            assert_eq!(p_data.total_put_in_pot, p_data.initial_stack - p_data.stack);

            if self.game_state.player_states[player_index].is_folded() {
                // trace!(
                //     "Player #{} named {} folded, did not win, skipping",
                //     player_index,
                //     &self.game_state.player_states[player_index].player_name
                // );
                continue;
            }

            let hole_cards = self.game_runner_source.get_hole_cards(player_index)?;

            let rank = fast_hand_eval(
                self.game_state
                    .board
                    .get_iter()
                    .chain(hole_cards.get_iter()),
                &self.hash_func,
            );

            hand_rankings.push((rank, player_index));
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
                    player_state.final_state = Some(FinalPlayerState::WonShowdown);
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
            {
                let player_state = &mut self.game_state.player_states[player_index];
                if player_state.final_state.is_none() {
                    player_state.final_state = Some(FinalPlayerState::LostShowdown);
                }
            }
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

        // let cur_active_player_count = self.active_player_count();

        // assert_eq!(cur_active_player_count as u8, self.game_state.total_active_players);

        trace!(
            "Process next action for player #{} named {} ({} active players) in round {}.  Num left to act: {}",
            player_index,
            &self.game_state.player_states[player_index].player_name,
            self.game_state.total_active_players,
            self.game_state.current_round,
            self.game_state.num_left_to_act
        );

        //Update before deciding, num_left_to_act == 0 on the player that can close the action
        assert!(self.game_state.num_left_to_act > 0);
        self.game_state.num_left_to_act -= 1;

        let decision = self.game_runner_source.get_action(
            &self.game_state.player_states[player_index],
            &self.game_state,
        )?;
        let action = decision.action;

        trace!(
            "Player #{} named {} does action {}",
            player_index,
            &self.game_state.player_states[player_index].player_name,
            action
        );

        assert!(!self.game_state.player_states[player_index].all_in);
        assert!(self.game_state.player_states[player_index].is_active());

        match action {
            ActionEnum::Fold => {
                // Do before anything is modified
                self.game_state.actions.push(self.build_player_action(
                    &self.game_state.player_states[player_index],
                    &action,
                    &decision.comment.unwrap_or_default(),
                ));

                self.game_state.player_states[player_index].final_state =
                    Some(FinalPlayerState::Folded(self.game_state.current_round));

                assert!(self.game_state.total_active_players > 0);
                self.game_state.total_active_players -= 1;

                //if we folded before betting anything then make sure we have a not None value
                //to indicate we acted
                self.game_state.player_states[player_index].cur_round_putting_in_pot = Some(
                    self.game_state.player_states[player_index]
                        .cur_round_putting_in_pot
                        .unwrap_or(0),
                );

                
            }
            ActionEnum::Call(check_amt) => {
                let amt_to_call = self.game_state.current_to_call;

                if amt_to_call == 0 {
                    return Err(format!(
                        "Player {} named {} tried to call but there is no current to call",
                        player_index, &self.game_state.player_states[player_index].player_name
                    )
                    .into());
                }

                //do before stack/pot are modified
                self.game_state.actions.push(self.build_player_action(
                    &self.game_state.player_states[player_index],
                    &action,
                    &decision.comment.unwrap_or_default(),
                ));

                let actual_amt = self.handle_put_money_in_pot(player_index, amt_to_call)?;

                if actual_amt != check_amt {
                    return Err(format!(
                        "Player {} named {} tried to call {} but only actually put in {}",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        check_amt,
                        actual_amt
                    )
                    .into());
                }

                if self.game_state.player_states[player_index].all_in {
                    self.game_state.total_players_all_in += 1;
                    assert!(self.game_state.total_active_players > 0);
                    self.game_state.total_active_players -= 1;
                };
            }
            ActionEnum::Raise(increase_amt_check, raise_amt) => {
                let amt_to_call = self.game_state.current_to_call;

                self.check_able_to_raise(raise_amt)?;

                //do before anything else is modified
                self.game_state.actions.push(self.build_player_action(
                    &self.game_state.player_states[player_index],
                    &action,
                    &decision.comment.unwrap_or_default(),
                ));

                //this is also the amount increased from the bet
                let increase_amt = raise_amt - amt_to_call;
                //the next raise also has to increase by at least this amount
                self.game_state.min_raise = increase_amt;
                self.game_state.current_to_call = raise_amt;

                

                let amount_already_put = self.game_state.player_states[player_index]
                    .cur_round_putting_in_pot
                    .unwrap_or(0);
                let actual_amt = self.handle_put_money_in_pot(player_index, raise_amt)?;

                if increase_amt_check != increase_amt {
                    return Err(format!(
                        "Player {} named {} tried to raise {} to {} but should be {} to {}",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        increase_amt_check,
                        raise_amt,
                        increase_amt,
                        raise_amt
                    )
                    .into());
                }

                if amount_already_put + actual_amt != raise_amt {
                    return Err(format!(
                        "Player {} named {} had put in {}, added {} to raise to {} but should be {}",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        amount_already_put,
                        actual_amt,                        
                        amount_already_put + actual_amt,
                        raise_amt,
                    )
                    .into());
                }

                //we go around again
                assert!(self.game_state.total_active_players > 0);
                self.game_state.num_left_to_act = self.game_state.total_active_players - 1;

                // discount all_in after we set num left to act
                if self.game_state.player_states[player_index].all_in {
                    self.game_state.total_players_all_in += 1;
                    self.game_state.total_active_players -= 1;
                };
            }
            ActionEnum::Check => {
                if self.game_state.current_to_call > 0 {
                    return Err(format!(
                        "Player #{} {} tried to check but there is a current to call of {}",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        self.game_state.current_to_call
                    )
                    .into());
                }

                // Do before anything is modified
                self.game_state.actions.push(self.build_player_action(
                    &self.game_state.player_states[player_index],
                    &action,
                    &decision.comment.unwrap_or_default(),
                ));

                assert_eq!(0, self.game_state.current_to_call);
                self.game_state.player_states[player_index].cur_round_putting_in_pot = Some(0);

                
            }
            ActionEnum::Bet(bet_amt) => {
                self.check_able_to_bet(bet_amt)?;

                // Do before anything is modified
                self.game_state.actions.push(self.build_player_action(
                    &self.game_state.player_states[player_index],
                    &action,
                    &decision.comment.unwrap_or_default(),
                ));

                self.game_state.min_raise = bet_amt;
                self.game_state.current_to_call = bet_amt;

                let actual_amt = self.handle_put_money_in_pot(player_index, bet_amt)?;

                if actual_amt != bet_amt {
                    return Err(format!(
                        "Player {} named {} tried to bet {} but only actually put in {}",
                        player_index,
                        &self.game_state.player_states[player_index].player_name,
                        bet_amt,
                        actual_amt
                    )
                    .into());
                }

                //we go around again, but not including us
                assert!(self.game_state.total_active_players > 0);
                self.game_state.num_left_to_act = self.game_state.total_active_players - 1;

                //Discount after setting num left to act
                if self.game_state.player_states[player_index].all_in {
                    self.game_state.total_players_all_in += 1;
                    self.game_state.total_active_players -= 1;
                };
            }
        }

        trace!(
            "Last action: {} Num Left To Act: {}",
            &self.game_state.actions.last().as_ref().unwrap(),
            self.game_state.num_left_to_act
        );

        let not_folded_count =
            self.game_state.total_active_players + self.game_state.total_players_all_in;
        if not_folded_count == 1 {
            //this is when everyone folds to a player and we don't go to the end
            trace!("Only 1 player left; everyone else folded, game is done");
            self.finish()?;
            return Ok(true);
        }

        let any_all_in = self.game_state.total_players_all_in > 0;

        //either only all in left or only 1 active left that is ok with the pot
        if self.game_state.total_active_players <= 0 || (
            self.game_state.num_left_to_act == 0 && 
            self.game_state.total_active_players == 1)
            {
            trace!("Only all in player left, and we have at least 1 all in, advancing to river");

            assert!(any_all_in);
            assert!(self.game_state.total_players_all_in > 0);

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
        trace!(
            "{} active players left, any all in? {}",
            self.game_state.total_active_players,
            any_all_in
        );

        //Do we need to move to the next round?
        if self.game_state.num_left_to_act == 0 {
            //note moving to next round has checks to make sure everyone has called
            //or is all in

            if self.game_state.current_round == Round::River {
                trace!("River is done, game is done");
                self.finish()?;
                return Ok(true);
            }
            let cur_round = self.game_state.current_round;
            self.move_to_next_round()?;
            assert_eq!(cur_round.next().unwrap(), self.game_state.current_round);

            return Ok(false);
        }

        //Same round, next player
        self.game_state.current_to_act = self
            .game_state
            .current_to_act
            .next(self.game_state.player_states.len() as _);
        self.game_state.current_to_act = self.find_next_to_act()?;

        Ok(false)
    }

    //For logging purposes, take current game state and action and create player action that
    //goes into the log
    fn build_player_action(
        &self,
        player_state: &PlayerState,
        action: &ActionEnum,
        comment: &str,
    ) -> PlayerAction {
        PlayerAction {
            player_index: player_state.player_index(),
            action: action.clone(),
            round: self.game_state.current_round,
            player_comment: Some(comment.to_string()),
            pot: self.game_state.pot(),
            current_amt_to_call: self.game_state.current_to_call,
            amount_put_in_pot_this_round: player_state.cur_round_putting_in_pot.unwrap_or(0),
            total_amount_put_in_pot: player_state.total_put_in_pot,
            players_left_to_act: self.game_state.num_left_to_act,
            is_all_in: player_state.all_in,
            non_folded_players: self.game_state.total_active_players
                + self.game_state.total_players_all_in,
        }
    }

    fn check_able_to_bet(self: &Self, bet_amt: ChipType) -> Result<(), PokerError> {
        let player_index: usize = self.game_state.current_to_act.into();
        let player_state = &self.game_state.player_states[player_index];

        if self.game_state.current_to_call != 0 {
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
            // if bet_amt % self.game_state.bb != 0 {
            //     return Err(format!(
            //         "Player #{} {} tried to bet {} but must be a multiple of big blind {}",
            //         player_index, &player_state.player_name, bet_amt, self.game_state.bb
            //     )
            //     .into());
            // }
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

        let amt_to_call = self.game_state.current_to_call;

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

        if raise_amt < player_state.cur_round_putting_in_pot.unwrap_or(0) {
            return Err(format!(
                "Player #{} {} tried to raise {} but needs to be at least {} more than {}, what they put in last time",
                player_index,
                &player_state.player_name,
                raise_amt,
                player_state.cur_round_putting_in_pot.unwrap_or(0),
                amt_to_call
            )
            .into());
        }

        let actual_increase = raise_amt - player_state.cur_round_putting_in_pot.unwrap_or(0);

        //A raise all in doesn't need to be at least anything
        if actual_increase < player_state.stack {
            if raise_amt < self.game_state.min_raise + self.game_state.current_to_call {
                return Err(format!(
                    "Player #{} {} tried to raise {} but needs to be at least {} more than {}",
                    player_index,
                    &player_state.player_name,
                    raise_amt,
                    self.game_state.min_raise,
                    self.game_state.current_to_call
                )
                .into());
            }

            //Also check multiple of bb
            // if (raise_amt - self.game_state.current_to_call) % self.game_state.bb != 0 {
            //     return Err(format!(
            //         "Player #{} {} tried to raise {} but must be a multiple of big blind {}",
            //         player_index, &player_state.player_name, raise_amt, self.game_state.bb
            //     )
            //     .into());
            // }
        }

        if actual_increase > player_state.stack {
            return Err(format!(
                "Player #{} {} tried to raise {} but only has {}",
                player_index, &player_state.player_name, raise_amt, player_state.stack
            )
            .into());
        }

        Ok(())
    }

    pub fn to_game_log(&self) -> Result<GameLog, PokerError> {
        let players: Vec<InitialPlayerState> = self
            .game_state
            .player_states
            .iter()
            .map(|p| {
                let hole_cards = self
                    .game_runner_source
                    .get_hole_cards(p.player_index())
                    .unwrap();

                InitialPlayerState {
                    player_name: p.player_name.clone(),
                    stack: p.initial_stack,
                    position: p.position,
                    cards: Some(hole_cards),
                }
            })
            .collect();

        let board: Vec<Card> = self.game_state.board.as_slice_card().to_vec();

        let actions = self.game_state.actions.clone();

        let final_stacks: Vec<ChipType> = self
            .game_state
            .player_states
            .iter()
            .map(|p| p.stack)
            .collect();

        let final_states: Vec<FinalPlayerState> = self
            .game_state
            .player_states
            .iter()
            .map(|p| p.final_state.unwrap().clone())
            .collect();

        let game_log: GameLog = GameLog {
            players,
            sb: self.game_state.sb,
            bb: self.game_state.bb,
            board,
            actions,
            final_stacks,
            final_states,
            //Don't calculate yet as it's expensive
            best_player_hands: vec![],
            player_ranks_per_round: vec![],
        };

        Ok(game_log)
    }
}

#[cfg(test)]
mod tests {
    use log::debug;

    use crate::{
        game::game_log_source::GameLogSource, init_test_logger, test_game_runner, GameLog,
    };

    use super::*;

    #[test]
    fn test_game_runner_basic() {
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
Plyr C calls 10   # UTG acts first
Plyr D raises 10 to 20
Plyr A calls 7 # all in
Plyr B raises 10 to 30
Plyr C folds
Plyr D calls 10
*** Flop ***
2s 7c 8s
Plyr B bets 10
Plyr D calls 10
*** Turn ***
2h
Plyr B bets 10
Plyr D folds
*** River ***
2d
Plyr A bets 15 # This never gets used
Plyr B raises 10 to 30 # minimum raise
Plyr A raises 10 to 45
Plyr B calls 10
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
            game_runner.game_state.board.as_slice_card(),
            Board::try_from("2s 7c 8s").unwrap().as_slice_card()
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
            game_runner.game_state.board.as_slice_card(),
            Board::try_from("2s 7c 8s 2h").unwrap().as_slice_card()
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
            game_runner.game_state.board.as_slice_card(),
            Board::try_from("2s 7c 8s 2h 2d").unwrap().as_slice_card(),
        );

        //river actions

        //assert_eq!(true, game_runner.process_next_action().unwrap());
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
Player A1 calls 5   # UTG acts first
Player B2 calls 5
Player A2 calls 5
Player B1 calls 5
Player A4 raises 98 to 103
Player A3 calls 103
Player C2 calls 103
Player D1 calls 103
Player C1 calls 102
Player B3 calls 98
Player A1 calls 95 # all in
Player B2 calls 98
Player A2 calls 98
Player B1 calls 98
*** Flop ***
Ts 9h 8c
Player C1 checks
Player B3 checks
Player B2 checks
Player A2 checks
Player B1 checks
Player A3 bets 27 # all in for 130
Player C2 calls 27
Player D1 calls 27
Player C1 calls 27
Player B3 calls 27
Player B2 calls 27
Player A2 calls 7 # all in
Player B1 calls 27
*** Turn ***
6h
Player C1 bets 100
Player B3 raises 1 to 101 # all in
Player B2 calls 90 # all in ? 
Player B1 calls 70 # all in ?
Player C2 calls 101
Player D1 raises 169 to 270 # all in
Player C1 calls 110 # all in ? 
Player C2 calls 89 # all in ? 
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

    #[test]
    fn test_3way_all_in_river() {
        init_test_logger();

        let hh = "
*** Players ***
L1 - 100 - Kd Kh
W1 - 100 - Ad Qs
L2 - 100 - 2d 2h
W2 - 100 - Ah Qc
W3 - 100 - Ac Qd
*** Blinds ***
L1 - 1
W1 - 5
*** Preflop ***
L2 raises 5 to 10
W2 raises 5 to 15
W3 raises 5 to 20
L1 raises 10 to 30
W1 raises 10 to 40
L2 raises 10 to 50
W2 calls 35
W3 calls 30
L1 folds
W1 raises 45 to 95
L2 folds
W2 calls 45
W3 calls 45
*** Flop ***
As 3d 4c
W1 checks
W2 checks
W3 checks
*** Turn ***
5d
W1 checks
W2 checks
W3 checks
*** River ***
7d
W1 checks
W2 checks
W3 bets 5
W1 calls 5
W2 calls 5
*** Summary ***
L1 - 70 # Lost 30
W1 - 126 # 380 / 3
L2 - 50 # Lost 50
W2 - 126 # 
W3 - 126 # 
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

    #[test]
    fn test_cbet_win() {
        init_test_logger();

        let hh = "
*** Players ***
L1 - 100 - Kd Kh
W1 - 100 - Ad Qs
L2 - 100 - 2d 2h
W2 - 100 - Ah Qc
W3 - 100 - Ac Qd
*** Blinds ***
L1 - 1
W1 - 5
*** Preflop ***
L2 raises 5 to 10
W2 raises 5 to 15
W3 raises 5 to 20
L1 raises 10 to 30
W1 raises 10 to 40
L2 raises 10 to 50
W2 calls 35
W3 calls 30
L1 folds
W1 raises 45 to 95
L2 folds
W2 calls 45
W3 calls 45
*** Flop ***
As 3d 4c
W1 checks
W2 checks
W3 bets 5
W1 folds
W2 folds
*** Summary ***
L1 - 70 # Lost 30
W1 - 5 # Lost 95
L2 - 50 # Lost 50
W2 - 5 # Lost 95
W3 - 370 #  Wins everything
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

    #[test]
    fn test_all_in_preflop_win() {
        init_test_logger();

        let hh = "
*** Players ***
L1 - 80 - 3c 2h
W1 - 110 - Ad Ac
L2 - 95 - 2d 2c
W2 - 100 - Kh Kd
W3 - 100 - Ks Kc
*** Blinds ***
L1 - 1
W1 - 5
*** Preflop ***
L2 raises 5 to 10
W2 raises 5 to 15
W3 calls 15
L1 raises 15 to 30
W1 raises 80 to 110
L2 calls 85
W2 calls 85
W3 calls 85
L1 folds
*** Flop ***
As 8d 4c
*** Turn ***
5d
*** River ***
7d
*** Summary ***
L1 - 50 # Lost 30
W1 - 435 # Wins 295+110+50
L2 - 0 # Lost 50
W2 - 0 
W3 - 0
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

    #[test]
    fn test_fold_in_preflop_win() {
        init_test_logger();

        let hh = "
*** Players ***
L1 - 80 - 3c 2h
W1 - 110 - Ad Ac
L2 - 95 - 2d 2c
W2 - 100 - Kh Kd
W3 - 100 - Ks Kc
*** Blinds ***
L1 - 1
W1 - 5
*** Preflop ***
L2 folds
W2 folds
W3 folds
L1 folds
*** Summary ***
L1 - 79
W1 - 111
L2 - 95
W2 - 100
W3 - 100
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

    #[test]
    fn test_normal_3way_showdown() {
        init_test_logger();

        let hh = "
*** Players ***
L1 - 80 - Ac Ah
L2 - 110 - Ad As
L3 - 195 - 2d 2c
L4 - 100 - Kh Kd
W1 - 100 - 2s 7c
*** Blinds ***
L1 - 1
L2 - 5
*** Preflop ***
L3 calls 5
L4 raises 15 to 20
W1 calls 20
L1 calls 19
L2 raises 15 to 35
L3 calls 30
L4 raises 15 to 50
W1 calls 30
L1 calls 30
L2 calls 15
L3 calls 15
*** Flop ***
7d 7h 7s
L1 checks
L2 checks
L3 checks
L4 bets 45
W1 calls 45
L1 folds
L2 calls 45
L3 calls 45
*** Turn ***
8d 
L2 checks
L3 checks
L4 checks
W1 checks
*** River ***
9d
L2 checks
L3 checks
L4 checks
W1 checks
*** Summary ***
L1 - 30
L2 - 15
L3 - 100
L4 - 5
W1 - 435
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

    #[test]
    fn test_useless_raise_to_all_in() {
        init_test_logger();

        let hh = "
*** Players ***
L1 - 80 - 2c 2h
L2 - 110 - 2d 2s
L3 - 195 - Ad Ac
*** Blinds ***
L1 - 1
L2 - 5
*** Preflop ***
L3 calls 5
L1 raises 75 to 80 # all in
L2 raises 30 to 110 # all in  
L3 raises 85 to 195 # all in, useless
*** Flop ***
7d 7h 7s
*** Turn ***
8d 
*** River ***
9d
*** Summary ***
L1 - 0
L2 - 0
L3 - 385 # gets his useless all in back
    ";
        let game_log: GameLog = hh.parse().unwrap();

        let game_log_source = GameLogSource::new(game_log);

        let mut game_runner = GameRunner::new(GameRunnerSourceEnum::from(game_log_source)).unwrap();

        test_game_runner(&mut game_runner).unwrap();
    }
}
