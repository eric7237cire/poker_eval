export interface PercOrBetter {
  perc: number;
  better: number;
}

export interface ResultsInterface {
  equity: number;
  rank_family_count: Array<PercOrBetter>;
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
  'Straight Flush',
];
export function getRankFamilyName(rank_family_index: number) {
  return RANK_FAMILY_NAMES[rank_family_index];
}
