//Need a game_runner, except one position will be the agent we're training
//When it's the agents turn, we get an array of actions from it it would like to prototype

// The actions go into a queue which holds --
// Infostate of agent (or id of it)
// Gamestate

// Once this gamestate reaches the end of the hand, update the agents data
// with infostate + action == result (chips won/lost in bb)

//For subsequent actions, we'll maybe just have additional infostates to update

use log::info;

use crate::{game::{runner::{GameRunnerSource, GameRunner}, core::{CommentedAction, ActionEnum}}, Card};

pub struct AgentTrainer {}


pub fn run_full_game_tree<T: GameRunnerSource>(game_source: &mut T, board: Vec<Card>, hero_index: usize) {
    let game_runner = GameRunner::new(
        game_source.get_initial_players(),
        game_source.get_small_blind(),
        game_source.get_big_blind(),
        &board,
    )
    .unwrap();

    let mut game_runner_queue = Vec::new();
    game_runner_queue.push(game_runner);

    while !game_runner_queue.is_empty() {

        let mut current_game_runner = game_runner_queue.pop().unwrap();

        //Need to check if this is already done
        if current_game_runner.game_state.player_states[0].final_state.is_some() {
            process_finished_gamestate(current_game_runner);
            continue;
        }

        //action loop
        for _ in 0..2000 {
            

            //If we have a choice, need to run with those choices, then actually take the best one to
            //evaluate the value of preceeding actions

            //In a game tree, if our agent being trained has 3 choices, to evaluate the 1st choice
            //We need the value of the best choice

            if current_game_runner.get_current_player_state().player_index() == hero_index {
                //let action_choices = [...]

                //If we have a value already evaluated for those action choices, we take it and
                //move on

                //Otherwise we queue all the possible choices, and also this game state
                //We don't need to run the choice again, we should have its expected value

                //Action number is not perfect since rounds can have varaiable # of actions

                /*
                OO A1
                OO A2
                OO A3

                OO A3 MM A4
                OO A3 MM A5
                OO A3 MM A6

                OO A3 MM A4 NN A7
                OO A3 MM A4 NN A8
                OO A3 MM A4 NN A9

                OO A3 MM A4 NN A9 P [45]
                OO A3 MM A4 NN A8 Q [99]

                give each action an integer, maybe action number will work

                clear every action in gamestate that is not heros

                Then at end, get info states for each action, update equity for that infostate

                InfoState can be derived from PlayerAction
                */

                let hero_helpers = current_game_runner
                    .get_current_player_state()
                    .get_helpers(&current_game_runner.game_state);
                //Are we facing a bet?
                if current_game_runner.game_state.current_to_call > 0 {
                    //Fold, call, raise
                    let mut fold_game_runner = current_game_runner.clone();
                    let fold_action = CommentedAction {
                        action: ActionEnum::Fold,
                        comment: None,
                    };
                    fold_game_runner.process_next_action(&fold_action).unwrap();
                    game_runner_queue.push(fold_game_runner);

                    let mut call_game_runner = current_game_runner.clone();
                    let call_action = CommentedAction {
                        action: ActionEnum::Call(hero_helpers.call_amount),
                        comment: None
                    };
                    call_game_runner.process_next_action(&call_action).unwrap();
                    game_runner_queue.push(call_game_runner);

                    if hero_helpers.can_raise {
                        let mut raise_game_runner = current_game_runner.clone();
                        let raise_action = hero_helpers.build_raise_to(&raise_game_runner.game_state, 
                                raise_game_runner.game_state.current_to_call * 3, "".to_string());
                        
                        raise_game_runner
                            .process_next_action(&raise_action)
                            .unwrap();
                        game_runner_queue.push(raise_game_runner);
                    }

                    //out of action loop
                    break;


                } else {
                    //no bet
                    let mut check_game_runner = current_game_runner.clone();
                    let check_action = CommentedAction {
                        action: ActionEnum::Check,
                        comment: None
                    };
                    break;
                }
            } else {
                let action = game_source
                    .get_action(
                        current_game_runner.get_current_player_state(),
                        &current_game_runner.game_state,
                    )
                    .unwrap();
                let r = current_game_runner.process_next_action(&action).unwrap();

                //We don't keep the action if it's not the hero's
                //current_game_runner.game_state.actions.pop().unwrap();

                if r {
                    process_finished_gamestate(current_game_runner);
                    break;
                }
            }
            
        }
    }
}

fn process_finished_gamestate(game_runner: GameRunner) {
    info!("Processing finished gamestate");

    //assign the max ev for each infostate in the game
    for action in game_runner.game_state.actions.iter() {
        //info!("Action {}", action);
    }

    info!("{}", game_runner.to_game_log().unwrap().to_game_log_string(false, true, 1));
}


#[cfg(test)]
mod tests {
    use crate::{init_test_logger, game::{runner::GameRunnerSource, core::{InitialPlayerState, ChipType, PlayerState, GameState, CommentedAction, ActionEnum, Round}}, PokerError, HoleCards, Board};

    use super::run_full_game_tree;

    struct TestGameSource {
        //trained agent is position 1, everyone will fold only if he bets on the river
        //the other 2 players have better cards and will go all in on any bet on preflop/flop/turn
        //The test is the best ev for this setup is call preflop, check check, then bet only the river
        init_player_state: Vec<InitialPlayerState>
    }

    impl TestGameSource {
        fn new() -> Self {
            let mut ret = Vec::with_capacity(3);
            ret.push(InitialPlayerState{
                stack: 45,
                player_name: "P1".to_string(),
                position: 0.try_into().unwrap(),
                cards: Some("Kh Ks".parse().unwrap())
            });
            ret.push(InitialPlayerState{
                stack: 35,
                player_name: "Trainee".to_string(),
                position: 1.try_into().unwrap(),
                cards: Some("2h 2s".parse().unwrap())
            });
            ret.push(InitialPlayerState{
                stack: 55,
                player_name: "P2".to_string(),
                position: 2.try_into().unwrap(),
                cards: Some("Ah As".parse().unwrap())
            });
            Self {
                init_player_state: ret
            }
        }
    }

    impl GameRunnerSource for TestGameSource {
        fn get_initial_players(&self) ->  &[InitialPlayerState] {
            &self.init_player_state
        }

        fn get_small_blind(&self) -> ChipType {
            3
        }

        fn get_big_blind(&self) -> ChipType {
            19
        }

        fn get_action(&mut self,player_state: &PlayerState,game_state: &GameState,) -> Result<CommentedAction,PokerError> {
            //The agent being trained should not have an action
            assert_ne!(player_state.player_index(), 1 );

            let helper = player_state.get_helpers(game_state);

            if game_state.current_round == Round::River && game_state.current_to_call > 0 {
                return Ok(CommentedAction {
                    action: ActionEnum::Fold,
                    comment: None
                });
            }

            if game_state.current_round == Round::Preflop && game_state.current_to_call <= 19 && game_state.current_to_call > 0 {
                return Ok(CommentedAction {
                    action: ActionEnum::Call(helper.call_amount),
                    comment: None
                })
            }


            Ok(if game_state.current_to_call > 0 {
                helper.build_raise_to(game_state, helper.max_can_raise, "".to_string())
            } else {
                CommentedAction {
                    action: ActionEnum::Check,
                    comment: None
                }
            })
        }

        fn get_hole_cards(&self,player_index:usize) -> Result<HoleCards,PokerError> {
            Ok(self.init_player_state[player_index].cards.unwrap())
        }
    }

    #[test]
    fn test_run_full_game_tree() {
        init_test_logger();

        // cargo test --lib test_run_full_game_tree -- --nocapture

        let mut game_source = TestGameSource::new();
        let board: Board = "3s 4c 5h 7d 8h".parse().unwrap();
        run_full_game_tree(&mut game_source, board.as_slice_card().to_vec(), 1);


    }
}