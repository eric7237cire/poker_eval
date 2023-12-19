import { Draws } from '@pkg/poker_eval';

export interface PercOrBetter {
  perc: number;
  better: number;
}

export interface StreetResults {
  equity: number;
  rank_family_count: Array<PercOrBetter>;

  //index is simple range index, 0 to 169, 0 is AA, 1 is AKs, 2 is AQs, 13 is AKo, 14 is KK
  eq_by_simple_range_idx: Array<null | number>;

}

export interface ResultsInterface {
  player_index: number;

  //flop, turn, river
  street_results: Array<StreetResults>;

  draw_results: Array<Draws>;
}

export const RANK_FAMILY_NAMES = [
  'High Card',
  'Pair',
  'Two Pair',
  'Three of a Kind',
  'Straight',
  'Flush',
  'Full House',
  'Four of a Kind',
  'Straight Flush'
];
export function getRankFamilyName(rank_family_index: number) {
  return RANK_FAMILY_NAMES[rank_family_index];
}
