
export interface PercOrBetter {
    perc: number;
    better: number;
  }

export interface ResultsInterface {
    equity: number;
    rank_family_count: Array<PercOrBetter>;
}

 