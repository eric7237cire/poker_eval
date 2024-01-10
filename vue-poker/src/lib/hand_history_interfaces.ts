export interface HandHistory {
  players: Player[];
  sb: number;
  bb: number;
  board: Board[];
  actions: Action[];
  final_stacks: number[];
  final_states: Array<FinalState>;
  //index 0 -- round flop turn river
  //index 1 -- player
  //index 2 -- hand
  best_player_hands: Array<Array<Array<Card>>>;
}

export type FinalState = 'Folded' | 'WonShowdown' | 'LostShowdown';

export interface Player {
  stack: number;
  player_name: string;
  position: Position;
  cards: Cards;
}

export interface Position {
  pos: number;
}

export interface Cards {
  card_hi_lo: Card[];
}

export interface Card {
  value: string;
  suit: string;
  index: number;
}

export interface Board {
  value: string;
  suit: string;
  index: number;
}

export type Round = 'Preflop' | 'Flop' | 'Turn' | 'River';

export interface Action {
  index: number;
  player_index: number;
  action: FoldAction | RaiseAction | BetAction | CallAction | CheckAction;
  round: Round;
  player_comment: string;
  pot: number;
  current_amt_to_call: number;
  amount_put_in_pot_this_round: number;
  total_amount_put_in_pot: number;
  players_left_to_act: number;
  non_folded_players: number;
  is_all_in: boolean;
}

export type FoldAction = 'Fold';
export type CheckAction = 'Check';

export interface RaiseAction {
  Raise: [number, number];
}
export interface BetAction {
  Bet: number;
}
export interface CallAction {
  Call: number;
}
