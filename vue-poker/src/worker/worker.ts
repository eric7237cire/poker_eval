import * as Comlink from 'comlink';
import { Draws, PlayerFlopResults } from '@pkg/poker_eval';
import { PercOrBetter, ResultsInterface, StreetResults } from './result_types';
//import { detect } from "detect-browser";

type Mod = typeof import('@pkg/poker_eval');

let rankIndexes = [0, 1, 2, 3, 4, 5, 6, 7, 8];

const createHandler = (mod: Mod) => {
  return {
    flop_analyzer: mod.flop_analyzer.new(),
    player_flop_results: [] as Array<PlayerFlopResults>,
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
      this.player_flop_results = this.flop_analyzer.build_results();
      console.log(`initResults ${this.player_flop_results.length}`);
    },
    simulateFlop(num_iterations: number) {
      this.player_flop_results = this.flop_analyzer.simulate_flop(
        num_iterations,
        this.player_flop_results
      );
    },
    getResults(): Array<ResultsInterface> {
      console.log('getResults');
      //const r = this.flop_analyzer.get_results();
      const r = this.player_flop_results;
      //console.log(`getResults ${r[0].num_iterations} ${r[0].get_perc_family_or_better(1)}`);
      const ri = r.map((r) => {

        const street_results: Array<StreetResults> = [];
        const draw_results: Array<Draws> = [];

        for (let i = 0; i < 3; i++) {
          street_results.push({
            equity: r.get_equity(i),
            rank_family_count: rankIndexes.map((ri) => {
              return {
                perc: r.get_perc_family(i, ri),
                better: r.get_perc_family_or_better(i, ri)
              } as PercOrBetter;
            })
          } as StreetResults);
        }

        for(let i = 0; i < 2; ++i) {
          draw_results.push(r.get_street_draw(i));
        }

        return {
          player_index: r.player_index,
          street_results,
          draw_results
        } as ResultsInterface;
      });
      return ri;
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
