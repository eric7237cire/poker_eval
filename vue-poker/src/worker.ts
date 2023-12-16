import * as Comlink from 'comlink';
import { Results } from '../pkg/poker_eval';
//import { detect } from "detect-browser";

type Mod = typeof import('../pkg/poker_eval');

const createHandler = (mod: Mod) => {
  return {
    flop_analyzer: mod.flop_analyzer.new(),

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
    simulateFlop(num_iterations: number) {
      this.flop_analyzer.simulate_flop(num_iterations);
    },
    getResults(): Array<Results> {
      return this.flop_analyzer.get_results();
    },
    getResult(player_idx: number): Results {
      return this.flop_analyzer.get_result(player_idx);
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

  mod = await import('../pkg/poker_eval');
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
