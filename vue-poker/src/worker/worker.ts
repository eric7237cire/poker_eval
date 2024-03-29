import * as Comlink from 'comlink';
import { Draws, FlopSimulationResults, PlayerFlopResults } from '@pkg/poker_eval';
import { PercOrBetter, ResultsInterface, StreetResults } from './result_types';
import * as _ from 'lodash';

//import { detect } from "detect-browser";

type Mod = typeof import('@pkg/poker_eval');

let rankIndexes = [0, 1, 2, 3, 4, 5, 6, 7, 8];

const createHandler = (mod: Mod) => {
  return {
    flop_analyzer: mod.flop_analyzer.new(),
    results: null as null | FlopSimulationResults,

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

      if (!this.results) {
        console.error('results not initialized');
        return false;
      } else {
        return true;
      }
    },
    simulateFlop(num_iterations: number, equity_only: boolean) {
      if (!this.results) {
        console.error('results not initialized');
        return false;
      }
      this.results = this.flop_analyzer.simulate_flop(num_iterations, this.results, equity_only);
      return true;
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

      for (
        let active_player_index = 0;
        active_player_index < n_active_players;
        ++active_player_index
      ) {
        let ri = buildResultsInterface(r, active_player_index);
        ret.push(ri);
      }

      //Add villians
      ret.push(buildResultsInterface(r, undefined));

      return ret;
    },

    narrowRange(
      range_to_narrow: string,
      opponent_ranges: Array<string>,
      min_equity: number,
      cards: Uint8Array,
      num_simulations: number
    ): string {
      const opp_range_str = opponent_ranges.join(';');
      const result = this.flop_analyzer.narrow_range(
        range_to_narrow,
        opp_range_str,
        min_equity,
        cards,
        num_simulations
      );
      return result;
    },

    narrowRangeByPref(
      range_to_narrow: string,
      likes_hand_min_level: number,
      cards: Uint8Array,
      num_active_players: number
    ): string {
      const result = this.flop_analyzer.narrow_range_by_pref(
        range_to_narrow,
        likes_hand_min_level,
        cards,
        num_active_players
      );
      return result;
    }
  };
};

function buildResultsInterface(
  r: FlopSimulationResults,
  active_player_index: number | undefined
): ResultsInterface {
  const street_results: Array<StreetResults> = [];
  const draw_results: Array<Draws> = [];

  //flop/turn/river
  for (let i = 0; i < 3; i++) {
    const sr: StreetResults = {
      equity: r.get_equity(active_player_index, i),
      win_rank_family_count: rankIndexes.map((ri) => {
        return {
          perc: r.get_perc_family(active_player_index, i, ri, true),
          better: r.get_perc_family_or_better(active_player_index, i, ri, true)
        } as PercOrBetter;
      }),
      lose_rank_family_count: rankIndexes.map((ri) => {
        return {
          perc: r.get_perc_family(active_player_index, i, ri, false),
          better: r.get_perc_family_or_better(active_player_index, i, ri, false)
        } as PercOrBetter;
      }),
      eq_by_simple_range_idx: [],
      it_num_by_simple_range_idx: []
    };

    const r_eq = r.get_range_equity(active_player_index, i);
    const r_it = r.get_range_it_count(active_player_index, i);

    //assert(r_eq.length === r_it.length);

    const eq_range = [] as Array<number | null>;

    for (let ri = 0; ri < r_eq.length; ++ri) {
      if (r_it[ri] > 0) {
        eq_range.push(r_eq[ri] / r_it[ri]);
      } else {
        eq_range.push(null);
      }
    }

    sr.eq_by_simple_range_idx = eq_range;
    sr.it_num_by_simple_range_idx = r_it;

    street_results.push(sr);
  }

  //flop & river
  for (let street_idx = 0; street_idx < 2; ++street_idx) {
    draw_results.push(r.get_street_draw(active_player_index, street_idx));
  }

  let player_index = -1;

  if (active_player_index !== undefined) {
    player_index = r.get_player_index(active_player_index);
  }

  return {
    player_index,
    street_results,
    draw_results
  };
}

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
