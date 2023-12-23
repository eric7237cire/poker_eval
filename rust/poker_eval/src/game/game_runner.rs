//maintain player states and game state
// and we have 2 playbacks, either agent or a log
// via trait that will do

use std::cmp::max;

use crate::{
    ActionEnum, Card, CardUsedType, ChipType, GameLog, GameState, HoleCards, InitialPlayerState,
    PlayerState, PokerError, Position, Round,
};
use enum_dispatch::enum_dispatch;
use log::trace;


// Enforces the poker rules
pub struct GameRunner {
    used_cards: CardUsedType,

    pub game_state: GameState,

    // Source of actions, cards
    pub game_runner_source: GameRunnerSourceEnum,
}

#[enum_dispatch]
pub enum GameRunnerSourceEnum {
    GameLogSource,
    //LogarithmicKnob,
}

#[enum_dispatch(GameRunnerSourceEnum)]
pub trait GameRunnerSource {
    fn get_initial_players(&self) -> &[InitialPlayerState];

    fn get_small_blind(&self) -> ChipType;
    fn get_big_blind(&self) -> ChipType;

    fn get_action(&mut self, player_state: &PlayerState, game_state: &GameState) -> ActionEnum;

    //get cards for player?
    fn get_hole_cards(&self, player_index: usize) -> Result<HoleCards, PokerError>;

    //get board cards?
    fn get_next_board_card(&mut self) -> Result<Card, PokerError>;
}

pub struct GameLogSource {
    game_log: GameLog,
    cur_action: usize,
    cur_board_card: usize,
}

impl GameLogSource {
    pub fn new(game_log: GameLog) -> Self {
        GameLogSource {
            game_log,
            cur_action: 0,
            cur_board_card: 0,
        }
    }
}

impl GameRunnerSource for GameLogSource {
    fn get_initial_players(&self) -> &[InitialPlayerState] {
        &self.game_log.players
    }

    fn get_small_blind(&self) -> ChipType {
        self.game_log.sb
    }

    fn get_big_blind(&self) -> ChipType {
        self.game_log.bb
    }

    fn get_action(&mut self, _player_state: &PlayerState, _game_state: &GameState) -> ActionEnum {
        let action = self.game_log.actions[self.cur_action].action;
        self.cur_action += 1;
        action
    }

    fn get_hole_cards(&self, player_index: usize) -> Result<HoleCards, PokerError> {
        if player_index >= self.game_log.players.len() {
            return Err(PokerError::from_string(format!(
                "Invalid player index {}",
                player_index
            )));
        }

        self.game_log.players[player_index]
            .cards
            .ok_or(PokerError::from_string(format!(
                "Player {} does not have hole cards",
                player_index
            )))
    }

    fn get_next_board_card(&mut self) -> Result<Card, PokerError> {
        if self.cur_board_card >= self.game_log.board.len() {
            return Err(PokerError::from_string(format!(
                "Invalid board card index {}",
                self.cur_board_card
            )));
        }
        let card = self.game_log.board[self.cur_board_card];
        self.cur_board_card += 1;
        Ok(card)
    }
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

        if amount < player_state.cur_round_putting_in_pot {
            return Err(PokerError::from_string(format!(
                "Player {} tried to put {} in pot, but already put in {}",
                player_state.player_name, amount, player_state.cur_round_putting_in_pot
            )));
        }

        let actual_amount = amount - player_state.cur_round_putting_in_pot;

        if player_state.stack <= actual_amount {
            player_state.all_in_for =
                Some(player_state.stack + player_state.cur_round_putting_in_pot);
            player_state.stack = 0;
            //max_pot is created when the round is done
            return Ok(());
        }

        assert!(player_state.stack > actual_amount);

        player_state.stack -= actual_amount;
        player_state.cur_round_putting_in_pot += actual_amount;
        assert_eq!(player_state.cur_round_putting_in_pot, amount);

        self.game_state.round_pot += actual_amount;

        Ok(())
    }

    fn active_player_count(&self) -> usize {
        self.game_state
            .player_states
            .iter()
            .filter(|p| !p.folded && p.all_in_for.is_none())
            .count()
    }

    fn calc_max_pot(&self, all_in_for: ChipType) -> ChipType {
        let mut max_pot = 0;

        for player_state in &self.game_state.player_states {
            max_pot += max(player_state.cur_round_putting_in_pot, all_in_for)
        }
        max_pot
    }

    fn check_pots_good(&self) -> Result<(), PokerError> {
        let mut check_round_pot = 0;

        //Do some sanity checks, each player either folded or put in the same amount or is all in
        for player_state in &self.game_state.player_states {
            check_round_pot += player_state.cur_round_putting_in_pot;
            if player_state.folded {
                continue;
            }
            if player_state.all_in_for.is_some() {
                self.game_state.current_to_call.ok_or(format!("
                Player {} is all in for {} but there is no current to call", 
                &player_state.player_name, player_state.all_in_for.unwrap())
                )?;
                
                continue;
            }
            if let Some(current_to_call) = self.game_state.current_to_call {
                if player_state.cur_round_putting_in_pot != current_to_call {
                    return Err(format!(
                        "Player {} has put in {} but current to call is {}",
                        player_state.player_name,
                        player_state.cur_round_putting_in_pot,
                        current_to_call
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

    fn move_to_next_round(&mut self) -> Result<(), PokerError> {
        trace!("Move to next round");

        
        trace!("Check all players have called/folded/or are all in");

        self.check_pots_good()?;
        //calculate max_pot
        let player_count = self.game_state.player_states.len();
        for player_index in 0..self.game_state.player_states.len() {
            
            if let Some(all_in) = self.game_state.player_states[player_index].all_in_for {
                let max_pot = self.calc_max_pot(all_in);
                self.game_state.player_states[player_index].max_pot = Some(max_pot);
            }
        }

        self.game_state.prev_round_pot += self.game_state.round_pot;
        self.game_state.round_pot = 0;
        self.game_state.current_to_call = None;
        self.game_state.min_raise = 0;
        
        self.game_state.current_round = self.game_state.current_round.next().ok_or(format!(
            "No next round {}",
            self.game_state.current_round
        ))?;

        self.game_state.current_to_act = Position::first_to_act(player_count as _, self.game_state.current_round);


        Ok(())
    }

    fn finish(&mut self) -> Result<(), PokerError> {
        trace!("Finish game");
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

        //If current player is all-in or folded, go to next player
        //The assumption is this would never finish the round
        if self.game_state.player_states[player_index].all_in_for.is_some()
            || self.game_state.player_states[player_index].folded
        {
            self.game_state.current_to_act = self.game_state.current_to_act.next(self.game_state.player_states.len() as _);
            let action_count = self.game_state.actions.len();
            let ok = self.process_next_action()?;
            assert_eq!(action_count+1, self.game_state.actions.len());
            return Ok(ok);
        }


        //Do we need to move to the next round?
        if let Some(amt_to_call) = self.game_state.current_to_call {
            //If current player has called the amount needed we move to the next round
            if amt_to_call == self.game_state.player_states[player_index].cur_round_putting_in_pot {
                if self.game_state.current_round == Round::River {
                    self.finish()?;
                    return Ok(true);
                }
                let cur_round = self.game_state.current_round;
                self.move_to_next_round()?;
                assert_eq!(cur_round.next().unwrap(), self.game_state.current_round);
                return self.process_next_action();
            }
        }

        let action = self.game_runner_source.get_action(
            &self.game_state.player_states[player_index],
            &self.game_state,
        );

        trace!(
            "Player #{} named {} does action {}",
            player_index,
            &self.game_state.player_states[player_index].player_name,
            action
        );

        match action {
            ActionEnum::Fold => {
                self.game_state.player_states[player_index].folded = true;
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

                if raise_amt < self.game_state.min_raise + amt_to_call {
                    return Err(format!(
                        "Player #{} {} tried to raise {} but needs to be at least {} more than {}",
                        player_index, 
                        &self.game_state.player_states[player_index].player_name,
                        raise_amt, self.game_state.min_raise, amt_to_call
                    ).into());
                }

                self.game_state.min_raise = raise_amt - amt_to_call;
                self.game_state.current_to_call = Some(raise_amt);
                self.handle_put_money_in_pot(player_index,  raise_amt)?;
            }
            ActionEnum::Check => {

                if self.game_state.current_to_call.is_some() {
                    return Err(format!(
                        "Player #{} {} tried to check but there is a current to call",
                        player_index, 
                        &self.game_state.player_states[player_index].player_name,
                    ).into());
                }

                //Do nothing
            }
            ActionEnum::Bet(bet_amt) => {
                if self.game_state.current_to_call.is_some() {
                    return Err(format!(
                        "Player #{} {} tried to bet but there is a current to call, must call or raise or fold",
                        player_index, 
                        &self.game_state.player_states[player_index].player_name,
                    ).into());
                }
                self.game_state.min_raise = bet_amt;
                self.game_state.current_to_call = Some(bet_amt);

                self.handle_put_money_in_pot(player_index, bet_amt)?;
                
            }
            
        }

        self.game_state.current_to_act = self.game_state.current_to_act.next(self.game_state.player_states.len() as _);
            

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::init_test_logger;

    use super::*;

    #[test]
    fn test_game_runner() {
        init_test_logger();

        let hh = "
*** Players *** 
Plyr A - 12 - As Kh
Plyr B - 147 - 2d 2c
Plyr C - 55 - 7d 2h
Plyr D - 55 - Ks Kd
*** Blinds *** 
Plyr A - 5
Plyr B - 10
*** Preflop ***
Plyr C calls    # UTG acts first
Plyr D raises 20
Plyr A calls 
Plyr B raises 35 # so puts in an additional 15
Plyr C folds
Plyr D calls
Plyr A calls
*** Flop ***
2s 7c 8s
Plyr A checks
Plyr B bets 5
Plyr D calls
Plyr A calls
*** Turn ***
2h 
Plyr A checks
Plyr B bets 5
Plyr D folds
Plyr A calls
*** River ***
2d
Plyr A bets 15
Plyr B raises 30 # minimum raise
Plyr A raises 45
Plyr B calls
*** Summary ***
Plyr A - 12 # though this is not valid, the parsing just wants correct syntax
Plyr B - 148 # Plyr B loses 100 with 2h As Kh 2d 7c
Plyr C - 55 # can put in comments showdown, wins / losses side pot / etc
Plyr D - 90
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
            20
        );

        for _ in 0..5 {
            game_runner.process_next_action().unwrap();
        }

        assert_eq!(game_runner.game_state.current_round, Round::Flop);
        assert_eq!(game_runner.game_state.player_states[0].all_in_for, Some(12));
        assert_eq!(game_runner.game_state.prev_round_pot, 12+35*3);
        assert_eq!(game_runner.game_state.round_pot, 0);
    }
}
