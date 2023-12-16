import * as Comlink from 'comlink';
//import { detect } from "detect-browser";

type Mod = typeof import('../pkg/poker_eval');

const createHandler = (mod: Mod) => {
  return {
    flop_analyzer: mod.FlopAnalyzer.new(),

    reset(num_players: number, player_ranges: string[]) {
      this.flop_analyzer.reset(num_players, player_ranges);
    },
    setBoardCards(card_str: string) {
      this.flop_analyzer.set_board_cards(card_str);
    },
    setPlayerCards(player_idx: number, card_str: string) {
      this.flop_analyzer.set_player_cards(player_idx, card_str);
    },
    clearPlayerCards(player_idx: number) {
      this.flop_analyzer.clear_player_cards(player_idx);
    },
    simulateFlop(num_iterations: number) {
      this.flop_analyzer.simulate_flop(num_iterations);
    },
    getResults() {

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
