use log::debug;

use super::{GameLog, GameLogSource, GameRunner, GameRunnerSource, GameRunnerSourceEnum};

#[allow(dead_code)]
pub fn run_gamelog(game_log: GameLog) -> GameRunner {
    let board = game_log.board.clone();
    let game_log_source = GameLogSource::new(game_log);

    let mut game_source = GameRunnerSourceEnum::from(game_log_source);
    let mut game_runner = GameRunner::new(
        game_source.get_initial_players(),
        game_source.get_small_blind(),
        game_source.get_big_blind(),
        &board,
    )
    .unwrap();

    for _ in 0..200 {
        let action_count_before = game_runner.game_state.actions.len();
        let action = game_source
            .get_action(
                game_runner.get_current_player_state(),
                &game_runner.game_state,
            )
            .unwrap();
        let r = game_runner.process_next_action(&action).unwrap();
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

    game_runner
}

// pub fn test_game_runner(game_runner: &mut GameRunner) -> Result<(), PokerError> {
//     for _ in 0..200 {
//         let action_count_before = game_runner.game_state.actions.len();
//         let r = game_runner.process_next_action()?;
//         if r {
//             break;
//         }
//         let action_count_after = game_runner.game_state.actions.len();
//         // debug!(
//         //     "Last action: {}",
//         //     &game_runner.game_state.actions.last().as_ref().unwrap()
//         // );
//         assert_eq!(action_count_before + 1, action_count_after);
//     }

//     //let log_display = game_runner.to_game_log_string(true);
//     let game_log = game_runner.to_game_log().unwrap();
//     let check_log = game_log.to_game_log_string(false, false, 0);
//     debug!("log\n{}", check_log);

//     let parsed_game_log: GameLog = check_log.parse().unwrap();
//     let game_log_source: GameLogSource = GameLogSource::new(parsed_game_log);

//     //Run the game again with the log
//     let mut game_runner2 = GameRunner::new(GameRunnerSourceEnum::from(game_log_source)).unwrap();

//     for _ in 0..200 {
//         let action_count_before = game_runner2.game_state.actions.len();
//         let r = game_runner2.process_next_action()?;
//         if r {
//             break;
//         }
//         let action_count_after = game_runner2.game_state.actions.len();
//         assert_eq!(action_count_before + 1, action_count_after);
//     }

//     let log2 = game_runner2
//         .to_game_log()
//         .unwrap()
//         .to_game_log_string(false, false, 0);

//     //info!("log2:\n{}", log2);

//     assert_eq!(check_log, log2);

//     Ok(())
// }
