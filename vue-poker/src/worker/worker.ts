import * as Comlink from 'comlink';
import { Draws, FlopSimulationResults, PlayerFlopResults } from '@pkg/poker_eval';
import { PercOrBetter, ResultsInterface, StreetResults } from './result_types';
//import { detect } from "detect-browser";

type Mod = typeof import('@pkg/poker_eval');

let rankIndexes = [0, 1, 2, 3, 4, 5, 6, 7, 8];

const createHandler = (mod: Mod) => {
  return {
    flop_analyzer: mod.flop_analyzer.new(),
    results: null as null|FlopSimulationResults,
    //player_flop_results: Array<PlayerFlopResults> = [],

    reset() {
      this.flop_analyzer.reset();
    },
    setBoardCards(cards: Uint8Array) {
      this.flop_analyzer.set_board_cards(cards);
    },
    setPlayerCards(player_idx: number, cards: Uint8Array) {
      this.flop_analyzer.set_player_cards(player_idx, cards);
    },
    setPlayerRange(player_idx: number, range_str: string) {
      this.flop_analyzer.set_player_range(player_idx, range_str);
    },
    setPlayerState(player_idx: number, state: number) {
      this.flop_analyzer.set_player_state(player_idx, state);
    },
    clearPlayerCards(player_idx: number) {
      this.flop_analyzer.clear_player_cards(player_idx);
    },
    initResults() {
      this.results = this.flop_analyzer.build_results();
      console.log(`initResults`);
    },
    simulateFlop(num_iterations: number) {
      if (!this.results) {
        console.error('results not initialized');
        return;
      }
      this.results = this.flop_analyzer.simulate_flop(
        num_iterations,
        this.results
      );
    },
    getResults(): Array<ResultsInterface> {
      console.log('getResults');
      //const r = this.flop_analyzer.get_results();
      const r = this.results;
      if (!r) {
        console.error('results falsy');
        return [];
      }
      //console.log(`getResults ${r[0].num_iterations} ${r[0].get_perc_family_or_better(1)}`);
      const n_active_players = r.get_num_players();

      const ret = [] as Array<ResultsInterface>;

      for (let active_player_index = 0; active_player_index < n_active_players; ++active_player_index) {
      
        const street_results: Array<StreetResults> = [];
        const draw_results: Array<Draws> = [];

        //flop/turn/river
        for (let i = 0; i < 3; i++) {
          street_results.push({
            equity: r.get_equity(active_player_index, i),
            rank_family_count: rankIndexes.map((ri) => {
              return {
                perc: r.get_perc_family(active_player_index, i, ri),
                better: r.get_perc_family_or_better(active_player_index, i, ri)
              } as PercOrBetter;
            })
          } as StreetResults);
        }

        //flop & river
        for (let street_idx = 0; street_idx < 2; ++street_idx) {
          draw_results.push(r.get_street_draw(active_player_index, street_idx));
        }

        ret.push( {
          player_index: r.get_player_index(active_player_index),
          street_results,
          draw_results
        } );
      }
      
      return ret;
    }
  };
};

// const isMTSupported = () => {
//   const browser = detect();
//   return !(browser && (browser.name === "safari" || browser.os === "iOS"));
// };

let mod: Mod | null = null;
export type Handler = ReturnType<typeof createHandler>;

const initHandler = async (num_threads: number) => {
  //   if (isMTSupported()) {
  //     mod = await import("../pkg/solver-mt/solver.js");
  //     await mod.default();
  //     await (mod as ModMT).initThreadPool(num_threads);
  //   } else {
  //     mod = await import("../pkg/solver-st/solver.js");
  //     await mod.default();
  //   }

  mod = await import('../../pkg/poker_eval');
  await mod.default();

  return Comlink.proxy(createHandler(mod));
};

const beforeTerminate = async () => {
  //   if (isMTSupported()) {
  //     await (mod as ModMT).exitThreadPool();
  //   }
};

export interface WorkerApi {
  initHandler: typeof initHandler;
  beforeTerminate: typeof beforeTerminate;
}

Comlink.expose({ initHandler, beforeTerminate });
